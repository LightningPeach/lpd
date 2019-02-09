use super::ChannelId;
use super::OutputIndex;
use super::super::types::RawSignature;
use secp256k1::PublicKey;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
pub struct FundingTxid {
    data: [u8; 32],
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct FundingCreated {
    pub temporary_channel_id: ChannelId,
    pub funding_txid: FundingTxid,
    pub output_index: OutputIndex,
    pub signature: RawSignature,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct FundingSigned {
    pub channel_id: ChannelId,
    pub signature: RawSignature,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct FundingLocked {
    pub channel_id: ChannelId,
    pub next_per_commitment_point: PublicKey,
}

impl From<FundingTxid> for [u8; 32] {
    fn from(tx_id: FundingTxid) -> Self {
        return tx_id.data;
    }
}