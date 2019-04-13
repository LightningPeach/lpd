use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageInfo {
    // hex-encoded message
    pub msg_raw: String,

    // hex-encoded pubkey of peer (in compressed format)
    pub peer_pubkey: String,

    // sent or received
    pub direction: String,

    #[serde(rename = "type")]
    pub type_ : String,

    // Unix timestamp
    pub time: String,
}