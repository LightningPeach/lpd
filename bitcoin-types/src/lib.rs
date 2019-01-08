#![forbid(unsafe_code)]

use common_types::*;

mod output_index;
pub use self::output_index::OutputIndex;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct ChannelPoint {
    hash: Hash256,
    index: u32,
}
