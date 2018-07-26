use super::ChannelId;
use super::Satoshi;
use super::Signature;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct ShutdownChannel {
    channel_id: ChannelId,
    script: Vec<u8>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct ClosingNegotiation {
    channel_id: ChannelId,
    fee: Satoshi,
    signature: Signature,
}
