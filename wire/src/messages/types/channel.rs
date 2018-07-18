
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct ChannelId {
    raw: [u8; 32],
}

impl ChannelId {
    pub fn all() -> Self {
        ChannelId {
            raw: [0; 32],
        }
    }
}
