use super::Hash256;
use super::ShortChannelIdEncoding;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
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

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct ReplyChannelRange {
    chain_hash: Hash256,
    first_block_height: u32,
    number_of_blocks: u32,
    complete: bool,
    encoded_short_ids: ShortChannelIdEncoding,
}
