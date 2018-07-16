use super::feature::RawFeatureVector;

use serde::Serialize;
use serde::Serializer;
use serde::Deserialize;
use serde::Deserializer;

#[derive(Eq, PartialEq, Debug)]
pub enum Message {
    Init(Init),
    Error(Error),
    Ping(Ping),
    Pong(Pong),
}

impl Message {
    pub fn type_(&self) -> u16 {
        use self::Message::*;
        match self {
            &Init(_) => 16,
            &Error(_) => 17,
            &Ping(_) => 18,
            &Pong(_) => 19,
        }
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        use serde::ser::SerializeStruct;
        use self::Message::*;

        // The names provided only foral documentation, serializer drops it
        let mut s_struct = serializer.serialize_struct("Message", 2)?;
        s_struct.serialize_field("type", &self.type_())?;
        match self {
            &Init(ref payload) => s_struct.serialize_field("payload", payload),
            &Error(ref payload) => s_struct.serialize_field("payload", payload),
            &Ping(ref payload) => s_struct.serialize_field("payload", payload),
            &Pong(ref payload) => s_struct.serialize_field("payload", payload),
        }?;
        s_struct.end()
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        use serde::de;
        use std::fmt;

        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Message;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("s")
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
                where
                    A: de::MapAccess<'de>,
            {
                let _ = map;
                unimplemented!()
            }
        }

        deserializer.deserialize_struct("Message", &["type", "payload"], Visitor)
    }
}

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

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Error {
    channel_id: ChannelId,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Ping {
    pong_length: u16,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Pong {
    data: Vec<u8>,
}

#[cfg(test)]
mod test {
    use ::serde_facade::BinarySD;

    use super::Init;
    use ::feature::RawFeatureVector;
    use ::feature::FeatureBit;

    #[test]
    fn test_init_serde() {
        use self::FeatureBit::*;

        let init = Init {
            local_features: RawFeatureVector::new()
                .set_bit(DataLossProtectRequired),
            global_features: RawFeatureVector::new()
                .set_bit(DataLossProtectOptional)
                .set_bit(GossipQueriesOptional),
        };

        let mut data = Vec::<u8>::new();
        BinarySD::serialize(&mut data, &init).unwrap();

        println!("{:?}", data);
        let new = BinarySD::deserialize(&data[..]).unwrap();

        assert_eq!(init, new);
    }
}
