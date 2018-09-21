use super::Signed;
use super::Hash256;
use super::ShortChannelId;
use super::MilliSatoshi;
use super::AnnouncementChannel;

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
    pub timestamp: u32,
    pub flags: ChannelUpdateFlags,
    pub time_lock_delta: u16,
    pub htlc_minimum: MilliSatoshi,
    pub base_fee: u32,
    pub fee_rate: u32,
}

impl UpdateChannelData {
    pub fn hash(&self) -> &Hash256 {
        &self.hash
    }

    pub fn id(&self) -> &ShortChannelId {
        &self.short_channel_id
    }
}
