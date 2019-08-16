use dependencies::bitflags;

use super::Hash256;
use super::ShortChannelId;
use super::MilliSatoshi;
use super::super::types::RawSignature;

use common_types::secp256k1_m::{Signed, Data};

use bitflags::bitflags;
use serde_derive::{Serialize, Deserialize};

pub type UpdateChannel = Signed<Data<UpdateChannelData>, RawSignature>;

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct ChannelUpdateFlags: u16 {
        const DIRECTION = 0b00000001;
        const DISABLED = 0b00000010;
    }
}

/// After a channel has been initially announced, each side independently announces
/// the fees and minimum expiry delta it requires to relay HTLCs through this channel.
/// Each uses the 8-byte channel shortid that matches the `channel_announcement`
/// and the 1-bit `channel_flags` field to indicate which end of the channel it's on
/// (origin or final). A node can do this multiple times, in order to change fees.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct UpdateChannelData {
    pub hash: Hash256,
    pub short_channel_id: ShortChannelId,
    pub timestamp: u32,
    pub message_flags: u8,
    pub channel_flags: u8,
    pub time_lock_delta: u16,
    pub htlc_minimum: MilliSatoshi,
    pub base_fee: u32,
    pub fee_rate: u32,
    pub htlc_maximum: MilliSatoshi,
}

impl UpdateChannelData {
    pub fn hash(&self) -> &Hash256 {
        &self.hash
    }

    pub fn id(&self) -> &ShortChannelId {
        &self.short_channel_id
    }
}

#[cfg(test)]
mod test {
    use dependencies::hex;
    use dependencies::pretty_assertions;

    use super::*;
    use binformat::BinarySD;
    use std::io::Cursor;
    use crate::Message;
    use pretty_assertions::assert_eq;

    #[test]
    fn update_channel_test() {
        let msg_hex = "\
            01020000030000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000004000000000000000000000000\
            000000000000000000000000000000000000000064000004000f000f424001010064000000000000\
            03e800000064000000050000000005f5e100";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = Signed {
            signature: RawSignature::from_hex("3023021e030000000000000000000000000000000000000000000000000000000000020100").unwrap(),
            data: Data(
                UpdateChannelData{
                    hash: Hash256::from_hex("0004000000000000000000000000000000000000000000000000000000000000").unwrap(),
                    short_channel_id: ShortChannelId::from_u64(109951163039759),
                    timestamp: 1000000,
                    message_flags: 1,
                    channel_flags: 1,
                    time_lock_delta: 100,
                    htlc_minimum: MilliSatoshi::from(1000),
                    base_fee: 100,
                    fee_rate: 5,
                    htlc_maximum: MilliSatoshi::from(100000000),
                }
            )
        };
        let wrapped_msg_correct = Message::UpdateChannel(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }
}