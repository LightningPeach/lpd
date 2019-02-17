use secp256k1::{SecretKey, PublicKey};
use bitcoin::util::hash::{Sha256dHash};
use super::commit::{HTLCDirection, HTLC};

use super::tools::{sha256, s2dh256, s2byte32, s2pubkey, s2privkey};

//+funding_tx_id: 8984484a580b825b9972d7adb15050b3ab624ccd731946b3eeddb92f4e7ef6be
//+funding_output_index: 0
//+funding_amount_satoshi: 10000000
//+commitment_number: 42
//+local_delay: 144
//+local_dust_limit_satoshi: 546
//htlc 0 direction: remote->local
//htlc 0 amount_msat: 1000000
//htlc 0 expiry: 500
//htlc 0 payment_preimage: 0000000000000000000000000000000000000000000000000000000000000000

//htlc 1 direction: remote->local
//htlc 1 amount_msat: 2000000
//htlc 1 expiry: 501
//htlc 1 payment_preimage: 0101010101010101010101010101010101010101010101010101010101010101

//htlc 2 direction: local->remote
//htlc 2 amount_msat: 2000000
//htlc 2 expiry: 502
//htlc 2 payment_preimage: 0202020202020202020202020202020202020202020202020202020202020202

//htlc 3 direction: local->remote
//htlc 3 amount_msat: 3000000
//htlc 3 expiry: 503
//htlc 3 payment_preimage: 0303030303030303030303030303030303030303030303030303030303030303

//htlc 4 direction: remote->local
//htlc 4 amount_msat: 4000000
//htlc 4 expiry: 504
//htlc 4 payment_preimage: 0404040404040404040404040404040404040404040404040404040404040404


//local_payment_basepoint: 034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aa
//remote_payment_basepoint: 032c0b7cf95324a07d05398b240174dc0c2be444d96b159aa6c7f7b1e668680991
//# obscured commitment transaction number = 0x2bb038521914 ^ 42

//local_funding_privkey: 30ff4956bbdd3222d44cc5e8a1261dab1e07957bdac5ae88fe3261ef321f374901
//local_funding_pubkey: 023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb
//remote_funding_pubkey: 030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c1
//local_privkey: bb13b121cdc357cd2e608b0aea294afca36e2b34cf958e2e6451a2f27469449101
//localpubkey: 030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e7
//remotepubkey: 0394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b
//local_delayedpubkey: 03fd5960528dc152014952efdb702a88f71e3c1653b2314431701ec77e57fde83c
//local_revocation_pubkey: 0212a140cd0c6539d07cd08dfe09984dec3251ea808b892efeac3ede9402bf2b19
//# funding wscript = 5221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae

//INTERNAL: remote_funding_privkey: 1552dfba4f6cf29a62a0af13c8d6981d36d0ef8d61ba10fb0fe90da7634d7e130101
//INTERNAL: local_payment_basepoint_secret: 111111111111111111111111111111111111111111111111111111111111111101
//INTERNAL: remote_revocation_basepoint_secret: 222222222222222222222222222222222222222222222222222222222222222201
//INTERNAL: local_delayed_payment_basepoint_secret: 333333333333333333333333333333333333333333333333333333333333333301
//INTERNAL: remote_payment_basepoint_secret: 444444444444444444444444444444444444444444444444444444444444444401
//x_local_per_commitment_secret: 1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a0908070605040302010001
//# From remote_revocation_basepoint_secret
//INTERNAL: remote_revocation_basepoint: 02466d7fcae563e5cb09a0d1870bb580344804617879a14949cf22285f1bae3f27
//# From local_delayed_payment_basepoint_secret
//INTERNAL: local_delayed_payment_basepoint: 023c72addb4fdf09af94f0c94d7fe92a386a7e70cf8a1d85916386bb2535c7b1b1
//INTERNAL: local_per_commitment_point: 025f7117a78150fe2ef97db7cfc83bd57b2e2c0d0dd25eaf467a4a1c2a45ce1486
//INTERNAL: remote_privkey: 8deba327a7cc6d638ab0eb025770400a6184afcba6713c210d8d10e199ff2fda01
//# From local_delayed_payment_basepoint_secret, local_per_commitment_point and local_delayed_payment_basepoint
//INTERNAL: local_delayed_privkey: adf3464ce9c2f230fd2582fda4c6965e4993ca5524e8c9580e3df0cf226981ad01

// Contains data internal to spec. Private keys for remote node, ...
pub struct Internal {
    pub remote_funding_privkey: SecretKey,
    pub local_payment_basepoint_secret: SecretKey,
    pub remote_revocation_basepoint_secret: SecretKey,
    pub local_delayed_payment_basepoint_secret: SecretKey,
    pub remote_payment_basepoint_secret: SecretKey,
    pub x_local_per_commitment_secret: SecretKey,
    pub remote_revocation_basepoint: PublicKey,
    pub local_delayed_payment_basepoint: PublicKey,
    pub local_per_commitment_point: PublicKey,
    pub remote_privkey: SecretKey,
    pub local_delayed_privkey: SecretKey,
}

pub struct HtlcExample {
    pub direction: String,
    pub amount_msat: i64,
    pub expiry: i32,
    pub payment_preimage: [u8; 32]
}

impl HtlcExample {
    pub fn to_htlc(&self) -> HTLC {
        let direction = match self.direction.as_ref() {
            "remote->local" => HTLCDirection::Accepted,
            "local->remote" => HTLCDirection::Offered,
            _ => panic!("unknown htlc direction in example data"),
        };
        let htlc = HTLC{
            direction: direction,
            amount_msat: self.amount_msat,
            payment_hash: sha256(&self.payment_preimage),
            expiry: self.expiry,
        };
        return htlc;
    }
}

pub struct SpecExample {
    pub funding_tx_id: Sha256dHash,
    pub funding_output_index: i32,
    pub funding_amount_satoshi: i64,
    pub commitment_number: u64,
    pub local_delay: i32,
    pub local_dust_limit_satoshi: i64,
    pub htlcs: Vec<HtlcExample>,
    pub local_payment_basepoint: PublicKey,
    pub remote_payment_basepoint: PublicKey,
    pub obscuring_factor: u64,
    pub local_funding_privkey: SecretKey,
    pub local_funding_pubkey: PublicKey,
    pub remote_funding_pubkey: PublicKey,
    pub local_privkey: SecretKey,
    pub localpubkey: PublicKey,
    pub remotepubkey: PublicKey,
    pub local_delayedpubkey: PublicKey,
    pub local_revocation_pubkey: PublicKey,
    pub internal: Internal
}

pub fn get_example() -> SpecExample {
    let ex = SpecExample {
        funding_tx_id: s2dh256("8984484a580b825b9972d7adb15050b3ab624ccd731946b3eeddb92f4e7ef6be"),
        funding_output_index: 0,
        funding_amount_satoshi: 10000000,
        commitment_number: 42,
        local_delay: 144,
        local_dust_limit_satoshi: 546,
        htlcs: vec![
            HtlcExample {
                direction: "remote->local".to_owned(),
                amount_msat: 1000000,
                expiry: 500,
                payment_preimage: s2byte32("0000000000000000000000000000000000000000000000000000000000000000")
            },
            HtlcExample {
                direction: "remote->local".to_owned(),
                amount_msat: 2000000,
                expiry: 501,
                payment_preimage: s2byte32("0101010101010101010101010101010101010101010101010101010101010101")
            },
            HtlcExample {
                direction: "local->remote".to_owned(),
                amount_msat: 2000000,
                expiry: 502,
                payment_preimage: s2byte32("0202020202020202020202020202020202020202020202020202020202020202")
            },
            HtlcExample {
                direction: "local->remote".to_owned(),
                amount_msat: 3000000,
                expiry: 503,
                payment_preimage: s2byte32("0303030303030303030303030303030303030303030303030303030303030303")
            },
            HtlcExample {
                direction: "remote->local".to_owned(),
                amount_msat: 4000000,
                expiry: 504,
                payment_preimage: s2byte32("0404040404040404040404040404040404040404040404040404040404040404")
            }
        ],
        local_payment_basepoint: s2pubkey("034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aa"),
        remote_payment_basepoint: s2pubkey("032c0b7cf95324a07d05398b240174dc0c2be444d96b159aa6c7f7b1e668680991"),
        obscuring_factor: 0x2bb038521914,
        local_funding_privkey: s2privkey("30ff4956bbdd3222d44cc5e8a1261dab1e07957bdac5ae88fe3261ef321f374901"),
        local_funding_pubkey: s2pubkey("023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb"),
        remote_funding_pubkey: s2pubkey("030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c1"),
        local_privkey: s2privkey("bb13b121cdc357cd2e608b0aea294afca36e2b34cf958e2e6451a2f27469449101"),
        localpubkey: s2pubkey("030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e7"),
        remotepubkey: s2pubkey("0394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b"),
        local_delayedpubkey: s2pubkey("03fd5960528dc152014952efdb702a88f71e3c1653b2314431701ec77e57fde83c"),
        local_revocation_pubkey: s2pubkey("0212a140cd0c6539d07cd08dfe09984dec3251ea808b892efeac3ede9402bf2b19"),
        internal: Internal{
            remote_funding_privkey: s2privkey("1552dfba4f6cf29a62a0af13c8d6981d36d0ef8d61ba10fb0fe90da7634d7e1301"),
            local_payment_basepoint_secret: s2privkey("111111111111111111111111111111111111111111111111111111111111111101"),
            remote_revocation_basepoint_secret: s2privkey("222222222222222222222222222222222222222222222222222222222222222201"),
            local_delayed_payment_basepoint_secret: s2privkey("333333333333333333333333333333333333333333333333333333333333333301"),
            remote_payment_basepoint_secret: s2privkey("444444444444444444444444444444444444444444444444444444444444444401"),
            x_local_per_commitment_secret: s2privkey("1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a0908070605040302010001"),
            remote_revocation_basepoint: s2pubkey("02466d7fcae563e5cb09a0d1870bb580344804617879a14949cf22285f1bae3f27"),
            local_delayed_payment_basepoint: s2pubkey("023c72addb4fdf09af94f0c94d7fe92a386a7e70cf8a1d85916386bb2535c7b1b1"),
            local_per_commitment_point: s2pubkey("025f7117a78150fe2ef97db7cfc83bd57b2e2c0d0dd25eaf467a4a1c2a45ce1486"),
            remote_privkey: s2privkey("8deba327a7cc6d638ab0eb025770400a6184afcba6713c210d8d10e199ff2fda01"),
            local_delayed_privkey: s2privkey("adf3464ce9c2f230fd2582fda4c6965e4993ca5524e8c9580e3df0cf226981ad01"),
        }
    };
    return ex;
}