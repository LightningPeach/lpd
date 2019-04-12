use super::ChannelId;
use super::OutputIndex;
use super::super::types::RawSignature;
use secp256k1::PublicKey;

use crate::message::types::RawPublicKey;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub struct FundingTxid {
    data: [u8; 32],
}

// In Bitcoin TxId are printed y convention in reverse order
impl std::fmt::Debug for FundingTxid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut d = self.data.clone();
        d.reverse();
        write!(f, "{}", hex::encode(&d[..]))
    }
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
    pub next_per_commitment_point: RawPublicKey,
}

impl From<FundingTxid> for [u8; 32] {
    fn from(tx_id: FundingTxid) -> Self {
        return tx_id.data;
    }
}