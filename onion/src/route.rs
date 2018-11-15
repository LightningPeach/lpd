use super::hop::{Hop, HopBytes};
use super::crypto::{HmacData, KeyType};
use super::packet::OnionPacket;

use secp256k1::{SecretKey, PublicKey, Error as EcdsaError};
use wire::PublicKey as WirePublicKey;
use common_types::Hash256;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

impl OnionRoute {
    /// Dummy constructor, `associated_data` could be empty
    pub fn new(
        version: OnionPacketVersion,
        session_key: SecretKey,
        route: Vec<Hop>,
        associated_data: Vec<u8>,
    ) -> Self {
        assert!(route.len() <= OnionPacket::NUM_MAX_HOPS);

        OnionRoute {
            version: version,
            session_key: session_key,
            route: route,
            associated_data: associated_data,
        }
    }

    /// Generate the packet
    pub fn packet(self) -> Result<OnionPacket, EcdsaError> {
        use secp256k1::Secp256k1;
        use wire::BinarySD;

        fn generate_shared_secrets<'a, I>(
            context: &Secp256k1,
            payment_path: I,
            session_key: &SecretKey,
        ) -> Result<Vec<Hash256>, EcdsaError>
        where
            I: Iterator<Item = &'a PublicKey>,
        {
            // functions
            // `mul_pk` and `mul_sk` obviously performs the multiplication in the elliptic curve group
            // `hash` or `hash_s` computes a sha256 hash from a given slice or slices
            // `hash_to_sk` obviously casts a sha256 hash into a secret key
            let mul_pk = |x: &PublicKey, sk: &SecretKey| {
                let mut temp = x.clone();
                temp.mul_assign(context, sk).map(|()| temp)
            };
            let mul_sk = |x: &SecretKey, sk: &SecretKey| {
                let mut temp: SecretKey = x.clone();
                temp.mul_assign(context, sk).map(|()| temp)
            };
            let hash = |x: &[u8]| -> Hash256 { Hash256::from(x) };
            let hash_s = |xs: &[&[u8]]| -> Hash256 { Hash256::from(xs) };
            let hash_to_sk = |hash: &Hash256| SecretKey::from_slice(&context, hash.as_ref());

            // secp256k1 base point G
            let base_point = {
                // the string represents valid secp256k1 element, so both unwrap calls are safe
                let s = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
                PublicKey::from_slice(context, hex::decode(s).unwrap().as_slice()).unwrap()
            };

            let initial = (
                Vec::new(),
                session_key.clone(),
                PublicKey::from_secret_key(context, session_key)?,
            );

            let mut payment_path = payment_path;
            payment_path
                .try_fold(initial, |(mut v, secret, public), path_point| {
                    let temp = mul_pk(path_point, &secret)?;
                    let result = hash(&temp.serialize()[..]);
                    let blinding = hash_s(&[&public.serialize()[..], result.as_ref()][..]);
                    let secret = mul_sk(&secret, &hash_to_sk(&blinding)?)?;
                    let public = mul_pk(&base_point, &secret)?;

                    v.push(result);
                    Ok((v, secret, public))
                }).map(|(v, _, _)| v)
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
                    .seek_to(((OnionPacket::NUM_MAX_HOPS - i + 1) * HopBytes::SIZE) as _)
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
            let hop_shared_secrets = generate_shared_secrets(&context, i, &self.session_key)?;
            (
                generate_header_padding(&KeyType::Rho, hop_shared_secrets.as_slice()),
                hop_shared_secrets,
            )
        };

        let mut hmac = HmacData::default();
        let mut hops_bytes = [HopBytes::zero(); OnionPacket::NUM_MAX_HOPS];

        // decompose self
        let (version, route, associated_data) = (self.version, self.route, self.associated_data);

        route
            .into_iter()
            .enumerate()
            .rev()
            .for_each(|(index, hop)| {
                let mut rho_stream = KeyType::Rho.chacha(hop_shared_secrets[index]);

                // shift right
                for i in (1..OnionPacket::NUM_MAX_HOPS).rev() {
                    hops_bytes[i] = hops_bytes[i - 1];
                }
                hops_bytes[0] = HopBytes::new(hop, hmac.clone());

                // xor with the rho stream
                hops_bytes.iter_mut().for_each(|x| *x ^= &mut rho_stream);

                // for first iteration
                if index == filler.len() {
                    for i in 0..filler.len() {
                        hops_bytes[OnionPacket::NUM_MAX_HOPS - filler.len() + i] = filler[i];
                    }
                }

                let mut data = Vec::with_capacity(HopBytes::SIZE * OnionPacket::NUM_MAX_HOPS);
                // it is believed that such serialization won't fail
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
