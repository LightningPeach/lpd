use super::Signed;
use super::Hash256;
use super::ShortChannelId;
use super::MilliSatoshi;

pub type UpdateChannel = Signed<UpdateChannelData>;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct UpdateChannelData {
    hash: Hash256,
    short_channel_id: ShortChannelId,
    timestamp: u32,
    // TODO: bitflags
    flags: u16,
    time_lock_delta: u16,
    htlc_minimum: MilliSatoshi,
    base_fee: u32,
    fee_rate: u32,
}
