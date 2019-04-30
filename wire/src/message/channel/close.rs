use super::ChannelId;
use super::Satoshi;
use super::super::types::RawSignature;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ShutdownChannel {
    pub channel_id: ChannelId,
    pub script: Vec<u8>,
}


#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ClosingSigned {
    pub channel_id: ChannelId,
    pub fee: Satoshi,
    pub signature: RawSignature,
}

#[cfg(test)]
mod test{
    use crate::ClosingSigned;

    use super::*;
    use binformat::BinarySD;
    use crate::message::channel::ChannelId;
    use crate::message::channel::operation::{UpdateFulfillHtlc, HtlcId, u8_32_from_hex};
    use crate::CsvDelay;
    use std::io::{Cursor, Read, Seek, SeekFrom};
    use crate::Message;
    use pretty_assertions::{assert_eq, assert_ne};
    use secp256k1::PublicKey;


    #[test]
    fn closing_signed_test() {
        let msg_hex = "00270100000000000000000000000000000000000000000000000000000000000000000000000000007b00020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = ClosingSigned {
            channel_id: ChannelId::from_hex("0100000000000000000000000000000000000000000000000000000000000000").unwrap(),
            fee: Satoshi::from(123),
            signature: RawSignature::from_hex("3024021f02000000000000000000000000000000000000000000000000000000000000020100").unwrap(),
        };
        let wrapped_msg_correct = Message::ClosingSigned(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }

//    hex: 0026010000000000000000000000000000000000000000000000000000000000000000050102030405
//    ChannelID: 0100000000000000000000000000000000000000000000000000000000000000
//    Address: 0102030405
    #[test]
    fn shutdown_test() {
        let msg_hex = "0026010000000000000000000000000000000000000000000000000000000000000000050102030405";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = ShutdownChannel {
            channel_id: ChannelId::from_hex("0100000000000000000000000000000000000000000000000000000000000000").unwrap(),
            script: hex::decode("0102030405").unwrap(),
        };
        let wrapped_msg_correct = Message::ShutdownChannel(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }
}

