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
use serde::ser;
use serde::de;

macro_rules! message {
    (pub enum $name:ident { $($variant:ident($rtt:expr)),* }) => {
        /// Tagged union, the variant name equals to the type name witch the variant contains
        #[derive(Eq, PartialEq, Debug)]
        pub enum $name {
            $($variant($variant),)*
        }

        impl $name {
            fn read_from<'de, A>(payload: A) -> Result<Self, A::Error> where
                A: de::SeqAccess<'de>
            {
                let mut payload = payload;

                let notype_err = {
                    let temp = "cannot read 16-bit runtime type information of the message";
                    <A::Error as de::Error>::custom(temp)
                };

                let novalue_err = |s| {
                    let temp = format!("cannot interpret the value as an instance of: `{}`", s);
                    <A::Error as de::Error>::custom(temp)
                };

                let unknown_err = |t| {
                    let temp = format!("runtime type: `{}` is unknown", t);
                    <A::Error as de::Error>::custom(temp)
                };

                let runtime_type = payload.next_element()?.ok_or(notype_err)?;

                use self::$name::*;
                match runtime_type {
                    $(
                        $rtt => {
                            let error = novalue_err(stringify!($variant));
                            payload.next_element()
                                .and_then(|i| {
                                    i.ok_or(error).map(|x| $variant(x))
                                })
                        },
                    )*
                    t @ _ => Err(unknown_err(t)),
                }
            }

            fn write_into<A>(&self, consumer: &mut A) -> Result<(), A::Error> where
                A: ser::SerializeStruct
            {
                use self::$name::*;
                match self {
                    $(
                        &$variant(ref payload) => {
                            consumer.serialize_field("type", &$rtt)?;
                            consumer.serialize_field("payload", payload)
                        },
                    )*
                }
            }
        }
    }
}

/// Main data structure of the crate.
/// The enumeration contains all possible messages of the network.
/// Implements `Eq`, `Debug`, `Serialize`, `Deserialize`
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
        UpdateFailMalformedHtlc(135),
        CommitmentSigned(132),
        RevokeAndAck(133),
        UpdateFee(134),
        ReestablishChannel(136)
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        use self::ser::SerializeStruct;

        // The names provided only for documentation, serializer drops it
        let mut s_struct = serializer.serialize_struct("Message", 2)?;
        self.write_into(&mut s_struct)?;
        s_struct.end()
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        use std::fmt;

        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Message;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "pair: 16-bit runtime type information, \
                    the binary representation of the message"
                )
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error> where
                A: de::SeqAccess<'de>,
            {
                Message::read_from(seq)
            }
        }

        deserializer.deserialize_tuple(2, Visitor)
    }
}
