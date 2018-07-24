pub mod types;
pub mod setup;
pub mod channel;
pub mod control;

use self::setup::Init;
use self::setup::Error;
use self::control::Ping;
use self::control::Pong;
use self::channel::*;

use serde::Serialize;
use serde::Serializer;
use serde::Deserialize;
use serde::Deserializer;
use serde::de::SeqAccess;
use serde::ser::SerializeStruct;

macro_rules! message {
    (pub enum $name:ident { $($variant:ident($rtt:expr)),* }) => {
        #[derive(Eq, PartialEq, Debug)]
        pub enum $name {
            $($variant($variant),)*
        }

        impl $name {
            pub fn new<'de, A>(runtime_type: u16, payload: A) -> Result<Option<Self>, A::Error> where A: SeqAccess<'de> {
                let mut payload = payload;
                use self::$name::*;
                match runtime_type {
                    $($rtt => payload.next_element().map(|i| i.map(|x| Init(x))),)*
                    _ => Ok(None),
                }
            }

            pub fn runtime_type(&self) -> u16 {
                use self::$name::*;
                match self {
                    $(&$variant(_) => $rtt,)*
                }
            }

            pub fn consume<A>(&self, consumer: &mut A) -> Result<(), A::Error> where A: SerializeStruct {
                use self::$name::*;
                match self {
                    $(&$variant(ref payload) => consumer.serialize_field("payload", payload),)*
                }
            }
        }
    }
}

message! {
    pub enum Message {
        Init(16),
        Error(17),
        Ping(18),
        Pong(19),
        OpenChannel(32),
        AcceptChannel(33),
        FundingCreated(34),
        FundingSigned(35),
        FundingLocked(36),
        ShutdownChannel(38),
        ClosingNegotiation(39),
        UpdateAddHtlc(128),
        UpdateFulfillHtlc(130),
        UpdateFailHtlc(131),
        UpdateFailMalformedHtlc(135)
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        use serde::ser::SerializeStruct;

        // The names provided only for documentation, serializer drops it
        let mut s_struct = serializer.serialize_struct("Message", 2)?;
        s_struct.serialize_field("type", &self.runtime_type())?;
        self.consume(&mut s_struct)?;
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

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error> where A: de::SeqAccess<'de>,
            {
                let mut seq = seq;
                let type_ = seq.next_element()?
                    .ok_or(<A::Error as de::Error>::custom("message type expected"))?;
                Message::new(type_, seq)?
                    .ok_or(<A::Error as de::Error>::custom(""))
            }
        }

        deserializer.deserialize_tuple(2, Visitor)
    }
}
