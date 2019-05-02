use super::ChannelId;
use super::ShortChannelId;
use super::types::RawSignature;

use serde_derive::{Serialize, Deserialize};

/// This is a direct message between the two endpoints of a channel and serves
/// as an opt-in mechanism to allow the announcement of the channel to the rest of the network.
/// It contains the necessary signatures, by the sender,
/// to construct the channel_announcement message.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct AnnounceSignatures {
    pub channel_id: ChannelId,
    pub short_channel_id: ShortChannelId,
    pub node_signature: RawSignature,
    pub bitcoin_signature: RawSignature,
}


#[cfg(test)]
mod test {
    use super::*;
    use binformat::BinarySD;
    use crate::message::channel::ChannelId;
    use crate::message::AnnounceSignatures;
    use std::io::Cursor;
    use crate::Message;
    use pretty_assertions::assert_eq;

    #[test]
    fn announce_signatures_test() {
        let msg_hex = "\
            0103010000000000000000000000000000000000000000000000000000000000000000006400000a\
            00010000030000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000004000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000";
        let msg_bytes = hex::decode(msg_hex).unwrap();
        let msg_correct = AnnounceSignatures{
            channel_id: ChannelId::from_hex("0100000000000000000000000000000000000000000000000000000000000000").unwrap(),
            short_channel_id: ShortChannelId::from_u64(109951163432961),
            node_signature: RawSignature::from_hex("3023021e030000000000000000000000000000000000000000000000000000000000020100").unwrap(),
            bitcoin_signature: RawSignature::from_hex("3024021f04000000000000000000000000000000000000000000000000000000000000020100").unwrap(),
        };
        let wrapped_msg_correct = Message::AnnounceSignatures(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }
}
