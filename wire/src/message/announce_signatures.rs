use super::ChannelId;
use super::ShortChannelId;
use super::types::RawSignature;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct AnnounceSignatures {
    channel_id: ChannelId,
    short_channel_id: ShortChannelId,
    node_signature: RawSignature,
    bitcoin_signature: RawSignature,
}
