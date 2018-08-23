#![forbid(unsafe_code)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rand;
extern crate sha2;

extern crate common_types;

use common_types::*;

mod output_index;
pub use self::output_index::OutputIndex;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct ChannelPoint {
    hash: Hash256,
    index: u32,
}
