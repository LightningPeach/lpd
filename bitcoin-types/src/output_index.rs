use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
pub struct OutputIndex {
    index: u16,
}

impl From<OutputIndex> for u16 {
    fn from(x: OutputIndex) -> Self {
        return x.index;
    }
}
