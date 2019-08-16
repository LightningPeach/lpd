use serde_derive::{Serialize, Deserialize};
use std::error::Error;

use dependencies::hex;

const SIZE: usize = 32;

// NodeAlias may contain not UTF8 string, so it is better not to use string here
// because string MUST contain valid UTF8
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct NodeAlias([u8; SIZE]);

impl NodeAlias {
    pub fn string(self) -> String {
        String::from_utf8_lossy(&self.0[..]).to_string()
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0[..])
    }

    pub fn from_hex(s: &str) -> Result<NodeAlias, Box<dyn Error>> {
        use crate::u8_32_from_hex;

        let data = u8_32_from_hex(s)
            .map_err(|err| format!("cannot decode NodeAlias from hex: {:?}", err))?;
        Ok(NodeAlias(data))
    }
}
