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

#[derive(Clone, Debug, Eq, PartialEq)]
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
            .map(|b| (1 + (u16::from(b.clone())) / Self::BITS) as usize)
    }

    pub fn set_bit(self, feature_bit: FeatureBit) -> Self {
        let mut s = self;
        s.set.insert(feature_bit);
        s
    }
}

impl From<Vec<u8>> for RawFeatureVector {
    fn from(stuff: Vec<u8>) -> Self {
        let len = stuff.len();
        let feature_vector = RawFeatureVector::with_serialize_size(len as _);
        stuff
            .into_iter()
            .enumerate()
            .fold(feature_vector, |fv, (i, byte): (usize, u8)| {
                let byte_index = i as u16;
                (0..Self::BITS).fold(fv, |mut fv, bit_index| {
                    let global_bit_index = byte_index * RawFeatureVector::BITS + bit_index;
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

impl From<RawFeatureVector> for Vec<u8> {
    fn from(feature_vector: RawFeatureVector) -> Self {
        match feature_vector.serialize_size() {
            Some(len) => {
                // map each byte index into actual byte
                (0..(len as u16)).map(|byte_index| {
                    // for each bit in byte check
                    // if corresponding feature_bit is contained in the set
                    // and accumulate the presence in the byte
                    (0..RawFeatureVector::BITS)
                        .fold(0u8, |acc, bit_index| {
                            let global_bit_index = byte_index * RawFeatureVector::BITS + bit_index;
                            let feature_bit = FeatureBit::from(global_bit_index);
                            let bit = feature_vector.set.contains(&feature_bit);
                            acc << 1 | (if bit { 1 } else { 0 })
                        })
                }).collect()
            },
            None => vec![],
        }
    }
}

impl Serialize for RawFeatureVector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_bytes(Vec::<u8>::from(self.clone()).as_ref())
    }
}

#[derive(Default)]
struct BytesVisitor;

impl<'de> de::Visitor<'de> for BytesVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("s")
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E> where E: de::Error {
        Ok(v)
    }
}

impl<'de> Deserialize<'de> for RawFeatureVector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_byte_buf(BytesVisitor::default())
            .map(|bytes| bytes.into())
    }
}

pub struct FeatureVector {
    raw: RawFeatureVector,
    names: HashMap<FeatureBit, String>,
}

#[cfg(test)]
mod tests {
    use bincode;
    use super::RawFeatureVector;
    use super::FeatureBit;

    use serde::Deserialize;
    use serde::de::Deserializer;
    use serde::Serialize;
    use serde::ser::Serializer;
    use std::result;
    use bincode::Result;

    #[derive(Copy, Clone)]
    struct LengthSD;

    impl bincode::LengthSDOptions for LengthSD {
        fn serialized_length_size(&self, length: u64) -> Result<usize> {
            let _ = length;
            Ok(2)
        }

        fn serialize_length<S: Serializer>(&self, s: S, length: usize) -> result::Result<S::Ok, S::Error> {
            let length = length as u16;
            Serialize::serialize(&length, s)
        }

        fn deserialize_length<'de, D: Deserializer<'de>>(&self, d: D) -> result::Result<usize, D::Error> {
            Deserialize::deserialize(d).map(|l: u16| l as _)
        }
    }

    #[test]
    fn serde() {
        let mut temp = bincode::config();
        let bc_config = temp.big_endian();

        let feature_vector = RawFeatureVector::with_serialize_size(0)
            .set_bit(FeatureBit::GossipQueriesOptional)
            .set_bit(FeatureBit::DataLossProtectRequired);

        let mut data = vec![];
        bc_config.serialize_custom_length_into(&mut data, &feature_vector, LengthSD).unwrap();

        println!("{:?}", data);
        let new_feature_vector = bc_config.deserialize_custom_length_from(&data[..], LengthSD).unwrap();

        assert_eq!(feature_vector, new_feature_vector);
    }
}
