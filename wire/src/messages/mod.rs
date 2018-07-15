use super::feature::RawFeatureVector;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
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

#[cfg(test)]
mod test {
    use bincode;
    use serde::Deserialize;
    use serde::de::Deserializer;
    use serde::Serialize;
    use serde::ser::Serializer;
    use std::result;
    use bincode::Result;

    use super::Init;
    use ::feature::RawFeatureVector;
    use ::feature::FeatureBit;

    #[derive(Copy, Clone)]
    struct LengthSD;

    impl bincode::LengthSDOptions for LengthSD {
        fn serialized_length_size(&self, length: u64) -> Result<usize> {
            let _ = length;
            Ok(2)
        }

        fn serialize_length<S: Serializer>(&self, s: S, length: usize) -> result::Result<S::Ok, S::Error> {
            let length = length as u16;
            Serialize::serialize(&length, s)
        }

        fn deserialize_length<'de, D: Deserializer<'de>>(&self, d: D) -> result::Result<usize, D::Error> {
            Deserialize::deserialize(d).map(|l: u16| l as _)
        }
    }

    #[test]
    fn test_init_serde() {
        use self::FeatureBit::*;

        let mut temp = bincode::config();
        let bc_config = temp.big_endian();

        let init = Init {
            local_features: RawFeatureVector::new()
                .set_bit(DataLossProtectRequired),
            global_features: RawFeatureVector::new()
                .set_bit(DataLossProtectOptional)
                .set_bit(GossipQueriesOptional),
        };

        let mut data = vec![];
        bc_config.serialize_custom_length_into(&mut data, &init, LengthSD).unwrap();

        println!("{:?}", data);
        let new = bc_config.deserialize_custom_length_from(&data[..], LengthSD).unwrap();

        assert_eq!(init, new);
    }
}
