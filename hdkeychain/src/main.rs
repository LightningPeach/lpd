extern crate bitcoin;
extern crate secp256k1;
extern crate hex;

use bitcoin::util::bip32::ExtendedPrivKey;
use bitcoin::network::constants::Network;
use secp256k1::Secp256k1;

fn main() {
    let seed_hex = "000102030405060708090a0b0c0d0e0f";
    let seed_bytes = hex::decode(seed_hex).unwrap();

    let master_key = ExtendedPrivKey::new_master(&Secp256k1::new(), Network::Bitcoin, &seed_bytes).unwrap();
    println!("{}", master_key.to_string())
}
