use super::ChannelId;
use super::Satoshi;
use super::Signature;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ShutdownChannel {
    channel_id: ChannelId,
    script: Vec<u8>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ClosingNegotiation {
    channel_id: ChannelId,
    fee: Satoshi,
    signature: Signature,
}
