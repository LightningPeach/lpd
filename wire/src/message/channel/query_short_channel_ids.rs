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
    use ::BinarySD;
    use super::ShortChannelIdEncoding;
    use super::ShortChannelId;
    use ::SerdeVec;
    use ::PackSized;

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

            match self {
                &StoredPlain(ref plain) => {
                    let &SerdeVec(ref data) = plain;
                    let mut tuple = serializer.serialize_tuple(2 + data.len())?;
                    let size = ShortChannelId::SIZE * data.len() + 1; // one byte for header
                    tuple.serialize_element(&(size as u16))?;
                    tuple.serialize_element(&0u8)?;
                    for item in data {
                        tuple.serialize_element(item)?;
                    }
                    tuple.end()
                },
                &StoredZlib(ref to_compress) => {
                    let mut v = Vec::new();
                    let header = 1u8;
                    BinarySD::serialize(&mut v, to_compress)
                        .map_err(|e| <S::Error as ser::Error>::custom(format!("serialize error: {:?}", e)))?;
                    // v looks like [ size_lo, size_hi, b_0, b_1, ..., b_size ]
                    // result should look like [ header, b_0, b_1, ..., b_size ]
                    // let's replace the second byte with header, and ignore the first byte
                    v[1] = header;
                    serializer.serialize_bytes(&v[1..])
                }
            }
        }
    }

    impl<'de> Deserialize<'de> for ShortChannelIdEncoding {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            use std::fmt;

            struct Visitor;

            impl<'de> de::Visitor<'de> for Visitor {
                type Value = ShortChannelIdEncoding;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "one byte ShortChannelIdEncoding \
                                       and compressed or uncompressed data")
                }

                fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E> where E: de::Error, {
                    use self::ShortChannelIdEncoding::*;

                    // TODO: optimize me, get rid of copying
                    let mut v = v;
                    let header = v[0];
                    let size = v.len() - 1;
                    v.remove(0);
                    let mut other = Vec::with_capacity(2 + size);
                    let _ = BinarySD::serialize(&mut other, &(size as u16)).unwrap();
                    other.append(&mut v);

                    match header {
                        0 => Ok(StoredPlain(BinarySD::deserialize(&other[..])
                            .map_err(|e| E::custom(format!("deserialize error: {:?}", e)))?)),
                        1 => Ok(StoredZlib(BinarySD::deserialize(&other[..])
                            .map_err(|e| E::custom(format!("deserialize error: {:?}", e)))?)),
                        _ => Err(E::custom(format!("unknown compression type")))
                    }
                }
            }

            deserializer.deserialize_byte_buf(Visitor)
        }
    }
}
