#![forbid(unsafe_code)]

use serde_derive::{Serialize, Deserialize};

use common_types::*;

mod output_index;
pub use self::output_index::OutputIndex;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct ChannelPoint {
    hash: Hash256,
    index: u32,
}
