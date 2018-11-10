#![forbid(unsafe_code)]

extern crate secp256k1;

extern crate wire;
extern crate common_types;

use secp256k1::{PublicKey, SecretKey, Error as EcdsaError};

// TODO: move to common-types
use wire::Satoshi;

use common_types::Hash256;

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

/// generate shared secrets by the given `payment_path` and `session_key`
pub fn generate_shared_secrets(
    payment_path: &[PublicKey],
    session_key: &SecretKey,
) -> Result<Vec<Hash256>, EcdsaError> {
    use secp256k1::Secp256k1;

    let context = Secp256k1::new();

    // functions
    // `mul_pk` and `mul_sk` obviously performs the multiplication in the elliptic curve group
    // `hash` or `hash_s` computes a sha256 hash from a given slice or slices
    // `hash_to_sk` obviously casts a sha256 hash into a secret key
    let mul_pk = |x: &PublicKey, sk: &SecretKey| {
        let mut temp = x.clone();
        temp.mul_assign(&context, sk).map(|()| temp)
    };
    let mul_sk = |x: &SecretKey, sk: &SecretKey| {
        let mut temp: SecretKey = x.clone();
        temp.mul_assign(&context, sk).map(|()| temp)
    };
    let hash = |x: &[u8]| -> Hash256 { Hash256::from(x) };
    let hash_s = |xs: &[&[u8]]| -> Hash256 { Hash256::from(xs) };
    let hash_to_sk = |hash: &Hash256| SecretKey::from_slice(&context, hash.as_ref());

    // hardcoded public key
    let base_point = {
        let s = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
        PublicKey::from_slice(&context, hex::decode(s).unwrap().as_slice()).unwrap()
    };

    let initial = (
        Vec::with_capacity(payment_path.len()),
        session_key.clone(),
        PublicKey::from_secret_key(&context, session_key)?,
        Hash256::from([0; 32]),
    );

    payment_path
        .iter()
        .try_fold(initial, |(mut v, secret, public, blinding), path_point| {
            let temp = mul_pk(&path_point, &secret)?;
            let result = hash(&temp.serialize()[..]);
            let secret = mul_sk(&secret, &hash_to_sk(&blinding)?)?;
            let blinding = hash_s(&[&public.serialize()[..], result.as_ref()][..]);
            let public = mul_pk(&base_point, &secret)?;

            v.push(result);
            Ok((v, secret, public, blinding))
        }).map(|(v, _, _, _)| v)
}
