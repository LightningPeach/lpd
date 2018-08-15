use super::ChannelId;
use super::ShortChannelId;
use super::types::Signature;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct AnnounceSignature {
    channel_id: ChannelId,
    short_channel_id: ShortChannelId,
    node_signature: Signature,
    bitcoin_signature: Signature,
}
