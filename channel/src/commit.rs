use secp256k1::{PublicKey};
use bitcoin::util::hash::{Sha256dHash};
use bitcoin::blockdata::script::{Script};
use bitcoin::blockdata::transaction::{Transaction, TxIn, TxOut};
use bip69;
use tools::{get_sequence, get_locktime, accepted_htlc, offered_htlc, to_local_script, v0_p2wpkh};

pub const HTLC_TIMEOUT_WEIGHT: i64 = 663;
pub const HTLC_SUCCESS_WEIGHT: i64 = 703;
pub const BASE_COMMITMENT_WEIGHT: i64 = 724;
pub const PER_HTLC_COMMITMENT_WEIGHT: i64 = 172;

pub enum HTLCDirection {
    Accepted,
    Offered,
}

pub struct HTLC {
    pub direction: HTLCDirection,
    pub amount_msat: i64,
    pub expiry: i32,
    pub payment_hash: [u8; 32]
}

pub struct CommitTx {
    pub funding_amount: i64,

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
}