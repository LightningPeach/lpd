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

use std::u16;

pub type MessageSize = u16;

macro_rules! message {
    (pub enum $name:ident { $($variant:ident($rtt:expr)),* }) => {
        /// Tagged union, the variant name equals to the type name witch the variant contains
        #[derive(Eq, PartialEq, Debug)]
        pub enum $name {
            $($variant($variant),)*
        }

        impl $name {
            pub const SIZE_LIMIT: usize = (u16::MAX as usize) - 2;

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
        Init(16u16),
        Error(17u16),
        Ping(18u16),
        Pong(19u16),
        OpenChannel(32u16),
        AcceptChannel(33u16),
        FundingCreated(34u16),
        FundingSigned(35u16),
        FundingLocked(36u16),
        ShutdownChannel(38u16),
        ClosingNegotiation(39u16),
        UpdateAddHtlc(128u16),
        UpdateFulfillHtlc(130u16),
        UpdateFailHtlc(131u16),
        UpdateFailMalformedHtlc(135u16),
        CommitmentSigned(132u16),
        RevokeAndAck(133u16),
        UpdateFee(134u16),
        ReestablishChannel(136u16)
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

#[cfg(test)]
mod tests {
    use ::BinarySD;
    use super::*;

    use hex;

    #[test]
    fn deserialize_init() {
        let data = vec![0, 16, 0, 0, 0, 1, 138];

        let msg: Result<Message, _> = BinarySD::deserialize(&data[..]);
        println!("{:?}", msg);
        assert!(msg.is_ok());
    }

    #[test]
    fn foo(){
        let msg_bytes = hex::decode("002006226e46111a0b59caaf126043eb5bbf28c34f3a5e332a1fc7b2b73cf188910f3283054b8d351cfd58a790cb502069a64c40e226a0d228eae7e83e316dd2791700000000000186a00000000000000000000000000000023d0000000005e69ec000000000000003e800000000000003e8000030d4009001e302d254a0bc14d7c990d4c40e806bcaffc022ead28ba88eaa5450ef90565119020102859c2c7c7c0495198371dc0cb1329fdeca223972aeb089af2895c33b180cc6a20265ae921bd8cd25b7c62eda488b0f87955b3df36ccdb72cb0c75336cc8d5dc7030363b7cea6090e2f78a67a29a7cc5b351695a0dc6c0f2bbf14dc9098ed6074a3230213f314dcc6dbdaea4fac352277f55d53f873901477d80b8d2da794b411e5102202e19840efe9d300361f2624dfb5516f254bc6381be106c85ba0d3c429a54166c201").unwrap();
        let restored: Result<Message, _> = BinarySD::deserialize(msg_bytes.as_slice());
        assert!(restored.is_ok());
    }
}
