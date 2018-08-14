use super::ChannelId;
use super::Signature;
use super::PublicKey;
use super::OutputIndex;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
pub struct FundingTxid {
    data: [u8; 32],
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct FundingCreated {
    pub temporary_channel_id: ChannelId,
    pub funding_txid: FundingTxid,
    pub output_index: OutputIndex,
    pub signature: Signature,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct FundingSigned {
    pub channel_id: ChannelId,
    pub signature: Signature,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct FundingLocked {
    channel_id: ChannelId,
    next_per_commitment_point: PublicKey,
}

impl From<FundingTxid> for [u8; 32] {
    fn from(tx_id: FundingTxid) -> Self {
        return tx_id.data;
    }
}