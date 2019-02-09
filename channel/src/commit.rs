use secp256k1::{PublicKey, SecretKey, Signature, Secp256k1, Message};
use bitcoin::util::hash::{Sha256dHash};
use bitcoin::blockdata::script::{Script};
use bitcoin::blockdata::transaction::{Transaction, TxIn, TxOut};
use bitcoin::util::bip143;
use bip69;
use tools::{get_sequence, get_locktime, accepted_htlc, offered_htlc, to_local_script, v0_p2wpkh, new_2x2_multisig};

pub const HTLC_TIMEOUT_WEIGHT: i64 = 663;
pub const HTLC_SUCCESS_WEIGHT: i64 = 703;
pub const BASE_COMMITMENT_WEIGHT: i64 = 724;
pub const PER_HTLC_COMMITMENT_WEIGHT: i64 = 172;

#[derive(Clone, Debug)]
pub enum HTLCDirection {
    Accepted,
    Offered,
}

#[derive(Debug, Clone)]
pub struct HTLC {
    pub direction: HTLCDirection,
    pub amount_msat: i64,
    pub expiry: i32,
    pub payment_hash: [u8; 32]
}

#[derive(Clone, Debug)]
pub struct CommitTx {
    pub funding_amount: i64,
    pub local_funding_pubkey: PublicKey,
    pub remote_funding_pubkey: PublicKey,

    pub local_feerate_per_kw: i64,
    pub dust_limit_satoshi: i64,

    pub to_local_msat: i64,
    pub to_remote_msat: i64,

    pub obscured_commit_number: u64,

    pub local_htlc_pubkey: PublicKey,
    pub remote_htlc_pubkey: PublicKey,

    pub local_revocation_pubkey: PublicKey,
    pub local_delayedpubkey: PublicKey,
    pub local_delay: u64,

    pub remotepubkey: PublicKey,

    pub funding_tx_id: Sha256dHash,
    pub funding_output_index: u32,

    pub htlcs: Vec<HTLC>,
}

impl CommitTx {
    pub fn get_tx(&self) -> Transaction {
        let sequence = get_sequence(self.obscured_commit_number);
        let locktime = get_locktime(self.obscured_commit_number);

        let mut tx = Transaction{
            version: 2,
            input: vec![TxIn{
                prev_hash: self.funding_tx_id,
                prev_index: self.funding_output_index,
                sequence: sequence as u32,
                script_sig: Script::new(),
                witness: vec![]
            }],
            output: vec![
            ],
            lock_time: locktime as u32
        };

        let mut weight: i64 = BASE_COMMITMENT_WEIGHT;
        for h in &self.htlcs {
            if self.is_htlc_trimmed(h) {
                continue
            }
            weight += PER_HTLC_COMMITMENT_WEIGHT;
            let lock_script = match h.direction {
                HTLCDirection::Accepted => accepted_htlc(&self.local_revocation_pubkey, &self.remote_htlc_pubkey, &self.local_htlc_pubkey, h.payment_hash, h.expiry as u32),
                HTLCDirection::Offered => offered_htlc(&self.local_revocation_pubkey, &self.remote_htlc_pubkey, &self.local_htlc_pubkey, h.payment_hash),
            };
            tx.output.push(TxOut{
                value: (h.amount_msat / 1000) as u64,
                script_pubkey: lock_script.to_v0_p2wsh(),
            })
        }

        let base_fee = (weight * self.local_feerate_per_kw) / 1000;
        // TODO(mkl): who pays it
        // TODO(mkl): what happens if it is negative
        // Assume that local pays fee
        let mut to_local = (self.to_local_msat / 1000) - base_fee;
        let mut to_remote = self.to_remote_msat / 1000;

        if to_local < 0 {
            to_local = 0;
            println!("too high fee encountered, to_local < 0, set to 0");
        }

        if to_remote < 0 {
            to_remote = 0;
            println!("too high fee encountered, to_remote < 0, set to 0");
        }

        // To self output
        if to_local >= self.dust_limit_satoshi {
            tx.output.push(TxOut{
                value: to_local as u64,
                script_pubkey: to_local_script(&self.local_delayedpubkey, self.local_delay as u64, &self.local_revocation_pubkey).to_v0_p2wsh(),
            });
        }

        // To remote output
        if to_remote >= self.dust_limit_satoshi {
            tx.output.push(TxOut{
                value: to_remote as u64,
                script_pubkey: v0_p2wpkh(&self.remotepubkey),
            });
        }

        bip69::reorder_tx(&mut tx);

        return tx;
    }

    fn htlc_timeout_fee(&self) -> i64 {
        return self.local_feerate_per_kw * HTLC_TIMEOUT_WEIGHT / 1000;
    }

    fn htlc_success_fee(&self) -> i64 {
        return self.local_feerate_per_kw * HTLC_SUCCESS_WEIGHT / 1000;
    }

    fn is_htlc_trimmed(&self, h: &HTLC) -> bool {
        let required = self.dust_limit_satoshi + match h.direction {
            HTLCDirection::Accepted => self.htlc_success_fee(),
            HTLCDirection::Offered => self.htlc_timeout_fee(),
        };
        return (h.amount_msat / 1000) < required;
    }

    pub fn sign(&self, priv_key: &SecretKey) -> Signature {
        let sec = Secp256k1::new();
        let tx = self.get_tx();

        let funding_lock_script = new_2x2_multisig(
            &self.local_funding_pubkey.serialize(),
            &self.remote_funding_pubkey.serialize()
        );
        let tx_sig_hash = bip143::SighashComponents::new(&tx)
            .sighash_all(
                &tx.input[0],
                &funding_lock_script,
                self.funding_amount as u64
            );
        // TODO(mkl): maybe do not use unwrap
        let sig = sec.sign(
            &Message::from(tx_sig_hash.data()),
            priv_key
        ).unwrap();
        sig
    }

    // TODO(mkl): add signature validation

}

#[cfg(test)]
mod tests {
    use spec_example::get_example;
    use tools::{s2tx, assert_tx_eq, spending_witness_2x2_multisig};
    use commit::CommitTx;
    use secp256k1::Secp256k1;
    use hex;

    #[test]
    fn test_simple_commitment_tx_with_no_htlcs() {
        let ex = get_example();

        // name: simple commitment tx with no HTLCs
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8002c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311054a56a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400473044022051b75c73198c6deee1a875871c3961832909acd297c6b908d59e3319e5185a46022055c419379c5051a78d00dbbce11b5b664a0c22815fbcc6fcef6b1937c383693901483045022100f51d2e566a70ba740fc5d8c0f07b9b93d2ed741c3c0860c613173de7d39e7968022041376d520e9c0e1ad52248ddf4b22e12be8763007df977253ef45a4ca3bdb7c001475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

        let commit_tx = CommitTx{
            funding_amount: ex.funding_amount_satoshi,
            local_funding_pubkey: ex.local_funding_pubkey,
            remote_funding_pubkey: ex.remote_funding_pubkey,

            local_feerate_per_kw: 15000,
            dust_limit_satoshi: 546,

            to_local_msat: 7000000000,
            to_remote_msat: 3000000000,
            obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

            local_htlc_pubkey: ex.localpubkey.clone(),
            remote_htlc_pubkey: ex.remotepubkey.clone(),

            local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
            local_delayedpubkey: ex.local_delayedpubkey.clone(),
            local_delay: ex.local_delay as u64,

            remotepubkey: ex.remotepubkey.clone(),

            funding_tx_id: ex.funding_tx_id.clone(),
            funding_output_index: ex.funding_output_index as u32,

            htlcs: vec![],
        };

        // Validate that transaction without witness is correct
        let mut tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);

        // Validate signing
        let ctx = Secp256k1::new();
        let local_sig = commit_tx.sign(&ex.local_funding_privkey);
        assert_eq!(
            hex::encode(local_sig.serialize_der(&ctx)),
            "3044022051b75c73198c6deee1a875871c3961832909acd297c6b908d59e3319e5185a46022055c419379c5051a78d00dbbce11b5b664a0c22815fbcc6fcef6b1937c3836939"
        );

        let remote_sig = commit_tx.sign(&ex.internal.remote_funding_privkey);
        assert_eq!(
            hex::encode(remote_sig.serialize_der(&ctx)),
            "3045022100f51d2e566a70ba740fc5d8c0f07b9b93d2ed741c3c0860c613173de7d39e7968022041376d520e9c0e1ad52248ddf4b22e12be8763007df977253ef45a4ca3bdb7c0",
        );

        tx.input[0].witness = spending_witness_2x2_multisig(
            &ex.local_funding_pubkey,
            &ex.remote_funding_pubkey,
            &local_sig,
            &remote_sig,
        );
        assert_tx_eq(&tx, &example_tx, false);
    }

    // Most of spec examples use the same commit transaction but with different fee
    fn get_base_commit_tx(local_feerate_per_kw: i64) -> CommitTx {
        let ex = get_example();
        let mut commit_tx = CommitTx{
            funding_amount: ex.funding_amount_satoshi,
            local_funding_pubkey: ex.local_funding_pubkey,
            remote_funding_pubkey: ex.remote_funding_pubkey,

            local_feerate_per_kw: local_feerate_per_kw,
            dust_limit_satoshi: 546,

            to_local_msat: 6988000000,
            to_remote_msat: 3000000000,
            obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

            local_htlc_pubkey: ex.localpubkey.clone(),
            remote_htlc_pubkey: ex.remotepubkey.clone(),

            local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
            local_delayedpubkey: ex.local_delayedpubkey.clone(),
            local_delay: ex.local_delay as u64,

            remotepubkey: ex.remotepubkey.clone(),

            funding_tx_id: ex.funding_tx_id.clone(),
            funding_output_index: ex.funding_output_index as u32,

            htlcs: vec![],
        };

        for h in &ex.htlcs {
            commit_tx.htlcs.push(h.to_htlc());
        }
        commit_tx
    }

    #[test]
    fn test_commitment_tx_with_all_five_htlcs_untrimmed_minimum_feerate() {
        // name: commitment tx with all five HTLCs untrimmed (minimum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8007e80300000000000022002052bfef0479d7b293c27e0f1eb294bea154c63a3294ef092c19af51409bce0e2ad007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110e0a06a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e04004730440220275b0c325a5e9355650dc30c0eccfbc7efb23987c24b556b9dfdd40effca18d202206caceb2c067836c51f296740c7ae807ffcbfbf1dd3a0d56b6de9a5b247985f060147304402204fd4928835db1ccdfc40f5c78ce9bd65249b16348df81f0c44328dcdefc97d630220194d3869c38bc732dd87d13d2958015e2fc16829e74cd4377f84d215c0b7060601475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(0);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }

    #[test]
    fn test_commitment_tx_with_seven_outputs_untrimmed_maximum_feerate() {
        // name: commitment tx with seven outputs untrimmed (maximum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8007e80300000000000022002052bfef0479d7b293c27e0f1eb294bea154c63a3294ef092c19af51409bce0e2ad007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110e09c6a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040048304502210094bfd8f5572ac0157ec76a9551b6c5216a4538c07cd13a51af4a54cb26fa14320220768efce8ce6f4a5efac875142ff19237c011343670adf9c7ac69704a120d116301483045022100a5c01383d3ec646d97e40f44318d49def817fcd61a0ef18008a665b3e151785502203e648efddd5838981ef55ec954be69c4a652d021e6081a100d034de366815e9b01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(647);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }

    #[test]
    fn test_commitment_tx_with_six_outputs_untrimmed_minimum_feerate() {
        // name: commitment tx with six outputs untrimmed (minimum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8006d007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de8431104e9d6a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400483045022100a2270d5950c89ae0841233f6efea9c951898b301b2e89e0adbd2c687b9f32efa02207943d90f95b9610458e7c65a576e149750ff3accaacad004cd85e70b235e27de01473044022072714e2fbb93cdd1c42eb0828b4f2eff143f717d8f26e79d6ada4f0dcb681bbe02200911be4e5161dd6ebe59ff1c58e1997c4aea804f81db6b698821db6093d7b05701475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(648);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }

    #[test]
    fn test_commitment_tx_with_six_outputs_untrimmed_maximum_feerate() {
        // name: commitment tx with six outputs untrimmed (maximum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8006d007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311077956a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402203ca8f31c6a47519f83255dc69f1894d9a6d7476a19f498d31eaf0cd3a85eeb63022026fd92dc752b33905c4c838c528b692a8ad4ced959990b5d5ee2ff940fa90eea01473044022001d55e488b8b035b2dd29d50b65b530923a416d47f377284145bc8767b1b6a75022019bb53ddfe1cefaf156f924777eaaf8fdca1810695a7d0a247ad2afba8232eb401475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(2069);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }

    #[test]
    fn test_commitment_tx_with_five_outputs_untrimmed_minimum_feerate() {
        //name: commitment tx with five outputs untrimmed (minimum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8005d007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5b80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110da966a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e04004730440220443cb07f650aebbba14b8bc8d81e096712590f524c5991ac0ed3bbc8fd3bd0c7022028a635f548e3ca64b19b69b1ea00f05b22752f91daf0b6dab78e62ba52eb7fd001483045022100f2377f7a67b7fc7f4e2c0c9e3a7de935c32417f5668eda31ea1db401b7dc53030220415fdbc8e91d0f735e70c21952342742e25249b0d062d43efbfc564499f3752601475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(2070);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }

    #[test]
    fn test_commitment_tx_with_five_outputs_untrimmed_maximum_feerate() {
        // name: commitment tx with five outputs untrimmed (maximum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8005d007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5b80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311040966a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402203b1b010c109c2ecbe7feb2d259b9c4126bd5dc99ee693c422ec0a5781fe161ba0220571fe4e2c649dea9c7aaf7e49b382962f6a3494963c97d80fef9a430ca3f706101483045022100d33c4e541aa1d255d41ea9a3b443b3b822ad8f7f86862638aac1f69f8f760577022007e2a18e6931ce3d3a804b1c78eda1de17dbe1fb7a95488c9a4ec8620395334801475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(2194);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }

    #[test]
    fn test_commitment_tx_with_four_outputs_untrimmed_minimum_feerate() {
        // name: commitment tx with four outputs untrimmed (minimum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8004b80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110b8976a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402203b12d44254244b8ff3bb4129b0920fd45120ab42f553d9976394b099d500c99e02205e95bb7a3164852ef0c48f9e0eaf145218f8e2c41251b231f03cbdc4f29a54290147304402205e2f76d4657fb732c0dfc820a18a7301e368f5799e06b7828007633741bda6df0220458009ae59d0c6246065c419359e05eb2a4b4ef4a1b310cc912db44eb792429801475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(2195);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }

    #[test]
    fn test_commitment_tx_with_four_outputs_untrimmed_maximum_feerate() {
        //name: commitment tx with four outputs untrimmed (maximum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8004b80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de8431106f916a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402200e930a43c7951162dc15a2b7344f48091c74c70f7024e7116e900d8bcfba861c022066fa6cbda3929e21daa2e7e16a4b948db7e8919ef978402360d1095ffdaff7b001483045022100c1a3b0b60ca092ed5080121f26a74a20cec6bdee3f8e47bae973fcdceb3eda5502207d467a9873c939bf3aa758014ae67295fedbca52412633f7e5b2670fc7c381c101475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(3702);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }

    #[test]
    fn test_commitment_tx_with_three_outputs_untrimmed_minimum_feerate() {
        //name: commitment tx with three outputs untrimmed (minimum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8003a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110eb936a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400473044022047305531dd44391dce03ae20f8735005c615eb077a974edb0059ea1a311857d602202e0ed6972fbdd1e8cb542b06e0929bc41b2ddf236e04cb75edd56151f4197506014830450221008b7c191dd46893b67b628e618d2dc8e81169d38bade310181ab77d7c94c6675e02203b4dd131fd7c9deb299560983dcdc485545c98f989f7ae8180c28289f9e6bdb001475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(3703);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }

    #[test]
    fn test_commitment_tx_with_three_outputs_untrimmed_maximum_feerate() {
        //name: commitment tx with three outputs untrimmed (maximum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8003a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110ae8f6a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402206a2679efa3c7aaffd2a447fd0df7aba8792858b589750f6a1203f9259173198a022008d52a0e77a99ab533c36206cb15ad7aeb2aa72b93d4b571e728cb5ec2f6fe260147304402206d6cb93969d39177a09d5d45b583f34966195b77c7e585cf47ac5cce0c90cefb022031d71ae4e33a4e80df7f981d696fbdee517337806a3c7138b7491e2cbb077a0e01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(4914);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }

    #[test]
    fn test_commitment_tx_with_two_outputs_untrimmed_minimum_feerate() {
        // name: commitment tx with two outputs untrimmed (minimum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8002c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110fa926a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400483045022100a012691ba6cea2f73fa8bac37750477e66363c6d28813b0bb6da77c8eb3fb0270220365e99c51304b0b1a6ab9ea1c8500db186693e39ec1ad5743ee231b0138384b90147304402200769ba89c7330dfa4feba447b6e322305f12ac7dac70ec6ba997ed7c1b598d0802204fe8d337e7fee781f9b7b1a06e580b22f4f79d740059560191d7db53f876555201475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(4915);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }

    #[test]
    fn test_commitment_tx_with_two_outputs_untrimmed_maximum_feerate() {
        // name: commitment tx with two outputs untrimmed (maximum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b800222020000000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80ec0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311004004730440220514f977bf7edc442de8ce43ace9686e5ebdc0f893033f13e40fb46c8b8c6e1f90220188006227d175f5c35da0b092c57bea82537aed89f7778204dc5bacf4f29f2b901473044022037f83ff00c8e5fb18ae1f918ffc24e54581775a20ff1ae719297ef066c71caa9022039c529cccd89ff6c5ed1db799614533844bd6d101da503761c45c713996e3bbd01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(9651180);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }

    #[test]
    fn test_commitment_tx_with_one_output_untrimmed_minimum_feerate() {
        // name: commitment tx with one output untrimmed (minimum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8001c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de8431100400473044022031a82b51bd014915fe68928d1abf4b9885353fb896cac10c3fdd88d7f9c7f2e00220716bda819641d2c63e65d3549b6120112e1aeaf1742eed94a471488e79e206b101473044022064901950be922e62cbe3f2ab93de2b99f37cff9fc473e73e394b27f88ef0731d02206d1dfa227527b4df44a07599289e207d6fd9cca60c0365682dcd3deaf739567e01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(9651181);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }

    #[test]
    fn test_commitment_tx_with_fee_greater_than_funder_amount() {
        // name: commitment tx with fee greater than funder amount
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8001c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de8431100400473044022031a82b51bd014915fe68928d1abf4b9885353fb896cac10c3fdd88d7f9c7f2e00220716bda819641d2c63e65d3549b6120112e1aeaf1742eed94a471488e79e206b101473044022064901950be922e62cbe3f2ab93de2b99f37cff9fc473e73e394b27f88ef0731d02206d1dfa227527b4df44a07599289e207d6fd9cca60c0365682dcd3deaf739567e01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
        let commit_tx = get_base_commit_tx(9651936);

        let tx = commit_tx.get_tx();
        assert_tx_eq(&tx, &example_tx, true);
    }
}