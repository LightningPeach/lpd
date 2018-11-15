use super::{Hop, hop::HopBytes, crypto::HmacData};
use secp256k1::SecretKey;
use wire::PublicKey as WirePublicKey;
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OnionPacket {
    pub(crate) version: u8,
    pub(crate) ephemeral_key: WirePublicKey,
    pub(crate) routing_info: [HopBytes; OnionPacket::NUM_MAX_HOPS],
    pub(crate) hmac: HmacData,
}

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

    pub fn process(
        &self,
        associated_data: Vec<u8>,
        incoming_cltv: u32,
        onion_key: &SecretKey
    ) -> Result<Processed, ()> {
        let _ = (associated_data, incoming_cltv, onion_key);
        Err(())
    }
}
