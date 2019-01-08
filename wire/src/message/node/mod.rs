mod announcement;
pub use self::announcement::*;

mod alias;
pub use self::alias::*;

use super::types::*;
use super::Signed;
use super::SignedData;

use std::ops::Range;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct NodePort(pub u16);

impl NodePort {
    pub fn range() -> Range<Self> {
        NodePort(1024)..NodePort(49151)
    }
}
