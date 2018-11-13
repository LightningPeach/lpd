#![forbid(unsafe_code)]

extern crate secp256k1;

extern crate wire;
extern crate common_types;
extern crate hmac;
extern crate chacha;
extern crate sha2;
extern crate serde;
extern crate serde_derive;

mod hop;
mod route;

pub use self::route::{OnionPacketVersion, OnionRoute, OnionPacket};
pub use self::hop::{Hop, HopData, HopDataRealm};

use secp256k1::{PublicKey, SecretKey, Error as EcdsaError};

use common_types::Hash256;

use serde_derive::{Serialize, Deserialize};

/// `HMAC_SIZE` is the length of the HMACs used to verify the integrity of
/// the onion. Any value lower than 32 will truncate the HMAC both
/// during onion creation as well as during the verification.
pub const HMAC_SIZE: usize = 32;

/// `NUM_MAX_HOPS` is the the maximum path length. This should be set to an
/// estimate of the upper limit of the diameter of the node graph.
pub const NUM_MAX_HOPS: usize = 20;

#[derive(Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct HmacData {
    data: [u8; HMAC_SIZE],
}

fn generate_shared_secrets(
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

    // secp256k1 base point G
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
