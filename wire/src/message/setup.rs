use super::types::RawFeatureVector;
use super::channel::ChannelId;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Error {
    channel_id: ChannelId,
    data: Vec<u8>,
}

#[cfg(test)]
mod test {
    use ::serde_facade::BinarySD;

    use super::Init;
    use super::super::types::RawFeatureVector;
    use super::super::types::FeatureBit;

    #[test]
    fn test_init_serde() {
        use self::FeatureBit::*;

        let init = Init::new(
            RawFeatureVector::new()
                .set_bit(DataLossProtectRequired),
            RawFeatureVector::new()
                .set_bit(DataLossProtectOptional)
                .set_bit(GossipQueriesOptional)
        );

        let mut data = Vec::<u8>::new();
        BinarySD::serialize(&mut data, &init).unwrap();

        println!("{:?}", data);
        let new = BinarySD::deserialize(&data[..]).unwrap();

        assert_eq!(init, new);
    }
}
