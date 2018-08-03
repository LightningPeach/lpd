extern crate bitcoin;
extern crate secp256k1;
extern crate hex;

mod key_manager;
mod scoped_manager;
mod account_manager;

use bitcoin::util::bip32::ChildNumber;
use secp256k1::PublicKey;

use std::error::Error;

use key_manager::{KeyManager, KeyScope};

const BIP0084_PURPOSE: u32 = 84;

// COIN_TYPE_BITCOIN specifies the BIP44 coin type for Bitcoin key
// derivation.
const COIN_TYPE_BITCOIN: u32 = 0;

const BIP0084_KEY_SCOPE: KeyScope = KeyScope {
    purpose: ChildNumber::Hardened(BIP0084_PURPOSE),
    coin:    ChildNumber::Hardened(COIN_TYPE_BITCOIN),
};

const DEFAULT_ACCOUNT: ChildNumber = ChildNumber::Hardened(0);

struct HDWallet {
    key_manager: KeyManager,
}

impl HDWallet {
    fn from_seed(seed: &[u8]) -> Result<Self, Box<Error>> {
        Ok(Self {
            key_manager: KeyManager::from_seed(seed)?,
        })
    }

    /*
    fn new_public_key(&self, change: bool) -> Result<PublicKey, Box<Error>> {
        let key_scope = KeyScope {
            purpose: ChildNumber::Hardened(BIP0084_PURPOSE),
            coin:    ChildNumber::Hardened(COIN_TYPE_BITCOIN),
        };

        let mut branch = 0;
        if change {
            branch = 1;
        }

        let derivation_path = DerivationPath {
            account: ChildNumber::Hardened(0),
            branch:  ChildNumber::Normal(branch),
	        index:   ChildNumber::Normal(0),
        };

        let public_key = self.key_manager.derive_public_key_from_path(&key_scope, &derivation_path)?.public_key;
        Ok(public_key)
    }
    */
}

#[test]
fn test_bip0084() {
    let seed_hex = "5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4";
    let seed_bytes = hex::decode(seed_hex).unwrap();

    let wallet = HDWallet::from_seed(&seed_bytes).unwrap();
    let mut account_manager = wallet.key_manager
        .scoped_manager(&BIP0084_KEY_SCOPE).unwrap()
        .account_manager(DEFAULT_ACCOUNT).unwrap();

    let pk = account_manager.next_external_pk().unwrap();
    let pk_hex = hex::encode(&pk.serialize()[..]);
    assert_eq!(pk_hex, "0330d54fd0dd420a6e5f8d3624f5f3482cae350f79d5f0753bf5beef9c2d91af3c");

    let pk = account_manager.next_external_pk().unwrap();
    let pk_hex = hex::encode(&pk.serialize()[..]);
    assert_eq!(pk_hex, "03e775fd51f0dfb8cd865d9ff1cca2a158cf651fe997fdc9fee9c1d3b5e995ea77");

    let pk = account_manager.next_internal_pk().unwrap();
    let pk_hex = hex::encode(&pk.serialize()[..]);
    assert_eq!(pk_hex, "03025324888e429ab8e3dbaf1f7802648b9cd01e9b418485c5fa4c1b9b5700e1a6");
}

fn main() {
    println!("Hello, world!");
}