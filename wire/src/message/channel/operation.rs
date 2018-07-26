use super::ChannelId;
use super::Hash256;
use super::MilliSatoshi;
use super::OnionBlob;
use super::Signature;
use super::PublicKey;
use super::SatoshiPerKiloWeight;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
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

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct UpdateAddHtlc {
    channel_id: ChannelId,
    id: HtlcId,
    amount: MilliSatoshi,
    payment: Hash256,
    expiry: u32,
    onion_blob: OnionBlob,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct UpdateFulfillHtlc {
    channel_id: ChannelId,
    id: HtlcId,
    payment_preimage: [u8; 32],
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct UpdateFailHtlc {
    channel_id: ChannelId,
    id: HtlcId,
    reason: Vec<u8>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct UpdateFailMalformedHtlc {
    channel_id: ChannelId,
    id: HtlcId,
    sha256_of_onion: Hash256,
    failure_code: u16,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct CommitmentSigned {
    channel_id: ChannelId,
    signature: Signature,
    htlc_signatures: Vec<Signature>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct RevokeAndAck {
    channel_id: ChannelId,
    revocation_preimage: [u8; 32],
    next_per_commitment_point: PublicKey,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct UpdateFee {
    channel_id: ChannelId,
    fee: SatoshiPerKiloWeight,
}
