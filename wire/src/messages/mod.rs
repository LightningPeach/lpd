pub mod types;
pub mod setup;
pub mod channel;
pub mod control;

use self::setup::Init;
use self::setup::Error;
use self::control::Ping;
use self::control::Pong;

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

        // The names provided only for documentation, serializer drops it
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
