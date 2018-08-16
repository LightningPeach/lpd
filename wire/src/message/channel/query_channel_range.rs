use super::Hash256;
use super::ShortChannelIdEncoding;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct QueryChannelRange {
    chain_hash: Hash256,
    first_block_height: u32,
    number_of_blocks: u32,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct ReplyChannelRange {
    chain_hash: Hash256,
    first_block_height: u32,
    number_of_blocks: u32,
    complete: bool,
    encoded_short_ids: ShortChannelIdEncoding,
}
