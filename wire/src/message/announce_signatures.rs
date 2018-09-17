use super::ChannelId;
use super::ShortChannelId;
use super::types::Signature;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct AnnounceSignatures {
    channel_id: ChannelId,
    short_channel_id: ShortChannelId,
    node_signature: Signature,
    bitcoin_signature: Signature,
}
