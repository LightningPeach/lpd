use flate2::Compression;
use flate2::read;

use serde::Serialize;
use serde::Serializer;
use serde::Deserialize;
use serde::Deserializer;
use serde::ser;
use serde::de;

use ::BinarySD;
use std::io::Read;

use std::mem;

pub trait PackSized: Sized {
    const SIZE: usize = mem::size_of::<Self>();

    fn pack_size(&self) -> usize {
        Self::SIZE
    }
}

/// write size in byte rather then length of the vector
#[derive(Eq, PartialEq, Debug)]
pub struct SerdeVec<T>(pub Vec<T>) where T: PackSized;

impl<T> Serialize for SerdeVec<T> where T: PackSized + Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        use self::ser::SerializeTuple;

        let &SerdeVec(ref data) = self;
        let mut tuple = serializer.serialize_tuple(2 + data.len())?;
        let size_in_bytes = if T::SIZE == 0 {
            data.iter()
                .fold(0, |accumulator, item| accumulator + item.pack_size())
        } else {
            T::SIZE * data.len()
        };
        let _ = SerializeTuple::serialize_element(&mut tuple, &size_in_bytes)?;
        for item in data {
            let _ = SerializeTuple::serialize_element(&mut tuple, item)?;
        }

        SerializeTuple::end(tuple)
    }
}

impl<'de, T> Deserialize<'de> for SerdeVec<T> where T: PackSized + de::DeserializeOwned {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        use std::fmt;
        use std::marker::PhantomData;
        use self::de::SeqAccess;

        struct Visitor<T> where T: PackSized + de::DeserializeOwned {
            phantom_data: PhantomData<T>,
        }

        impl<'de, T> de::Visitor<'de> for Visitor<T> where T: PackSized + de::DeserializeOwned {
            type Value = SerdeVec<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "length in bytes and the bytes")
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de>, {
                let mut seq = seq;

                let mut size = seq.size_hint()
                    .ok_or(<A::Error as de::Error>::custom(format!("expected size")))?;
                let mut data = Vec::new();
                loop {
                    if size == 0 { break; }
                    let element: T = seq.next_element()?
                        .ok_or(<A::Error as de::Error>::custom(format!("cannot read T")))?;
                    if size == element.pack_size() {
                        break;
                    } else {
                        if size < element.pack_size() {
                            return Err(<A::Error as de::Error>::custom(format!("cannot assemble integer amount of T")))
                        }
                    }
                    size -= element.pack_size();
                    data.push(element);
                }
                Ok(SerdeVec(data))
            }

        }

        deserializer.deserialize_seq(Visitor {
            phantom_data: PhantomData,
        })
    }
}

/// the underlying data is uncompressed and deserialized into rust type,
/// but serialization / deserialization will read / write zlib compressed data
/// this is exactly desired by lnd specification
#[derive(Eq, PartialEq, Debug)]
pub struct UncompressedData<T>(pub SerdeVec<T>) where T: PackSized;

impl<T> Serialize for UncompressedData<T> where T: PackSized + Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut bytes = Vec::<u8>::new();
        BinarySD::serialize(&mut bytes, self)
            .map_err(|e| <S::Error as ser::Error>::custom(format!("serialize error: {:?}", e)))?;
        let mut encoder = read::ZlibEncoder::new(bytes.as_slice(), Compression::fast());
        let mut compressed_bytes = Vec::<u8>::new();
        let _ = Read::read_to_end(&mut encoder, &mut compressed_bytes)
            .map_err(|e| <S::Error as ser::Error>::custom(format!("compression error: {:?}", e)))?;
        serializer.serialize_bytes(compressed_bytes.as_slice())
    }
}

impl<'de, T> Deserialize<'de> for UncompressedData<T> where T: PackSized + de::DeserializeOwned {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        use std::fmt;
        use std::marker::PhantomData;

        struct Visitor<T> where T: de::DeserializeOwned {
            phantom_data: PhantomData<T>,
        }

        impl<'de, T> de::Visitor<'de> for Visitor<T> where T: PackSized + de::DeserializeOwned {
            type Value = UncompressedData<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "zlib compressed data")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: de::Error, {
                let mut decoder = read::ZlibDecoder::new(v);
                let mut decompressed_bytes = Vec::<u8>::new();
                let _ = Read::read_to_end(&mut decoder, &mut decompressed_bytes)
                    .map_err(|e| E::custom(format!("decompression error: {:?}", e)))?;
                BinarySD::deserialize(decompressed_bytes.as_slice())
                    .map_err(|e| E::custom(format!("deserialize error: {:?}", e)))
            }
        }

        deserializer.deserialize_bytes(Visitor {
            phantom_data: PhantomData,
        })
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn compression_test() {
        // TODO:
        assert!(true)
    }
}
