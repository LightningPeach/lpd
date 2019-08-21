use super::Sha256;
use super::ShortChannelIdEncoding;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct QueryChannelRange {
    pub chain_hash: Sha256,
    pub first_block_height: u32,
    pub number_of_blocks: u32,
}

impl QueryChannelRange {
    pub fn new(chain_hash: Sha256, first_block_height: u32, number_of_blocks: u32) -> Self {
        QueryChannelRange {
            chain_hash: chain_hash,
            first_block_height: first_block_height,
            number_of_blocks: number_of_blocks,
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ReplyChannelRange {
    pub chain_hash: Sha256,
    pub first_block_height: u32,
    pub number_of_blocks: u32,
    pub complete: bool,
    pub encoded_short_ids: ShortChannelIdEncoding,
}

#[cfg(test)]
mod tests {
    use dependencies::hex;
    use dependencies::pretty_assertions;

    use binformat::BinarySD;
    use super::*;

    use std::io::Cursor;
    use crate::Message;
    use pretty_assertions::assert_eq;

    #[test]
    fn reply_channel_range() {
        let v = vec![
            104, 62, 134, 189, 92, 109, 17, 13, 145, 185, 75, 151, 19, 123, 166, 191,
            224, 45, 187, 219, 142, 61, 255, 114, 42, 102, 155, 93, 105, 215, 122, 246,
            0, 0, 0, 0,
            0, 0, 0, 120,
            1,
            0, 1,
            0
        ];
        let t: ReplyChannelRange = BinarySD::deserialize(&v[..]).unwrap();
        println!("{:?}", t);
    }


    #[test]
    fn test_query_channel_range() {
        let msg_hex = "010700000b0000000000000000000000000000000000000000000000000000000000000027100000000c";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = QueryChannelRange {
            chain_hash: Sha256::from_hex("00000b0000000000000000000000000000000000000000000000000000000000").unwrap(),
            first_block_height: 10000,
            number_of_blocks: 12,
        };
        let wrapped_msg_correct = Message::QueryChannelRange(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }

//    hex: 010800000b0000000000000000000000000000000000000000000000000000000000000027100000000c01001900002fbd000001000a003a9800000c0065030d40000005000b
//    ChainHash: 00000b0000000000000000000000000000000000000000000000000000000000
//    FirstBlockHeight: 10000
//    NumBlocks: 12
//    Complete: 1
//    EncodingType: 0
//    ShortChanIDs: 3
//    13437131603116042
//    16492674417426533
//    219902325555527691
    #[test]
    fn reply_channel_range_plain_test() {
        let msg_hex = "010800000b0000000000000000000000000000000000000000000000000000000000000027100000000c01001900002fbd000001000a003a9800000c0065030d40000005000b";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = ReplyChannelRange {
            chain_hash: Sha256::from_hex("00000b0000000000000000000000000000000000000000000000000000000000").unwrap(),
            first_block_height: 10000,
            number_of_blocks: 12,
            complete: 1 != 0,
            encoded_short_ids: ShortChannelIdEncoding::from_u64_vec(0, &vec![
                13437131603116042,
                16492674417426533,
                219902325555527691,
            ]).unwrap(),
        };
        let wrapped_msg_correct = Message::ReplyChannelRange(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }


    // Seems not working
    //    hex: 010800000b0000000000000000000000000000000000000000000000000000000000000027100000000c01002501789c62d0dfcbc0c0c8c0c5603583818187219599d781818195811b100000ffff2720029b
    //    ChainHash: 00000b0000000000000000000000000000000000000000000000000000000000
    //    FirstBlockHeight: 10000
    //    NumBlocks: 12
    //    Complete: 1
    //    EncodingType: 1
    //    ShortChanIDs: 3
    //    13437131603116042
    //    16492674417426533
    //    219902325555527691
    #[test]
    fn reply_channel_range_zlib_test() {
        let msg_hex = "\
            010800000b0000000000000000000000000000000000000000000000000000000000000027100000\
            000c01002501789c62d0dfcbc0c0c8c0c5603583818187219599d781818195811b100000ffff2720\
            029b";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = ReplyChannelRange {
            chain_hash: Sha256::from_hex("00000b0000000000000000000000000000000000000000000000000000000000").unwrap(),
            first_block_height: 10000,
            number_of_blocks: 12,
            complete: 1 != 0,
            encoded_short_ids: ShortChannelIdEncoding::from_u64_vec(1, &vec![
                13437131603116042,
                16492674417426533,
                219902325555527691,
            ]).unwrap(),
        };
        let wrapped_msg_correct = Message::ReplyChannelRange(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // It seems that serialisation in lnd and rust are slightly different
        // they can read each other but byte result is different
        // let mut new_msg_bytes = vec![];
        // BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        // assert_eq!(new_msg_bytes, msg_bytes);
    }
}
