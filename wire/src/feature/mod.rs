use std::collections::HashSet;
use std::collections::HashMap;
use std::fmt;

use serde::Serialize;
use serde::Serializer;
use serde::Deserialize;
use serde::Deserializer;
use serde::de;

mod feature_bit;
pub use self::feature_bit::FeatureBit;

#[derive(Clone, Debug)]
pub struct RawFeatureVector {
    set: HashSet<FeatureBit>,
}

impl RawFeatureVector {
    pub const BITS: u16 = 8;

    pub fn with_serialize_size(serialize_size: u16) -> Self {
        RawFeatureVector {
            set: HashSet::with_capacity((serialize_size * Self::BITS) as _),
        }
    }

    pub fn serialize_size(&self) -> Option<usize> {
        self
            .set
            .iter()
            .max()
            .map(|b| ((u16::from(b.clone())) / Self::BITS) as usize)
    }
}

impl Serialize for RawFeatureVector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        use serde::ser::SerializeSeq;

        match self.serialize_size() {
            Some(length) => {
                let length_u16 = length as u16;

                let seq = serializer
                    .serialize_seq(Some(length))
                    .and_then(|mut s|
                        s.serialize_element(&((length_u16 >> Self::BITS) as u8)).map(|_| s)
                    )
                    .and_then(|mut s|
                        s.serialize_element(&(length_u16 as u8)).map(|_| s)
                    );

                // fold the seq structure with each byte index
                (0..length_u16)
                    .fold(seq, |seq, byte_index| {
                        match byte_index {
                            _ => {
                                // for each bit in byte check
                                // if corresponding feature_bit is contained in the set
                                // and accumulate the presence in the byte
                                let byte = (0..Self::BITS)
                                    .map(|bit_index| {
                                        let global_bit_index = byte_index * Self::BITS + bit_index;
                                        let feature_bit = global_bit_index.into();
                                        let bit = self.set.contains(&feature_bit);
                                        if bit { 1u8 } else { 0u8 }
                                    })
                                    .fold(0u8, |acc, is_contained| acc << 1 | is_contained);

                                // serialize the obtained byte
                                seq.and_then(|mut s|
                                    s.serialize_element(&byte).map(|_| s)
                                )
                            }
                        }
                    })
                    .and_then(|s| s.end())
            },
            None => serializer.serialize_unit()
        }
    }
}

impl From<Vec<u8>> for RawFeatureVector {
    fn from(stuff: Vec<u8>) -> Self {
        let len = stuff.len();
        stuff
            .into_iter()
            .enumerate()
            .fold(RawFeatureVector::with_serialize_size(len as _), |fv, (byte_index, byte): (usize, u8)| {
                (0..Self::BITS).fold(fv, |mut fv, bit_index| {
                    let global_bit_index = (byte_index as u16) * RawFeatureVector::BITS + bit_index;
                    let feature_bit = global_bit_index.into();
                    let bit = (byte & ((1 << bit_index) as u8)) != 0;
                    if bit {
                        fv.set.insert(feature_bit);
                    }
                    fv
                })
            })

    }
}

#[derive(Default)]
struct BytesVisitor;

impl<'de> de::Visitor<'de> for BytesVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("s")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: de::Error {
        // explicitly big endian
        assert!(v.len() >= 2);
        let a = v[0] as u16;
        let b = v[1] as u16;
        let length = ((a << RawFeatureVector::BITS) | b) as usize;
        assert_eq!(v.len(), 2 + length);
        let mut vec = Vec::with_capacity(length);
        vec.copy_from_slice(&v[2..]);
        Ok(vec)
    }
}

impl<'de> Deserialize<'de> for RawFeatureVector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_bytes(BytesVisitor::default())
            .map(|bytes| bytes.into())
    }
}

pub struct FeatureVector {
    raw: RawFeatureVector,
    names: HashMap<FeatureBit, String>,
}
