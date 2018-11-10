#![forbid(unsafe_code)]

extern crate secp256k1;

extern crate wire;
extern crate common_types;
extern crate hmac;
extern crate chacha;
extern crate sha2;
extern crate serde;
extern crate serde_derive;

use secp256k1::{PublicKey, SecretKey, Error as EcdsaError};

// TODO(vlad): move to common-types
use wire::Satoshi;

use common_types::Hash256;

use hmac::{Hmac, Mac};
use chacha::{ChaCha, KeyStream};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde_derive::{Serialize, Deserialize};

/// `HMAC_SIZE` is the length of the HMACs used to verify the integrity of
/// the onion. Any value lower than 32 will truncate the HMAC both
/// during onion creation as well as during the verification.
pub const HMAC_SIZE: usize = 32;

/// `ADDRESS_SIZE` is the length of the serialized address used to uniquely
/// identify the next hop to forward the onion to. BOLT 04 defines this
/// as 8 byte channel_id.
pub const ADDRESS_SIZE: usize = 8;

/// `NUM_MAX_HOPS` is the the maximum path length. This should be set to an
/// estimate of the upper limit of the diameter of the node graph.
pub const NUM_MAX_HOPS: usize = 20;

/// `PAD_SIZE` is the number of padding bytes in the hop_data. These bytes
/// are currently unused within the protocol, and are reserved for
/// future use.
pub const PAD_SIZE: usize = 12;

// `KEY_LEN` is the length of the keys used to generate cipher streams and
// encrypt payloads. Since we use SHA256 to generate the keys, the
// maximum length currently is 32 bytes.
const KEY_LEN: usize = 32;

// `HOP_DATA_SIZE` is the fixed size of hop_data. BOLT 04 currently
// specifies this to be 1 byte realm, 8 byte channel_id, 8 byte amount
// to forward, 4 byte outgoing CLTV value, 12 bytes padding and 32
// bytes HMAC for a total of 65 bytes per hop.
const HOP_DATA_SIZE: usize = 1 + ADDRESS_SIZE + 8 + 4 + PAD_SIZE + HMAC_SIZE;

// `ROUTING_INFO_SIZE` is the fixed size of the the routing info. This
// consists of a `ADDRESS_SIZE` byte address and a `HMAC_SIZE` byte HMAC for
// each hop of the route, the first pair in clear_text and the following
// pairs increasingly obfuscated. In case fewer than `NUM_MAX_HOPS` are
// used, then the remainder is padded with null-bytes, also obfuscated.
const ROUTING_INFO_SIZE: usize = NUM_MAX_HOPS * HOP_DATA_SIZE;

// `NUM_STREAM_BYTES` is the number of bytes produced by our CSPRG for the
// key stream implementing our stream cipher to encrypt/decrypt the mix
// header. The last `HOP_DATA_SIZE` bytes are only used in order to
// generate/check the MAC over the header.
const NUM_STREAM_BYTES: usize = ROUTING_INFO_SIZE + HOP_DATA_SIZE;

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum OnionPacketVersion {
    _0 = 0,
}

/// `OnionPacket` is hop to hop routing information necessary to propagate a message
#[derive(Debug, Eq, PartialEq)]
pub struct OnionPacket {
    version: OnionPacketVersion,
    ephemeral_key: PublicKey,
    routing_info: Vec<HopData>,
    header_mac: [u8; HMAC_SIZE],
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HopDataRealm {
    Bitcoin = 0,
}

#[derive(Debug, Eq, PartialEq)]
pub struct HopData {
    realm: HopDataRealm,
    next_address: [u8; ADDRESS_SIZE],
    forward_amount: Satoshi,
    outgoing_cltv: u32,
    header_mac: [u8; HMAC_SIZE],
}

impl Serialize for HopData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut tuple = serializer.serialize_tuple(6)?;
        tuple.serialize_element(&self.realm)?;
        tuple.serialize_element(&self.next_address)?;
        tuple.serialize_element(&self.forward_amount)?;
        tuple.serialize_element(&self.outgoing_cltv)?;
        tuple.serialize_element(&[0; PAD_SIZE])?;
        tuple.serialize_element(&self.header_mac)?;
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for HopData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{Visitor, SeqAccess, Error};
        use std::fmt;

        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = HopData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "HopData {} bytes", HOP_DATA_SIZE)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let realm = seq
                    .next_element()?
                    .ok_or(Error::custom("expecting header byte, 0 for bitcoin"))?;
                let next_address = seq
                    .next_element()?
                    .ok_or(Error::custom("expecting addess"))?;
                let forward_amount = seq
                    .next_element()?
                    .ok_or(Error::custom("expecting satoshi amount"))?;
                let outgoing_cltv = seq.next_element()?.ok_or(Error::custom("expecting cltv"))?;
                let _: [u8; PAD_SIZE] = seq.next_element()?.ok_or(Error::custom(format!(
                    "expecting padding {} bytes",
                    PAD_SIZE
                )))?;
                let header_mac = seq.next_element()?.ok_or(Error::custom("expecting HMAC"))?;

                Ok(HopData {
                    realm: realm,
                    next_address: next_address,
                    forward_amount: forward_amount,
                    outgoing_cltv: outgoing_cltv,
                    header_mac: header_mac,
                })
            }
        }

        deserializer.deserialize_tuple(6, V)
    }
}

/// generate shared secrets by the given `payment_path` and `session_key`
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
