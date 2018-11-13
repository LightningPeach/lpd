use super::hop::{Hop, HopData};
use super::{NUM_MAX_HOPS, HMAC_SIZE};

use secp256k1::{SecretKey, PublicKey, Error as EcdsaError};
use wire::PublicKey as WirePublicKey;
use serde::{Serialize, Serializer};
use serde_derive::Serialize;

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum OnionPacketVersion {
    _0 = 0,
}

/// `OnionRoute` is hop to hop routing information necessary to propagate a message
#[derive(Debug, Eq, PartialEq)]
pub struct OnionRoute {
    version: OnionPacketVersion,
    session_key: SecretKey,
    route: Vec<Hop>,
    associated_data: Vec<u8>,
}

impl OnionRoute {
    /// Dummy constructor, `associated_data` could be empty
    pub fn new(
        version: OnionPacketVersion,
        session_key: SecretKey,
        route: Vec<Hop>,
        associated_data: Vec<u8>
    ) -> Self {
        OnionRoute {
            version: version,
            session_key: session_key,
            route: route,
            associated_data: associated_data,
        }
    }

    /// Compute the packet
    pub fn packet(self) -> Result<OnionPacket, EcdsaError> {
        use secp256k1::Secp256k1;
        use common_types::Hash256;
        use hmac::{Hmac, Mac};
        use chacha::{ChaCha, KeyStream};
        use std::default::Default;

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

        // `KEY_LEN` is the length of the keys used to generate cipher streams and
        // encrypt payloads. Since we use SHA256 to generate the keys, the
        // maximum length currently is 32 bytes.
        const KEY_LEN: usize = 32;

        fn generate_header_padding(key_type: &str, shared_secrets: &[Hash256]) -> Vec<u8> {
            let num = shared_secrets.len();
            let mut filler = vec![0; (num - 1) * HopData::SIZE];

            for i in 1..num {
                let stream_key = generate_key(key_type, shared_secrets[i - 1]);
                let stream_bytes = generate_cipher_stream(stream_key, NUM_STREAM_BYTES);

                for j in 0..(i * HopData::SIZE) {
                    filler[j] ^= stream_bytes[(NUM_STREAM_BYTES - i * HopData::SIZE) + j];
                }
            }

            filler
        }

        // generate_key generates a new key for usage in Sphinx packet
        // construction/processing based off of the denoted keyType. Within Sphinx
        // various keys are used within the same onion packet for padding generation,
        // MAC generation, and encryption/decryption.
        fn generate_key(key_type: &str, shared_key: Hash256) -> [u8; KEY_LEN] {
            use sha2::Sha256;

            let mut mac = Hmac::<Sha256>::new_varkey(key_type.as_bytes()).unwrap();
            mac.input(shared_key.as_ref());
            let result = mac.result().code();

            let mut array: [u8; 32] = [0; 32];
            array.copy_from_slice(result.as_slice());
            array
        }

        // generate_cipher_stream generates a stream of cryptographic pseudo-random bytes
        // intended to be used to encrypt a message using a one-time-pad like
        // construction.
        fn generate_cipher_stream(key: [u8; KEY_LEN], num_bytes: usize) -> Vec<u8> {
            let mut output = vec![0; num_bytes];
            ChaCha::new_chacha20(&key, &[0u8; 8])
                .xor_read(output.as_mut_slice())
                .unwrap();
            output
        }

        let context = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&context, &self.session_key)?;

        Ok(OnionPacket {
            version: OnionPacketVersion::_0,
            ephemeral_key: WirePublicKey::from(public_key),
            routing_info: unimplemented!(),
            hmac: HmacData::default(),
        })
    }
}

// `NUM_STREAM_BYTES` is the number of bytes produced by our CSPRG for the
// key stream implementing our stream cipher to encrypt/decrypt the mix
// header. The last `HOP_DATA_SIZE` bytes are only used in order to
// generate/check the MAC over the header.
const NUM_STREAM_BYTES: usize = (NUM_MAX_HOPS + 1) * (HopData::SIZE + HMAC_SIZE);

#[derive(Default, Debug, Eq, PartialEq, Serialize)]
struct HmacData {
    data: [u8; HMAC_SIZE],
}

struct HopBytes {
    data: [u8; HopData::SIZE],
    hmac: HmacData,
}

impl Serialize for HopBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut tuple = serializer.serialize_tuple(3)?;
        tuple.serialize_element(&self.data[0])?;
        tuple.serialize_element(&self.data[1..])?;
        tuple.serialize_element(&self.hmac)?;
        tuple.end()
    }
}

#[derive(Serialize)]
pub struct OnionPacket {
    version: OnionPacketVersion,
    ephemeral_key: WirePublicKey,
    routing_info: [HopBytes; NUM_MAX_HOPS],
    hmac: HmacData,
}
