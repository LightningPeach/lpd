use super::feature::RawFeatureVector;

#[derive(Serialize, Deserialize)]
pub struct Init {
    global_features: RawFeatureVector,
    local_features: RawFeatureVector,
}

impl Init {
    pub fn new(global_features: RawFeatureVector, local_features: RawFeatureVector) -> Self {
        Init {
            global_features: global_features as _,
            local_features: local_features as _,
        }
    }
}

#[derive(Serialize, Deserialize)]
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
