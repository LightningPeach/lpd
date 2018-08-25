use super::Signed;
use super::Hash256;
use super::ShortChannelId;
use super::MilliSatoshi;

pub type UpdateChannel = Signed<UpdateChannelData>;

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct ChannelUpdateFlags: u16 {
        const DIRECTION = 0b00000001;
        const DISABLED = 0b00000010;
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct UpdateChannelData {
    hash: Hash256,
    short_channel_id: ShortChannelId,
    timestamp: u32,
    flags: ChannelUpdateFlags,
    time_lock_delta: u16,
    htlc_minimum: MilliSatoshi,
    base_fee: u32,
    fee_rate: u32,
}
