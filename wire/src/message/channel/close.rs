use super::ChannelId;
use super::Satoshi;
use super::super::types::RawSignature;

use serde_derive::{Serialize, Deserialize};

/// Either node (or both) can send a shutdown message to initiate closing,
/// along with the scriptpubkey it wants to be paid to.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ShutdownChannel {
    pub channel_id: ChannelId,
    pub script: Vec<u8>,
}

/// Once shutdown is complete and the channel is empty of HTLCs,
/// the final current commitment transactions will have no HTLCs,
/// and closing fee negotiation begins. The funder chooses a fee it thinks is fair,
/// and signs the closing transaction with the `scriptpubkey` fields
/// from the shutdown messages (along with its chosen fee) and sends the signature;
/// the other node then replies similarly, using a fee it thinks is fair.
/// This exchange continues until both agree on the same fee or when one side fails the channel.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ClosingSigned {
    pub channel_id: ChannelId,
    /// less than or equal to the base fee of the final commitment transaction,
    /// as calculated in BOLT #3
    pub fee: Satoshi,
    /// the Bitcoin signature of the close transaction, as specified in BOLT #3
    pub signature: RawSignature,
}

#[cfg(test)]
mod test{
    use dependencies::hex;
    use dependencies::pretty_assertions;

    use crate::ClosingSigned;

    use super::*;
    use binformat::BinarySD;
    use crate::message::channel::ChannelId;
    use std::io::Cursor;
    use crate::Message;
    use pretty_assertions::assert_eq;

    #[test]
    fn closing_signed_test() {
        let msg_hex = "\
            00270100000000000000000000000000000000000000000000000000000000000000000000000000\
            007b0002000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000";
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

