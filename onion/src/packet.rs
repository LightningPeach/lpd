use dependencies::secp256k1;

use super::{hop::{HopData, HopBytes}, crypto::{KeyType, HmacData}, route::OnionPacketVersion};
use secp256k1::{SecretKey, PublicKey, Error as EcdsaError};
use serde::{Serialize, Deserialize};
use common_types::{Hash256, RawPublicKey};

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct OnionPacket {
    pub(crate) version: u8,
    pub(crate) ephemeral_key: RawPublicKey,
    pub(crate) routing_info: [HopBytes; OnionPacket::NUM_MAX_HOPS],
    pub(crate) hmac: HmacData,
}

impl OnionPacket {
    /// `NUM_MAX_HOPS` is the the maximum path length. This should be set to an
    /// estimate of the upper limit of the diameter of the node graph.
    pub const NUM_MAX_HOPS: usize = 20;

    // TODO(vlad): validate ephemeral_key, hmac and maybe other things
    pub fn validate(self) -> Result<ValidOnionPacket, (Self, String)> {
        if OnionPacketVersion::from(self.version) != OnionPacketVersion::_0 {
            let msg = format!("unknown packet version: {}", self.version);
            return Err((self, msg));
        }

        Ok(ValidOnionPacket(self))
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct ValidOnionPacket(pub(crate) OnionPacket);

#[derive(Debug)]
pub enum OnionPacketProcessingError {
    EcdsaError(EcdsaError),
    WrongHmac,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Processed {
    ExitNode,
    MoreHops {
        next: OnionPacket,
        forwarding_instructions: HopData,
    },
}

impl ValidOnionPacket {
    pub fn process(
        self,
        associated_data: Vec<u8>,
        incoming_cltv: u32,
        onion_key: &SecretKey,
    ) -> Result<Processed, OnionPacketProcessingError> {
        use secp256k1::Secp256k1;
        use binformat::BinarySD;
        use self::OnionPacketProcessingError::*;

        let context = Secp256k1::new();

        let ValidOnionPacket(s) = self;

        let (version, RawPublicKey(ephemeral_key), routing_info, hmac) =
            (s.version, s.ephemeral_key, s.routing_info, s.hmac);

        // TODO(vlad): reuse this code
        let mul_pk = |x: &PublicKey, sk: &SecretKey| {
            let mut temp = x.clone();
            temp.mul_assign(&context, &sk[..]).map(|()| temp)
        };
        let hash = |x: &[u8]| -> Hash256 { Hash256::from(x) };
        let hash_s = |xs: &[&[u8]]| -> Hash256 { Hash256::from(xs) };
        let hash_to_sk = |hash: &Hash256| SecretKey::from_slice(hash.as_ref());

        let temp = mul_pk(&ephemeral_key, onion_key).map_err(EcdsaError)?;
        let shared_secret = hash(&temp.serialize()[..]);

        let mut data = Vec::with_capacity(HopBytes::SIZE * OnionPacket::NUM_MAX_HOPS);
        // it is believed that such serialization won't fail
        BinarySD::serialize(&mut data, &routing_info).unwrap();

        let hmac_calc = KeyType::Mu.hmac(
            shared_secret,
            &[data.as_slice(), associated_data.as_slice()],
        );

        if hmac_calc != hmac {
            Err(WrongHmac)
        } else {
            // TODO(vlad): how should use it?
            let _ = incoming_cltv;

            let mut rho_stream = KeyType::Rho.chacha(shared_secret);
            let mut routing_info_extended = routing_info.to_vec();
            routing_info_extended.push(HopBytes::zero());
            routing_info_extended
                .iter_mut()
                .for_each(|x| *x ^= &mut rho_stream);

            let dh_key = ephemeral_key;
            let blinding = hash_s(&[&dh_key.serialize()[..], shared_secret.as_ref()][..]);
            let next_dh_key =
                mul_pk(&dh_key, &hash_to_sk(&blinding).map_err(EcdsaError)?).map_err(EcdsaError)?;

            // cut first
            let info = routing_info_extended.remove(0);
            let (forwarding_instructions, hmac) = info.destruct();

            if hmac.is_zero() {
                Ok(Processed::ExitNode)
            } else {
                let mut next = OnionPacket {
                    version: version,
                    ephemeral_key: next_dh_key.into(),
                    routing_info: [HopBytes::zero(); OnionPacket::NUM_MAX_HOPS],
                    hmac: hmac,
                };

                next.routing_info[..].copy_from_slice(routing_info_extended.as_slice());

                Ok(Processed::MoreHops {
                    next: next,
                    forwarding_instructions: forwarding_instructions,
                })
            }
        }
    }
}
