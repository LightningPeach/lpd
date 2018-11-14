use super::hop::{Hop, HopBytes};

use secp256k1::{SecretKey, PublicKey, Error as EcdsaError};
use wire::PublicKey as WirePublicKey;
use serde_derive::{Serialize, Deserialize};
use chacha::{ChaCha, KeyStream};
use common_types::Hash256;
use std::ops::BitXorAssign;

/// `NUM_MAX_HOPS` is the the maximum path length. This should be set to an
/// estimate of the upper limit of the diameter of the node graph.
pub const NUM_MAX_HOPS: usize = 20;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub enum OnionPacketVersion {
    _0 = 0,
}

impl From<u8> for OnionPacketVersion {
    fn from(v: u8) -> Self {
        use self::OnionPacketVersion::*;

        match v {
            0 => _0,
            _ => panic!("unknown onion packet version"),
        }
    }
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
    version: u8,
    ephemeral_key: WirePublicKey,
    routing_info: [HopBytes; NUM_MAX_HOPS],
    hmac: HmacData,
}

impl OnionRoute {
    /// Dummy constructor, `associated_data` could be empty
    pub fn new(
        version: OnionPacketVersion,
        session_key: SecretKey,
        route: Vec<Hop>,
        associated_data: Vec<u8>,
    ) -> Self {
        assert!(route.len() <= NUM_MAX_HOPS);

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
        use wire::BinarySD;

        fn generate_shared_secrets<'a, I>(
            payment_path: I,
            session_key: &SecretKey,
        ) -> Result<Vec<Hash256>, EcdsaError>
        where
            I: Iterator<Item = &'a PublicKey>,
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

        fn generate_header_padding(
            key_type: &KeyType,
            shared_secrets: &[Hash256],
        ) -> Vec<HopBytes> {
            let num = shared_secrets.len();
            let mut filler = vec![HopBytes::zero(); num - 1];

            for i in 1..num {
                use chacha::SeekableKeyStream;

                let mut stream = key_type.chacha(shared_secrets[i - 1]);
                stream
                    .seek_to(((NUM_MAX_HOPS - i + 1) * HopBytes::SIZE) as _)
                    .unwrap();
                for j in 0..i {
                    filler[j] ^= &mut stream;
                }
            }

            filler
        }

        let context = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&context, &self.session_key)?;

        let (filler, hop_shared_secrets) = {
            let i = self.route.iter().map(|hop| hop.id());
            let hop_shared_secrets = generate_shared_secrets(i, &self.session_key)?;
            (
                generate_header_padding(&KeyType::Rho, hop_shared_secrets.as_slice()),
                hop_shared_secrets,
            )
        };

        let mut hmac = HmacData::default();
        let mut hops_bytes = [HopBytes::zero(); NUM_MAX_HOPS];

        // decompose self
        let (version, route, associated_data) = (self.version, self.route, self.associated_data);

        route
            .into_iter()
            .enumerate()
            .rev()
            .for_each(|(index, hop)| {
                let mut rho_stream = KeyType::Rho.chacha(hop_shared_secrets[index]);

                // shift right
                for i in (1..NUM_MAX_HOPS).rev() {
                    hops_bytes[i] = hops_bytes[i - 1];
                }
                hops_bytes[0] = HopBytes::new(hop, hmac.clone());

                // xor with the rho stream
                hops_bytes.iter_mut().for_each(|x| *x ^= &mut rho_stream);

                // for first iteration
                if index == filler.len() {
                    for i in 0..filler.len() {
                        hops_bytes[NUM_MAX_HOPS - filler.len() + i] = filler[i];
                    }
                }

                let mut data = Vec::with_capacity(HopBytes::SIZE * NUM_MAX_HOPS);
                BinarySD::serialize(&mut data, &hops_bytes).unwrap();
                hmac = KeyType::Mu.hmac(
                    hop_shared_secrets[index],
                    &[data.as_slice(), associated_data.as_slice()],
                );
            });

        Ok(OnionPacket {
            version: version as _,
            ephemeral_key: WirePublicKey::from(public_key),
            routing_info: hops_bytes,
            hmac: hmac,
        })
    }
}

enum KeyType {
    Rho,
    Mu,
}

impl KeyType {
    // `KEY_LEN` is the length of the keys used to generate cipher streams and
    // encrypt payloads. Since we use SHA256 to generate the keys, the
    // maximum length currently is 32 bytes.
    const KEY_LEN: usize = 32;

    fn key(&self, shared_key: Hash256) -> [u8; Self::KEY_LEN] {
        use sha2::Sha256;
        use hmac::{Hmac, Mac};
        use self::KeyType::*;

        let key_type = match self {
            &Rho => "rho",
            &Mu => "mu",
        };

        let mut mac = Hmac::<Sha256>::new_varkey(key_type.as_bytes()).unwrap();
        mac.input(shared_key.as_ref());
        let result = mac.result().code();
        let mut array = [0; Self::KEY_LEN];
        array.copy_from_slice(result.as_slice());
        array
    }

    fn chacha(&self, shared_key: Hash256) -> ChaCha {
        ChaCha::new_chacha20(&self.key(shared_key), &[0u8; 8])
    }

    fn hmac(&self, shared_key: Hash256, msg: &[&[u8]]) -> HmacData {
        use sha2::Sha256;
        use hmac::{Hmac, Mac};

        let key = self.key(shared_key);
        let mac = Hmac::<Sha256>::new_varkey(&key).unwrap();
        let mut mac = msg.iter().fold(mac, |mut mac, &x| {
            mac.input(x);
            mac
        });
        let result = mac.result().code();
        let mut hmac = HmacData::default();
        hmac.data.copy_from_slice(result.as_slice());
        hmac
    }
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HmacData {
    data: [u8; 32],
}

impl HmacData {
    pub const SIZE: usize = 32;
}

impl<'a> BitXorAssign<&'a mut ChaCha> for HmacData {
    fn bitxor_assign(&mut self, rhs: &'a mut ChaCha) {
        rhs.xor_read(&mut self.data[..]).unwrap()
    }
}
