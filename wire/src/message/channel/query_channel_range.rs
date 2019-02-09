use super::Hash256;
use super::ShortChannelIdEncoding;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct QueryChannelRange {
    chain_hash: Hash256,
    first_block_height: u32,
    number_of_blocks: u32,
}

impl QueryChannelRange {
    pub fn new(chain_hash: Hash256, first_block_height: u32, number_of_blocks: u32) -> Self {
        QueryChannelRange {
            chain_hash: chain_hash,
            first_block_height: first_block_height,
            number_of_blocks: number_of_blocks,
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ReplyChannelRange {
    chain_hash: Hash256,
    first_block_height: u32,
    number_of_blocks: u32,
    complete: bool,
    encoded_short_ids: ShortChannelIdEncoding,
}

#[cfg(test)]
mod tests {
    use binformat::BinarySD;
    use super::*;

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
}
