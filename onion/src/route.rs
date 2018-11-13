use super::hop::{Hop, HopBytes};

use secp256k1::{SecretKey, PublicKey, Error as EcdsaError};
use wire::PublicKey as WirePublicKey;
use serde_derive::Serialize;
use smallvec::SmallVec;
use std::ops::BitXorAssign;

/// `NUM_MAX_HOPS` is the the maximum path length. This should be set to an
/// estimate of the upper limit of the diameter of the node graph.
pub const NUM_MAX_HOPS: usize = 20;

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

#[derive(Serialize)]
pub struct OnionPacket {
    version: OnionPacketVersion,
    ephemeral_key: WirePublicKey,
    routing_info: SmallVec<[HopBytes; NUM_MAX_HOPS]>,
    hmac: HmacData,
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

        fn generate_shared_secrets<'a, I>(
            payment_path: I,
            session_key: &SecretKey,
        ) -> Result<Vec<Hash256>, EcdsaError>
        where
            I: Iterator<Item=&'a PublicKey>,
        {
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
                Vec::new(),
                session_key.clone(),
                PublicKey::from_secret_key(&context, session_key)?,
                Hash256::from([0; 32]),
            );

            let mut payment_path = payment_path;
            payment_path
                .try_fold(initial, |(mut v, secret, public, blinding), path_point| {
                    let temp = mul_pk(path_point, &secret)?;
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
            let mut filler = vec![0; (num - 1) * HopBytes::SIZE];

            for i in 1..num {
                let stream_key = generate_key(key_type, shared_secrets[i - 1]);
                let stream_bytes = generate_cipher_stream(stream_key, NUM_STREAM_BYTES);

                for j in 0..(i * HopBytes::SIZE) {
                    filler[j] ^= stream_bytes[(NUM_STREAM_BYTES - i * HopBytes::SIZE) + j];
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

        let (filler, hop_shared_secrets) = {
            let i = self.route.iter().map(|hop| hop.id());
            let hop_shared_secrets = generate_shared_secrets(i, &self.session_key)?;
            (generate_header_padding("rho", hop_shared_secrets.as_slice()), hop_shared_secrets)
        };

        let mut hmac = HmacData::default();
        let hops_bytes = self.route.into_iter().enumerate().rev().map(|(i, hop)| {
            let rho_key = generate_key("rho", hop_shared_secrets[i]);
            let mu_key = generate_key("mu", hop_shared_secrets[i]);
            let stream_bytes = generate_cipher_stream(rho_key, NUM_STREAM_BYTES);

            HopBytes::new(hop, hmac.clone())
        }).rev().collect::<SmallVec<_>>();

        Ok(OnionPacket {
            version: OnionPacketVersion::_0,
            ephemeral_key: WirePublicKey::from(public_key),
            routing_info: hops_bytes,
            hmac: HmacData::default(),
        })
    }
}

// `NUM_STREAM_BYTES` is the number of bytes produced by our CSPRG for the
// key stream implementing our stream cipher to encrypt/decrypt the mix
// header. The last `HOP_DATA_SIZE` bytes are only used in order to
// generate/check the MAC over the header.
const NUM_STREAM_BYTES: usize = (NUM_MAX_HOPS + 1) * HopBytes::SIZE;

#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize)]
pub struct HmacData {
    data: [u8; 32],
}

impl HmacData {
    pub const SIZE: usize = 32;
}

impl BitXorAssign for HmacData {
    fn bitxor_assign(&mut self, rhs: Self) {
        for i in 0..HmacData::SIZE {
            self.data[i] ^= rhs.data[i];
        }
    }
}
