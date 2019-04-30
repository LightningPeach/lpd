use std::error::Error;
use super::ShortChannelId;
use super::Hash256;
use binformat::UncompressedData;
use binformat::SerdeVec;

use serde_derive::{Serialize, Deserialize};

// TODO(mkl): maybe change this to plain storage format
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ShortChannelIdEncoding {
    StoredPlain(SerdeVec<ShortChannelId>),
    StoredZlib(UncompressedData<ShortChannelId>),
}

impl ShortChannelIdEncoding {
    pub fn from_u64_vec(encoding: u8, ids: &[u64]) -> Result<ShortChannelIdEncoding, Box<Error>> {
        use ShortChannelIdEncoding::*;
        match encoding {
            0 => {
                let data: Vec<ShortChannelId> = ids.iter().map(|x| ShortChannelId::from_u64(*x)).collect();
                Ok(StoredPlain(SerdeVec(data)))
            },
            1 => {
                let data: Vec<ShortChannelId> = ids.iter().map(|x| ShortChannelId::from_u64(*x)).collect();
                Ok(StoredZlib(UncompressedData(SerdeVec(data))))
            },
            _ => Err(format!("unknown  ShortChannelIdEncoding: {}", encoding).into())
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct QueryShortChannelIds {
    pub chain_hash: Hash256,
    pub ids: ShortChannelIdEncoding,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ReplyShortChannelIdsEnd {
    pub chain_hash: Hash256,
    pub complete: bool,
}

mod serde_m {
    use binformat::BinarySD;
    use super::ShortChannelIdEncoding;
    use super::ShortChannelId;
    use binformat::SerdeVec;
    use binformat::PackSized;

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

#[cfg(test)]
mod test {
    use super::*;

    use binformat::BinarySD;
    use crate::message::channel::ChannelId;
    use std::io::{Cursor, Read, Seek, SeekFrom};
    use crate::{Message, RevokeAndAck, RawPublicKey, CommitmentSigned, RawSignature};
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn query_short_channel_ids_plane_test() {
        let msg_hex = "01050006000000000000000000000000000000000000000000000000000000000000001900002fbd000001000a003a9800000c0065030d40000005000b";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = QueryShortChannelIds {
            chain_hash: Hash256::from_hex("0006000000000000000000000000000000000000000000000000000000000000").unwrap(),
            ids: ShortChannelIdEncoding::from_u64_vec(0, &vec![
                13437131603116042,
                16492674417426533,
                219902325555527691,
            ]).unwrap(),
        };
        let wrapped_msg_correct = Message::QueryShortChannelIds(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }


//        hex: 01050006000000000000000000000000000000000000000000000000000000000000002501789c62d0dfcbc0c0c8c0c5603583818187219599d781818195811b100000ffff2720029b
//        ChainHash: 0006000000000000000000000000000000000000000000000000000000000000
//        EncodingType: 1
//        ShortChanIDs: 3
//            13437131603116042
//            16492674417426533
//            219902325555527691
    // Seems not working
    #[test]
    fn query_short_channel_ids_zlib_test() {
        let msg_hex = "01050006000000000000000000000000000000000000000000000000000000000000002501789c62d0dfcbc0c0c8c0c5603583818187219599d781818195811b100000ffff2720029b";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = QueryShortChannelIds {
            chain_hash: Hash256::from_hex("0006000000000000000000000000000000000000000000000000000000000000").unwrap(),
            ids: ShortChannelIdEncoding::from_u64_vec(1, &vec![13437131603116042, 16492674417426533, 219902325555527691,]).unwrap(),
        };
        let wrapped_msg_correct = Message::QueryShortChannelIds(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // It seems that serialisation in lnd and rust are slightly different
        // they can read each other but byte result is different
//        let mut new_msg_bytes = vec![];
//        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
//        println!("new_msg_bytes: {}", hex::encode(&new_msg_bytes));
//        assert_eq!(new_msg_bytes, msg_bytes);
    }

    #[test]
    fn reply_short_channel_ids_end_test() {
        let msg_hex = "0106000700000000000000000000000000000000000000000000000000000000000001";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = ReplyShortChannelIdsEnd {
            chain_hash: Hash256::from_hex("0007000000000000000000000000000000000000000000000000000000000000").unwrap(),
            complete: 1 != 0,
        };
        let wrapped_msg_correct = Message::ReplyShortChannelIdsEnd(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }
}