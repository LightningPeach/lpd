mod announcement;
mod alias;

pub use self::announcement::AnnouncementNodeData;
pub use self::announcement::AnnouncementNode;
pub use self::alias::NodeAlias;

use super::types::*;

use std::ops::Range;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct NodePort(pub u16);

impl NodePort {
    pub fn range() -> Range<Self> {
        NodePort(1024)..NodePort(49151)
    }
}
