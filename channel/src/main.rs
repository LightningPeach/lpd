extern crate bitcoin;
extern crate hex;
extern crate secp256k1;
extern crate crypto;

extern crate channel;

use bitcoin::blockdata::transaction::{Transaction, TxIn, TxOut};
use bitcoin::util::hash::{Sha256dHash, Hash160, Ripemd160Hash};
use bitcoin::blockdata::script::{Script, Builder};
use bitcoin::blockdata::opcodes::All::*;

use bitcoin::network::encodable::{ConsensusEncodable, ConsensusDecodable};
use bitcoin::network::serialize::{RawEncoder, RawDecoder};

use bitcoin::util::bip143;
use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};

use crypto::sha2::Sha256;
use crypto::digest::Digest;

pub const OP_CHECKSEQUENCEVERIFY: bitcoin::blockdata::opcodes::All = OP_NOP3;
pub const OP_CHECKLOCKTIMEVERIFY: bitcoin::blockdata::opcodes::All = OP_NOP2;

use channel::tools::{offered_htlc, sha256, accepted_htlc, assert_tx_eq, to_local_script, get_obscuring_number, get_sequence, get_locktime, s2dh256, s2byte32, s2pubkey, s2privkey, new_2x2_multisig, new_2x2_wsh_lock_script, v0_p2wpkh, p2pkh, p2pkh_unlock_script, s2script, s2tx};
use channel::bip69;
use channel::spec_example::get_example;
use channel::commit::CommitTx;

fn example_from_spec() {
    // Try to recreate example from specification:
    let local_pk = hex::decode("023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb").unwrap();
    let remote_pk = hex::decode("030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c1").unwrap();
    let funding_ws = new_2x2_multisig(&local_pk, &remote_pk);
    assert_eq!(hex::encode(funding_ws.data()), "5221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae");

    // Funding lockscript should be "0020c015c4a6be010e21657068fc2e6a9d02b27ebe4d490a25846f7237f104d1a3cd"
    let funding_lock_script = new_2x2_wsh_lock_script(&local_pk, &remote_pk);
    assert_eq!(hex::encode(funding_lock_script.data()), "0020c015c4a6be010e21657068fc2e6a9d02b27ebe4d490a25846f7237f104d1a3cd");

    let privkey_bytes = hex::decode("6bd078650fcee8444e4e09825227b801a1ca928debb750eb36e6d56124bb20e801").unwrap();
    assert_eq!(privkey_bytes.len(), 33);
    assert_eq!(privkey_bytes[32], 1);

    let sec = Secp256k1::new();
    let sk = SecretKey::from_slice(&sec, &privkey_bytes[0..32]).unwrap();
    // Pubkey (in compresses format) should have value: 03535b32d5eb0a6ed0982a0479bbadc9868d9836f6ba94dd5a63be16d875069184
    // This value was extracted from example transaction
    let pk = PublicKey::from_secret_key(&sec, &sk).unwrap();
    assert_eq!(hex::encode(&pk.serialize()[..]), "03535b32d5eb0a6ed0982a0479bbadc9868d9836f6ba94dd5a63be16d875069184");

    // Hash160 of compressed pubkey should be 3ca33c2e4446f4a305f23c80df8ad1afdcf652f9
    let pk_hash160 = Hash160::from_data(&pk.serialize()[..]).data();
    assert_eq!(hex::encode(&pk_hash160[..]), "3ca33c2e4446f4a305f23c80df8ad1afdcf652f9");

    // Change output script should be 00143ca33c2e4446f4a305f23c80df8ad1afdcf652f9
    let sc_change = v0_p2wpkh(&pk);
    assert_eq!(hex::encode(&sc_change.data()), "00143ca33c2e4446f4a305f23c80df8ad1afdcf652f9");

    let tx_out_funding = TxOut{
        value: 10_000_000,
        script_pubkey: funding_lock_script,
    };

    let tx_out_change = TxOut {
        value: 49_89_986_080,
        script_pubkey: sc_change,
    };

    let tx_in = TxIn {
        prev_hash: s2dh256("fd2105607605d2302994ffea703b09f66b6351816ee737a93e42a841ea20bbad"),
        prev_index: 0,
        script_sig: Script::new(),
        sequence: 4294967295,
        witness: vec![]
    };

    let mut tx = Transaction {
        version: 2,
        input: vec![tx_in],
        output: vec![tx_out_funding, tx_out_change],
        lock_time: 0
    };

    // Signature of transaction should be
    // 304502210090587b6201e166ad6af0227d3036a9454223d49a1f11839c1a362184340ef0240220577f7cd5cca78719405cbf1de7414ac027f0239ef6e214c90fcaab0454d84b3b[ALL]
    // We use deterministic signatures so it should be reproducible
    let sig_type = 1_u8; // SIGHASH_ALL
    let tx_sig_hash = tx.signature_hash(0, &p2pkh(&pk), sig_type as u32);
    let sig = sec.sign(&Message::from(tx_sig_hash.data()), &sk).unwrap();
    let mut sig_serialised = sig.serialize_der(&sec);
    assert_eq!(hex::encode(&sig_serialised), "304502210090587b6201e166ad6af0227d3036a9454223d49a1f11839c1a362184340ef0240220577f7cd5cca78719405cbf1de7414ac027f0239ef6e214c90fcaab0454d84b3b");
    // We need to add sigtype to the end of the signature
    sig_serialised.push(sig_type);
    tx.input[0].script_sig = p2pkh_unlock_script(&pk, &sig_serialised);

    // Hash of funding transaction should be 8984484a580b825b9972d7adb15050b3ab624ccd731946b3eeddb92f4e7ef6be
    assert_eq!(tx.txid().be_hex_string(), "8984484a580b825b9972d7adb15050b3ab624ccd731946b3eeddb92f4e7ef6be");
    // Transaction should be 0200000001adbb20ea41a8423ea937e76e8151636bf6093b70eaff942930d20576600521fd000000006b48304502210090587b6201e166ad6af0227d3036a9454223d49a1f11839c1a362184340ef0240220577f7cd5cca78719405cbf1de7414ac027f0239ef6e214c90fcaab0454d84b3b012103535b32d5eb0a6ed0982a0479bbadc9868d9836f6ba94dd5a63be16d875069184ffffffff028096980000000000220020c015c4a6be010e21657068fc2e6a9d02b27ebe4d490a25846f7237f104d1a3cd20256d29010000001600143ca33c2e4446f4a305f23c80df8ad1afdcf652f900000000
    let mut a = vec![];
    tx.consensus_encode(&mut RawEncoder::new(&mut a)).unwrap();
    assert_eq!(hex::encode(a), "0200000001adbb20ea41a8423ea937e76e8151636bf6093b70eaff942930d20576600521fd000000006b48304502210090587b6201e166ad6af0227d3036a9454223d49a1f11839c1a362184340ef0240220577f7cd5cca78719405cbf1de7414ac027f0239ef6e214c90fcaab0454d84b3b012103535b32d5eb0a6ed0982a0479bbadc9868d9836f6ba94dd5a63be16d875069184ffffffff028096980000000000220020c015c4a6be010e21657068fc2e6a9d02b27ebe4d490a25846f7237f104d1a3cd20256d29010000001600143ca33c2e4446f4a305f23c80df8ad1afdcf652f900000000");
}

fn spec_ex_1() {
    let ex = get_example();

    // This transaction should be obtained
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8002c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311054a56a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400473044022051b75c73198c6deee1a875871c3961832909acd297c6b908d59e3319e5185a46022055c419379c5051a78d00dbbce11b5b664a0c22815fbcc6fcef6b1937c383693901483045022100f51d2e566a70ba740fc5d8c0f07b9b93d2ed741c3c0860c613173de7d39e7968022041376d520e9c0e1ad52248ddf4b22e12be8763007df977253ef45a4ca3bdb7c001475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    // Transaction consists of:
    // 1. Version: 2
    // 2. Inputs:
    // 3. Outputs:
    // 4. Locktime:

    let obscuring_number = get_obscuring_number(&ex.local_payment_basepoint.serialize(), &ex.remote_payment_basepoint.serialize());
    assert_eq!(obscuring_number, ex.obscuring_factor);

    let remote_funding_pk = PublicKey::from_secret_key(&Secp256k1::new(), &ex.internal.remote_funding_privkey).unwrap();
    assert_eq!(remote_funding_pk, ex.remote_funding_pubkey);

    let obscured_commit_number = ex.commitment_number ^ obscuring_number;
    assert_eq!(get_sequence(obscured_commit_number), 2150346808);
    assert_eq!(get_locktime(obscured_commit_number), 542251326);

    let sequence = get_sequence(obscured_commit_number);
    let locktime = get_locktime(obscured_commit_number);

    let to_local_script_ex = s2script("63210212a140cd0c6539d07cd08dfe09984dec3251ea808b892efeac3ede9402bf2b1967029000b2752103fd5960528dc152014952efdb702a88f71e3c1653b2314431701ec77e57fde83c68ac");
    let to_local = to_local_script(&ex.local_delayedpubkey, ex.local_delay as u64, &ex.local_revocation_pubkey);
    assert_eq!(to_local, to_local_script_ex);
    let to_local_lock_script = to_local.to_v0_p2wsh();
    assert_eq!(to_local_lock_script, s2script("00204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e"));

    let to_remote_lock_script = v0_p2wpkh(&ex.remotepubkey);
    assert_eq!(to_remote_lock_script, s2script("0014ccf1af2f2aabee14bb40fa3851ab2301de843110"));

    let funding_lock_script = new_2x2_multisig(&ex.local_funding_pubkey.serialize(), &ex.remote_funding_pubkey.serialize());
    assert_eq!(funding_lock_script, s2script("5221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae"));

    let mut tx = Transaction{
        version: 2,
        input: vec![TxIn{
            prev_hash: ex.funding_tx_id,
            prev_index: ex.funding_output_index as u32,
            sequence: sequence as u32,
            script_sig: Script::new(),
            witness: vec![]
        }],
        output: vec![
            TxOut{
                value: 3000000,
                script_pubkey: to_remote_lock_script
            },
            TxOut{
                value: 6989140,
                script_pubkey: to_local_lock_script
            }
        ],
        lock_time: locktime as u32
    };

    // Check that all fields except witness is correct
    assert_tx_eq(&tx, &example_tx, true);

    let sec = Secp256k1::new();
    let tx_sig_hash = bip143::SighashComponents::new(&tx).sighash_all(&tx.input[0], &funding_lock_script, ex.funding_amount_satoshi as u64);
    let sig_local = sec.sign(&Message::from(tx_sig_hash.data()), &ex.local_funding_privkey).unwrap();
    let mut sig_local_serialised = sig_local.serialize_der(&sec);
    assert_eq!(hex::encode(&sig_local_serialised), "3044022051b75c73198c6deee1a875871c3961832909acd297c6b908d59e3319e5185a46022055c419379c5051a78d00dbbce11b5b664a0c22815fbcc6fcef6b1937c3836939");
    sig_local_serialised.push(1);

    let sig_remote = sec.sign(&Message::from(tx_sig_hash.data()), &ex.internal.remote_funding_privkey).unwrap();
    let mut sig_remote_serialised = sig_remote.serialize_der(&sec);
    assert_eq!(hex::encode(&sig_remote_serialised), "3045022100f51d2e566a70ba740fc5d8c0f07b9b93d2ed741c3c0860c613173de7d39e7968022041376d520e9c0e1ad52248ddf4b22e12be8763007df977253ef45a4ca3bdb7c0");
    sig_remote_serialised.push(1);

    tx.input[0].witness = vec![
        vec![],
        sig_local_serialised,
        sig_remote_serialised,
        funding_lock_script.data()
    ];
    assert_tx_eq(&tx, &example_tx, false);

    let mut a = vec![];
    tx.consensus_encode(&mut RawEncoder::new(&mut a)).unwrap();
    assert_eq!(hex::encode(a), "02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8002c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311054a56a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400473044022051b75c73198c6deee1a875871c3961832909acd297c6b908d59e3319e5185a46022055c419379c5051a78d00dbbce11b5b664a0c22815fbcc6fcef6b1937c383693901483045022100f51d2e566a70ba740fc5d8c0f07b9b93d2ed741c3c0860c613173de7d39e7968022041376d520e9c0e1ad52248ddf4b22e12be8763007df977253ef45a4ca3bdb7c001475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
}

fn spec_ex_2() {
    let ex = get_example();

    // name: commitment tx with all five HTLCs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8007e80300000000000022002052bfef0479d7b293c27e0f1eb294bea154c63a3294ef092c19af51409bce0e2ad007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110e0a06a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e04004730440220275b0c325a5e9355650dc30c0eccfbc7efb23987c24b556b9dfdd40effca18d202206caceb2c067836c51f296740c7ae807ffcbfbf1dd3a0d56b6de9a5b247985f060147304402204fd4928835db1ccdfc40f5c78ce9bd65249b16348df81f0c44328dcdefc97d630220194d3869c38bc732dd87d13d2958015e2fc16829e74cd4377f84d215c0b7060601475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

//    Transaction outputs:
//    (HTLC 0)
//    (HTLC 2)
//    (HTLC 1)
//    (HTLC 3)
//    (HTLC 4)
//    To remote
//    To local

    let obscuring_number = get_obscuring_number(&ex.local_payment_basepoint.serialize(), &ex.remote_payment_basepoint.serialize());
    assert_eq!(obscuring_number, ex.obscuring_factor);
    let obscured_commit_number = ex.commitment_number ^ obscuring_number;
    assert_eq!(get_sequence(obscured_commit_number), 2150346808);
    assert_eq!(get_locktime(obscured_commit_number), 542251326);

    let sequence = get_sequence(obscured_commit_number);
    let locktime = get_locktime(obscured_commit_number);

    // Try to recreate output 0
    assert_eq!(hex::encode(Hash160::from_data(&(ex.local_revocation_pubkey.serialize())).data()), "14011f7254d96b819c76986c277d115efce6f7b5");
    assert_eq!(hex::encode(Hash160::from_data(&ex.htlcs[0].payment_preimage).data()), "b8bcb07f6344b42ab04250c86a6e8b75d3fdbbc6");

    // It seems like it is remotepubkey
    let remote_htlc_pubkey = s2pubkey("0394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b");
    // It seems like it is localpubkey
    let local_htlc_pubkey = s2pubkey("030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e7");

    let htlc0_script = accepted_htlc(&ex.local_revocation_pubkey, &remote_htlc_pubkey, &local_htlc_pubkey, sha256(&ex.htlcs[0].payment_preimage), ex.htlcs[0].expiry as u32);
    assert_eq!(htlc0_script, s2script("76a91414011f7254d96b819c76986c277d115efce6f7b58763ac67210394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b7c8201208763a914b8bcb07f6344b42ab04250c86a6e8b75d3fdbbc688527c21030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e752ae677502f401b175ac6868"));

    let htlc2_script = offered_htlc(&ex.local_revocation_pubkey, &remote_htlc_pubkey, &local_htlc_pubkey, sha256(&ex.htlcs[2].payment_preimage));
    assert_eq!(htlc2_script, s2script("76a91414011f7254d96b819c76986c277d115efce6f7b58763ac67210394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b7c820120876475527c21030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e752ae67a914b43e1b38138a41b37f7cd9a1d274bc63e3a9b5d188ac6868"));

    let htlc1_script = accepted_htlc(&ex.local_revocation_pubkey, &remote_htlc_pubkey, &local_htlc_pubkey, sha256(&ex.htlcs[1].payment_preimage), ex.htlcs[1].expiry as u32);
    assert_eq!(htlc1_script, s2script("76a91414011f7254d96b819c76986c277d115efce6f7b58763ac67210394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b7c8201208763a9144b6b2e5444c2639cc0fb7bcea5afba3f3cdce23988527c21030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e752ae677502f501b175ac6868"));

    let htlc3_script = offered_htlc(&ex.local_revocation_pubkey, &remote_htlc_pubkey, &local_htlc_pubkey, sha256(&ex.htlcs[3].payment_preimage));
    assert_eq!(htlc3_script, s2script("76a91414011f7254d96b819c76986c277d115efce6f7b58763ac67210394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b7c820120876475527c21030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e752ae67a9148a486ff2e31d6158bf39e2608864d63fefd09d5b88ac6868"));

    let htlc4_script = accepted_htlc(&ex.local_revocation_pubkey, &remote_htlc_pubkey, &local_htlc_pubkey, sha256(&ex.htlcs[4].payment_preimage), ex.htlcs[4].expiry as u32);
    assert_eq!(htlc4_script, s2script("76a91414011f7254d96b819c76986c277d115efce6f7b58763ac67210394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b7c8201208763a91418bc1a114ccf9c052d3d23e28d3b0a9d1227434288527c21030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e752ae677502f801b175ac6868"));

    let to_local = to_local_script(&ex.local_delayedpubkey, ex.local_delay as u64, &ex.local_revocation_pubkey);
    let to_local_lock_script = to_local.to_v0_p2wsh();
    assert_eq!(to_local_lock_script, s2script("00204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e"));

    let to_remote_lock_script = v0_p2wpkh(&ex.remotepubkey);
    assert_eq!(to_remote_lock_script, s2script("0014ccf1af2f2aabee14bb40fa3851ab2301de843110"));

    let funding_lock_script = new_2x2_multisig(&ex.local_funding_pubkey.serialize(), &ex.remote_funding_pubkey.serialize());
    assert_eq!(funding_lock_script, s2script("5221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae"));

    let mut tx = Transaction{
        version: 2,
        input: vec![TxIn{
            prev_hash: ex.funding_tx_id,
            prev_index: ex.funding_output_index as u32,
            sequence: sequence as u32,
            script_sig: Script::new(),
            witness: vec![]
        }],
        output: vec![
            TxOut{
                value: (ex.htlcs[0].amount_msat / 1000) as u64,
                script_pubkey: htlc0_script.to_v0_p2wsh()
            },
            TxOut{
                value: (ex.htlcs[2].amount_msat / 1000) as u64,
                script_pubkey: htlc2_script.to_v0_p2wsh()
            },
            TxOut{
                value: (ex.htlcs[1].amount_msat / 1000) as u64,
                script_pubkey: htlc1_script.to_v0_p2wsh()
            },
            TxOut{
                value: (ex.htlcs[3].amount_msat / 1000) as u64,
                script_pubkey: htlc3_script.to_v0_p2wsh()
            },
            TxOut{
                value: (ex.htlcs[4].amount_msat / 1000) as u64,
                script_pubkey: htlc4_script.to_v0_p2wsh()
            },
            TxOut{
                value: 3000000,
                script_pubkey: to_remote_lock_script
            },
            TxOut{
                value: 6988000,
                script_pubkey: to_local_lock_script
            }
        ],
        lock_time: locktime as u32
    };

    println!("{:?}", bip69::TransactionReordering::from_tx_to_bip69(&tx));
    // Check that all fields except witness is correct
    assert_tx_eq(&tx, &example_tx, true);

    let sec = Secp256k1::new();
    let tx_sig_hash = bip143::SighashComponents::new(&tx).sighash_all(&tx.input[0], &funding_lock_script, ex.funding_amount_satoshi as u64);
    let sig_local = sec.sign(&Message::from(tx_sig_hash.data()), &ex.local_funding_privkey).unwrap();
    let mut sig_local_serialised = sig_local.serialize_der(&sec);
    assert_eq!(hex::encode(&sig_local_serialised), "30440220275b0c325a5e9355650dc30c0eccfbc7efb23987c24b556b9dfdd40effca18d202206caceb2c067836c51f296740c7ae807ffcbfbf1dd3a0d56b6de9a5b247985f06");
    sig_local_serialised.push(1);

    let sig_remote = sec.sign(&Message::from(tx_sig_hash.data()), &ex.internal.remote_funding_privkey).unwrap();
    let mut sig_remote_serialised = sig_remote.serialize_der(&sec);
    assert_eq!(hex::encode(&sig_remote_serialised), "304402204fd4928835db1ccdfc40f5c78ce9bd65249b16348df81f0c44328dcdefc97d630220194d3869c38bc732dd87d13d2958015e2fc16829e74cd4377f84d215c0b70606");
    sig_remote_serialised.push(1);

    tx.input[0].witness = vec![
        vec![],
        sig_local_serialised,
        sig_remote_serialised,
        funding_lock_script.data()
    ];
    assert_tx_eq(&tx, &example_tx, false);

    let mut a = vec![];
    tx.consensus_encode(&mut RawEncoder::new(&mut a)).unwrap();
    assert_eq!(hex::encode(a), "02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8007e80300000000000022002052bfef0479d7b293c27e0f1eb294bea154c63a3294ef092c19af51409bce0e2ad007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110e0a06a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e04004730440220275b0c325a5e9355650dc30c0eccfbc7efb23987c24b556b9dfdd40effca18d202206caceb2c067836c51f296740c7ae807ffcbfbf1dd3a0d56b6de9a5b247985f060147304402204fd4928835db1ccdfc40f5c78ce9bd65249b16348df81f0c44328dcdefc97d630220194d3869c38bc732dd87d13d2958015e2fc16829e74cd4377f84d215c0b7060601475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
}



fn spec_ex_1_1() {
    let ex = get_example();

    // name: simple commitment tx with no HTLCs
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8002c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311054a56a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400473044022051b75c73198c6deee1a875871c3961832909acd297c6b908d59e3319e5185a46022055c419379c5051a78d00dbbce11b5b664a0c22815fbcc6fcef6b1937c383693901483045022100f51d2e566a70ba740fc5d8c0f07b9b93d2ed741c3c0860c613173de7d39e7968022041376d520e9c0e1ad52248ddf4b22e12be8763007df977253ef45a4ca3bdb7c001475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_2_1() {
    let ex = get_example();

    // name: commitment tx with all five HTLCs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8007e80300000000000022002052bfef0479d7b293c27e0f1eb294bea154c63a3294ef092c19af51409bce0e2ad007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110e0a06a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e04004730440220275b0c325a5e9355650dc30c0eccfbc7efb23987c24b556b9dfdd40effca18d202206caceb2c067836c51f296740c7ae807ffcbfbf1dd3a0d56b6de9a5b247985f060147304402204fd4928835db1ccdfc40f5c78ce9bd65249b16348df81f0c44328dcdefc97d630220194d3869c38bc732dd87d13d2958015e2fc16829e74cd4377f84d215c0b7060601475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 0,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_3_1() {
    let ex = get_example();

    // name: commitment tx with seven outputs untrimmed (maximum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8007e80300000000000022002052bfef0479d7b293c27e0f1eb294bea154c63a3294ef092c19af51409bce0e2ad007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110e09c6a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040048304502210094bfd8f5572ac0157ec76a9551b6c5216a4538c07cd13a51af4a54cb26fa14320220768efce8ce6f4a5efac875142ff19237c011343670adf9c7ac69704a120d116301483045022100a5c01383d3ec646d97e40f44318d49def817fcd61a0ef18008a665b3e151785502203e648efddd5838981ef55ec954be69c4a652d021e6081a100d034de366815e9b01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 647,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_4_1() {
    let ex = get_example();

    // name: commitment tx with six outputs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8006d007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de8431104e9d6a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400483045022100a2270d5950c89ae0841233f6efea9c951898b301b2e89e0adbd2c687b9f32efa02207943d90f95b9610458e7c65a576e149750ff3accaacad004cd85e70b235e27de01473044022072714e2fbb93cdd1c42eb0828b4f2eff143f717d8f26e79d6ada4f0dcb681bbe02200911be4e5161dd6ebe59ff1c58e1997c4aea804f81db6b698821db6093d7b05701475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 648,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_5_1() {
    let ex = get_example();

    // name: commitment tx with six outputs untrimmed (maximum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8006d007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311077956a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402203ca8f31c6a47519f83255dc69f1894d9a6d7476a19f498d31eaf0cd3a85eeb63022026fd92dc752b33905c4c838c528b692a8ad4ced959990b5d5ee2ff940fa90eea01473044022001d55e488b8b035b2dd29d50b65b530923a416d47f377284145bc8767b1b6a75022019bb53ddfe1cefaf156f924777eaaf8fdca1810695a7d0a247ad2afba8232eb401475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 2069,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_6_1() {
    let ex = get_example();

    //name: commitment tx with five outputs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8005d007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5b80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110da966a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e04004730440220443cb07f650aebbba14b8bc8d81e096712590f524c5991ac0ed3bbc8fd3bd0c7022028a635f548e3ca64b19b69b1ea00f05b22752f91daf0b6dab78e62ba52eb7fd001483045022100f2377f7a67b7fc7f4e2c0c9e3a7de935c32417f5668eda31ea1db401b7dc53030220415fdbc8e91d0f735e70c21952342742e25249b0d062d43efbfc564499f3752601475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 2070,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_7_1() {
    let ex = get_example();

    // name: commitment tx with five outputs untrimmed (maximum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8005d007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5b80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311040966a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402203b1b010c109c2ecbe7feb2d259b9c4126bd5dc99ee693c422ec0a5781fe161ba0220571fe4e2c649dea9c7aaf7e49b382962f6a3494963c97d80fef9a430ca3f706101483045022100d33c4e541aa1d255d41ea9a3b443b3b822ad8f7f86862638aac1f69f8f760577022007e2a18e6931ce3d3a804b1c78eda1de17dbe1fb7a95488c9a4ec8620395334801475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 2194,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_8_1() {
    let ex = get_example();

    // name: commitment tx with four outputs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8004b80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110b8976a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402203b12d44254244b8ff3bb4129b0920fd45120ab42f553d9976394b099d500c99e02205e95bb7a3164852ef0c48f9e0eaf145218f8e2c41251b231f03cbdc4f29a54290147304402205e2f76d4657fb732c0dfc820a18a7301e368f5799e06b7828007633741bda6df0220458009ae59d0c6246065c419359e05eb2a4b4ef4a1b310cc912db44eb792429801475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 2195,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_9_1() {
    let ex = get_example();

    //name: commitment tx with four outputs untrimmed (maximum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8004b80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de8431106f916a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402200e930a43c7951162dc15a2b7344f48091c74c70f7024e7116e900d8bcfba861c022066fa6cbda3929e21daa2e7e16a4b948db7e8919ef978402360d1095ffdaff7b001483045022100c1a3b0b60ca092ed5080121f26a74a20cec6bdee3f8e47bae973fcdceb3eda5502207d467a9873c939bf3aa758014ae67295fedbca52412633f7e5b2670fc7c381c101475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 3702,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_10_1() {
    let ex = get_example();

    //name: commitment tx with three outputs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8003a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110eb936a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400473044022047305531dd44391dce03ae20f8735005c615eb077a974edb0059ea1a311857d602202e0ed6972fbdd1e8cb542b06e0929bc41b2ddf236e04cb75edd56151f4197506014830450221008b7c191dd46893b67b628e618d2dc8e81169d38bade310181ab77d7c94c6675e02203b4dd131fd7c9deb299560983dcdc485545c98f989f7ae8180c28289f9e6bdb001475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 3703,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_11_1() {
    let ex = get_example();

    //name: commitment tx with three outputs untrimmed (maximum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8003a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110ae8f6a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402206a2679efa3c7aaffd2a447fd0df7aba8792858b589750f6a1203f9259173198a022008d52a0e77a99ab533c36206cb15ad7aeb2aa72b93d4b571e728cb5ec2f6fe260147304402206d6cb93969d39177a09d5d45b583f34966195b77c7e585cf47ac5cce0c90cefb022031d71ae4e33a4e80df7f981d696fbdee517337806a3c7138b7491e2cbb077a0e01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 4914,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_12_1() {
    let ex = get_example();

    // name: commitment tx with two outputs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8002c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110fa926a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400483045022100a012691ba6cea2f73fa8bac37750477e66363c6d28813b0bb6da77c8eb3fb0270220365e99c51304b0b1a6ab9ea1c8500db186693e39ec1ad5743ee231b0138384b90147304402200769ba89c7330dfa4feba447b6e322305f12ac7dac70ec6ba997ed7c1b598d0802204fe8d337e7fee781f9b7b1a06e580b22f4f79d740059560191d7db53f876555201475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 4915,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_13_1() {
    let ex = get_example();

    // name: commitment tx with two outputs untrimmed (maximum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b800222020000000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80ec0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311004004730440220514f977bf7edc442de8ce43ace9686e5ebdc0f893033f13e40fb46c8b8c6e1f90220188006227d175f5c35da0b092c57bea82537aed89f7778204dc5bacf4f29f2b901473044022037f83ff00c8e5fb18ae1f918ffc24e54581775a20ff1ae719297ef066c71caa9022039c529cccd89ff6c5ed1db799614533844bd6d101da503761c45c713996e3bbd01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 9651180,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_14_1() {
    let ex = get_example();

    // name: commitment tx with one output untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8001c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de8431100400473044022031a82b51bd014915fe68928d1abf4b9885353fb896cac10c3fdd88d7f9c7f2e00220716bda819641d2c63e65d3549b6120112e1aeaf1742eed94a471488e79e206b101473044022064901950be922e62cbe3f2ab93de2b99f37cff9fc473e73e394b27f88ef0731d02206d1dfa227527b4df44a07599289e207d6fd9cca60c0365682dcd3deaf739567e01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 9651181,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_15_1() {
    let ex = get_example();

    // name: commitment tx with fee greater than funder amount
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8001c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de8431100400473044022031a82b51bd014915fe68928d1abf4b9885353fb896cac10c3fdd88d7f9c7f2e00220716bda819641d2c63e65d3549b6120112e1aeaf1742eed94a471488e79e206b101473044022064901950be922e62cbe3f2ab93de2b99f37cff9fc473e73e394b27f88ef0731d02206d1dfa227527b4df44a07599289e207d6fd9cca60c0365682dcd3deaf739567e01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 9651936,
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

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}


fn main() {
    println!("it works: create-funding-example");
    println!("Task is to create funding transaction and compare it hash to correct");
    // Known:
    // One funding input: 9fd6518132028825404a02cd02a1c4885317bf7e193674547f4b7cfec92d095b:1
    // One change output(index: 1): value:1240930000, lockScript: 0014177811f6ffea178e4fb96b4ed5e01a566cbd3fe2
    // Funding amount: 1000000
    // (Funder) Funding pubkey: 033df6f19a33d091d6196b20540de9ebfd8d704d8f05133add990eaffa946effbe
    // (Fundee) Funding pubkey: 03b6dc51a2d1b29e67a204656903a3b3ef42e45e3486eb8a506316acd36872422f
    // Funding output index:0
    // WE NEED TO OBTAIN: 47c632779ccfed3f3d36ba423eae871690e2d9270dde1df962b52a6c8be89336:0
    let pk1 = hex::decode("033df6f19a33d091d6196b20540de9ebfd8d704d8f05133add990eaffa946effbe").unwrap();
    let pk2 = hex::decode("03b6dc51a2d1b29e67a204656903a3b3ef42e45e3486eb8a506316acd36872422f").unwrap();

    let tx_in = TxIn {
        prev_hash: s2dh256("9fd6518132028825404a02cd02a1c4885317bf7e193674547f4b7cfec92d095b"),
        prev_index: 1,
        script_sig: Script::new(),
        sequence: 4294967295,
        witness: vec![vec![]]
    };

    let tx_out_funding = TxOut {
        value: 1000000,
        script_pubkey: new_2x2_wsh_lock_script(&pk1, &pk2),
    };

    let tx_out_change = TxOut {
        value: 1240930000,
        script_pubkey: s2script("0014177811f6ffea178e4fb96b4ed5e01a566cbd3fe2"),
    };
    assert_eq!(tx_out_funding.script_pubkey.data(), hex::decode("002085e247fa5331a24613863e86039d539cb21e012b966e014dd090f33eeb555760").unwrap());
    let tx = Transaction{
        version: 1,
        input: vec![tx_in],
        output: vec![tx_out_funding, tx_out_change],
        lock_time: 0
    };
    assert_eq!(tx.txid().be_hex_string(), "47c632779ccfed3f3d36ba423eae871690e2d9270dde1df962b52a6c8be89336");

    example_from_spec();

    spec_ex_1();
    spec_ex_2();
    spec_ex_1_1();
    spec_ex_2_1();
    spec_ex_3_1();
    spec_ex_4_1();
    spec_ex_5_1();
    spec_ex_6_1();
    spec_ex_7_1();
    spec_ex_8_1();
    spec_ex_9_1();
    spec_ex_10_1();
    spec_ex_11_1();
    spec_ex_12_1();
    spec_ex_13_1();
    spec_ex_14_1();
    spec_ex_15_1();
}