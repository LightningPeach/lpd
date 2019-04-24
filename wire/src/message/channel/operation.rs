use super::ChannelId;
use super::Hash256;
use super::MilliSatoshi;
use super::OnionBlob;
use super::SatoshiPerKiloWeight;
use super::super::types::{RawSignature, RawPublicKey};

use binformat::SerdeVec;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
pub struct HtlcId {
    id: u64,
}

impl HtlcId {
    pub fn new() -> Self {
        HtlcId {
            id: 0,
        }
    }

    pub fn next(&self) -> Self {
        HtlcId {
            id: self.id + 1,
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct UpdateAddHtlc {
    pub channel_id: ChannelId,
    pub id: HtlcId,
    pub amount: MilliSatoshi,
    pub payment: Hash256,
    pub expiry: u32,
    pub onion_blob: OnionBlob,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct UpdateFulfillHtlc {
    pub channel_id: ChannelId,
    pub id: HtlcId,
    pub payment_preimage: [u8; 32],
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct UpdateFailHtlc {
    channel_id: ChannelId,
    id: HtlcId,
    reason: Vec<u8>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct UpdateFailMalformedHtlc {
    channel_id: ChannelId,
    id: HtlcId,
    sha256_of_onion: Hash256,
    failure_code: u16,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct CommitmentSigned {
    pub channel_id: ChannelId,
    pub signature: RawSignature,
    pub htlc_signatures: SerdeVec<RawSignature>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct RevokeAndAck {
    pub channel_id: ChannelId,
    pub revocation_preimage: [u8; 32],
    pub next_per_commitment_point: RawPublicKey,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct UpdateFee {
    channel_id: ChannelId,
    fee: SatoshiPerKiloWeight,
}
