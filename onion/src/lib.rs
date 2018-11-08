#![forbid(unsafe_code)]

extern crate secp256k1;
extern crate sha2;
extern crate wire;

use secp256k1::{PublicKey, SecretKey, Error as EcdsaError};
use wire::Satoshi;

#[repr(u8)]
#[derive(Debug, Eq, PartialEq)]
pub enum OnionPacketVersion {
    _0 = 0,
}

/// `OnionPacket` is hop to hop routing information necessary to propagate a message
#[derive(Debug, Eq, PartialEq)]
pub struct OnionPacket {
    version: OnionPacketVersion,
    ephemeral_key: PublicKey,
    routing_info: Vec<HopData>,
    header_mac: [u8; 32],
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq)]
pub enum HopDataRealm {
    Bitcoin = 0,
}

#[derive(Debug, Eq, PartialEq)]
pub struct HopData {
    realm: HopDataRealm,
    next_address: [u8; 8],
    forward_amount: Satoshi,
    outgoing_cltv: u32,
    header_mac: [u8; 32],
}

fn generate_shared_secrets(payment_path: &[PublicKey], session_key: &SecretKey) -> Result<Vec<[u8; 32]>, EcdsaError> {
    use secp256k1::Secp256k1;

    let number = payment_path.len();
    let mut last_ephemeral_pub_key = PublicKey::from_secret_key(&Secp256k1::new(), session_key)?;

    unimplemented!()
}

// ecdh performs an ECDH operation between pk and sk. The returned value is
// the sha256 of the compressed shared point.
fn ecdh(pk: &PublicKey, sk: &SecretKey) -> Result<[u8; 32], EcdsaError> {
    use secp256k1::Secp256k1;

    let mut pk_cloned = pk.clone();
    pk_cloned.mul_assign(&Secp256k1::new(), sk)?;
    Ok(sha256(&pk_cloned.serialize()))
}

fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};

    let mut hasher = Sha256::default();
    hasher.input(data);
    let hash = hasher.result();

    let mut array: [u8; 32] = [0; 32];
    array.copy_from_slice(&hash);
    array
}
