use super::ShortChannelId;
use super::Hash256;
use ::UncompressedData;
use ::SerdeVec;

#[derive(Eq, PartialEq, Debug)]
pub enum ShortChannelIdEncoding {
    StoredPlain(SerdeVec<ShortChannelId>),
    StoredZlib(UncompressedData<ShortChannelId>),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct QueryShortChannelIds {
    chain_hash: Hash256,
    ids: ShortChannelIdEncoding,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct ReplyShortChannelIdsEnd {
    chain_hash: Hash256,
    complete: bool,
}

mod serde {
    use super::ShortChannelIdEncoding;

    use serde::Serialize;
    use serde::Serializer;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::ser;
    use serde::de;

    impl Serialize for ShortChannelIdEncoding {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
            use self::ser::SerializeTuple;
            use self::ShortChannelIdEncoding::*;

            let mut tuple = serializer.serialize_tuple(2)?;
            match self {
                &StoredPlain(ref plain) => {
                    let _ = SerializeTuple::serialize_element(&mut tuple, &0u8)?;
                    let _ = SerializeTuple::serialize_element(&mut tuple, plain)?;
                },
                &StoredZlib(ref to_compress) => {
                    let _ = SerializeTuple::serialize_element(&mut tuple, &1u8)?;
                    let _ = SerializeTuple::serialize_element(&mut tuple, to_compress)?;
                }
            }

            SerializeTuple::end(tuple)
        }
    }

    impl<'de> Deserialize<'de> for ShortChannelIdEncoding {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            use std::fmt;
            use self::de::SeqAccess;

            struct Visitor;

            impl<'de> de::Visitor<'de> for Visitor {
                type Value = ShortChannelIdEncoding;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "one byte ShortChannelIdEncoding \
                                       and compressed or uncompressed data")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de>, {
                    use self::ShortChannelIdEncoding::*;

                    let mut seq = seq;
                    let header: u8 = seq.next_element()?
                        .ok_or(<A::Error as de::Error>::custom(format!("expected ShortChannelIdEncoding byte")))?;
                    match header {
                        0 => {
                            seq.next_element()?
                                .ok_or(<A::Error as de::Error>::custom(format!("expected plain ShortChannelId list")))
                                .map(|v| StoredPlain(v))
                        },
                        1 => {
                            seq.next_element()?
                                .ok_or(<A::Error as de::Error>::custom(format!("expected zlib compressed ShortChannelId list")))
                                .map(|v| StoredPlain(v))
                        },
                        _ => Err(<A::Error as de::Error>::custom(format!("unknown compression type")))
                    }
                }
            }

            deserializer.deserialize_tuple(2, Visitor)
        }
    }
}
