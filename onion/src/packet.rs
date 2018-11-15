use super::{hop::{Hop, HopBytes}, crypto::{KeyType, HmacData}, route::OnionPacketVersion};
use secp256k1::{SecretKey, PublicKey, Error as EcdsaError};
use wire::PublicKey as WirePublicKey;
use serde_derive::{Serialize, Deserialize};
use common_types::Hash256;

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct OnionPacket {
    pub(crate) version: u8,
    pub(crate) ephemeral_key: WirePublicKey,
    pub(crate) routing_info: [HopBytes; OnionPacket::NUM_MAX_HOPS],
    pub(crate) hmac: HmacData,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct ValidOnionPacket(pub(crate) OnionPacket);

pub enum Processed {
    ExitNode,
    MoreHops {
        next: OnionPacket,
        forwarding_instructions: Hop,
    },
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

        if !self.ephemeral_key.as_ref().is_valid() {
            let msg = format!("public key is invalid");
            return Err((self, msg));
        }

        Ok(ValidOnionPacket(self))
    }
}

impl ValidOnionPacket {
    pub fn process(
        self,
        associated_data: Vec<u8>,
        incoming_cltv: u32,
        onion_key: &SecretKey,
    ) -> Result<Processed, EcdsaError> {
        use secp256k1::Secp256k1;
        use wire::BinarySD;

        let context = Secp256k1::new();

        let ValidOnionPacket(s) = self;

        let (ephemeral_key, routing_info, hmac) = (s.ephemeral_key, s.routing_info, s.hmac);

        let mul_pk = |x: &PublicKey, sk: &SecretKey| {
            let mut temp = x.clone();
            temp.mul_assign(&context, sk).map(|()| temp)
        };
        let hash = |x: &[u8]| -> Hash256 { Hash256::from(x) };

        let temp = mul_pk(ephemeral_key.as_ref(), onion_key)?;
        let shared_secret = hash(&temp.serialize()[..]);

        let mut data = Vec::with_capacity(HopBytes::SIZE * OnionPacket::NUM_MAX_HOPS);
        // it is believed that such serialization won't fail
        BinarySD::serialize(&mut data, &routing_info).unwrap();

        let hmac_calc = KeyType::Mu.hmac(
            shared_secret,
            &[data.as_slice(), associated_data.as_slice()],
        );
        assert_eq!(hmac, hmac_calc);

        let _ = incoming_cltv;
        unimplemented!()
    }
}
