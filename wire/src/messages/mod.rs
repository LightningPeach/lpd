pub mod types;
pub mod setup;
pub mod channel;
pub mod control;

use self::setup::Init;
use self::setup::Error;
use self::control::Ping;
use self::control::Pong;
use self::channel::OpenChannel;
use self::channel::AcceptChannel;
use self::channel::FundingCreated;
use self::channel::FundingSigned;
use self::channel::FundingLocked;
use self::channel::ShutdownChannel;
use self::channel::ClosingNegotiation;

use serde::Serialize;
use serde::Serializer;
use serde::Deserialize;
use serde::Deserializer;

// TODO: get rid of repeats using macros
#[derive(Eq, PartialEq, Debug)]
pub enum Message {
    Init(Init),
    Error(Error),
    Ping(Ping),
    Pong(Pong),
    OpenChannel(OpenChannel),
    AcceptChannel(AcceptChannel),
    FundingCreated(FundingCreated),
    FundingSigned(FundingSigned),
    FundingLocked(FundingLocked),
    ShutdownChannel(ShutdownChannel),
    ClosingNegotiation(ClosingNegotiation),
}

impl Message {
    pub fn type_(&self) -> u16 {
        use self::Message::*;
        match self {
            &Init(_) => 16,
            &Error(_) => 17,
            &Ping(_) => 18,
            &Pong(_) => 19,
            &OpenChannel(_) => 32,
            &AcceptChannel(_) => 33,
            &FundingCreated(_) => 34,
            &FundingSigned(_) => 35,
            &FundingLocked(_) => 36,
            &ShutdownChannel(_) => 38,
            &ClosingNegotiation(_) => 39,
        }
    }
}

macro_rules! new_message {
    ($type_:expr, $payload:expr) => {
        match $type_ {
            16 => Some(Message::Init($payload)),
            17 => Some(Message::Error($payload)),
            18 => Some(Message::Ping($payload)),
            19 => Some(Message::Pong($payload)),
            32 => Some(Message::OpenChannel($payload)),
            33 => Some(Message::AcceptChannel($payload)),
            34 => Some(Message::FundingCreated($payload)),
            35 => Some(Message::FundingSigned($payload)),
            36 => Some(Message::FundingLocked($payload)),
            38 => Some(Message::ShutdownChannel($payload)),
            39 => Some(Message::ClosingNegotiation($payload)),
            _ => None,
        }
    }
}

macro_rules! consume_from_message {
    ($message:expr, $consume:expr) => {
        match $message {
            &Init(ref payload) => $consume(payload),
            &Error(ref payload) => $consume(payload),
            &Ping(ref payload) => $consume(payload),
            &Pong(ref payload) => $consume(payload),
            &OpenChannel(ref payload) => $consume(payload),
            &AcceptChannel(ref payload) => $consume(payload),
            &FundingCreated(ref payload) => $consume(payload),
            &FundingSigned(ref payload) => $consume(payload),
            &FundingLocked(ref payload) => $consume(payload),
            &ShutdownChannel(ref payload) => $consume(payload),
            &ClosingNegotiation(ref payload) => $consume(payload),
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
        consume_from_message!(self, |p| s_struct.serialize_field("payload", p))?;
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
                new_message!(type_, seq.next_element()?.unwrap())
                    .ok_or(<A::Error as de::Error>::custom(""))
            }
        }

        deserializer.deserialize_tuple(2, Visitor)
    }
}
