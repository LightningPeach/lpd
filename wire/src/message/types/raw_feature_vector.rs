use std::collections::HashSet;

use serde::Serialize;
use serde::Serializer;
use serde::Deserialize;
use serde::Deserializer;

use super::FeatureBit;

#[derive(Default, Clone, Eq, PartialEq)]
pub struct RawFeatureVector {
    set: HashSet<FeatureBit>,
}

impl RawFeatureVector {
    pub const BITS: u16 = 8;

    pub fn new() -> Self {
        RawFeatureVector {
            set: HashSet::new(),
        }
    }

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

    pub fn is_set_bit(&self, feature_bit: &FeatureBit) -> bool {
        self.set.contains(feature_bit)
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
                (0..Self::BITS).fold(fv, |fv, bit_index| {
                    let global_bit_index = byte_index * RawFeatureVector::BITS + bit_index;
                    let feature_bit = global_bit_index.into();
                    let bit = (byte & ((1 << bit_index) as u8)) != 0;
                    if bit {
                        fv.set_bit(feature_bit)
                    } else {
                        fv
                    }
                })
            })

    }
}

impl From<RawFeatureVector> for Vec<u8> {
    fn from(feature_vector: RawFeatureVector) -> Self {
        match feature_vector.serialize_size() {
            Some(len) => {
                // map each byte index into actual byte
                (0..(len as u16)).map(|byte_index|
                    // for each bit in byte check
                    // if corresponding feature_bit is contained in the set
                    // and accumulate the presence in the byte
                    (0..RawFeatureVector::BITS)
                        .fold(0u8, |acc, bit_index| {
                            let bit_index = RawFeatureVector::BITS - bit_index - 1;
                            let global_bit_index = byte_index * RawFeatureVector::BITS + bit_index;
                            let feature_bit = FeatureBit::from(global_bit_index);
                            let bit = feature_vector.is_set_bit(&feature_bit);
                            acc << 1 | (if bit { 1 } else { 0 })
                        })
                ).collect()
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

impl<'de> Deserialize<'de> for RawFeatureVector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        use serde::de;
        use std::fmt;

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

        deserializer.deserialize_byte_buf(BytesVisitor::default())
            .map(|bytes| bytes.into())
    }
}

mod debug {
    use std::fmt;
    use super::RawFeatureVector;

    impl fmt::Debug for RawFeatureVector {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if self.set.is_empty() {
                write!(f, "RawFeatureVector [ ]")
            } else {
                write!(f, "RawFeatureVector [ {} ]", self.set.iter().fold(String::new(), |s, item| {
                    if s.is_empty() {
                        format!("{:?}", item)
                    } else {
                        format!("{}, {:?}", s, item)
                    }
                }))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::RawFeatureVector;
    use super::FeatureBit;

    use ::serde_facade::BinarySD;

    use rand::thread_rng;
    use rand::Rng;
    use rand::distributions::Standard;

    #[test]
    fn serde() {
        let feature_vector = RawFeatureVector::with_serialize_size(0)
            .set_bit(FeatureBit::GossipQueriesOptional)
            .set_bit(FeatureBit::DataLossProtectRequired)
            .set_bit(FeatureBit::Custom(15));

        let mut data = Vec::<u8>::new();
        BinarySD::serialize(&mut data, &feature_vector).unwrap();

        println!("{:?}", data);
        data.append(&mut thread_rng().sample_iter(&Standard).take(10).collect());
        println!("{:?} added additional bytes", data);
        let new_feature_vector = BinarySD::deserialize(&data[..]).unwrap();

        assert_eq!(feature_vector, new_feature_vector);
    }

    #[test]
    fn empty() {
        let v = vec![0u8, 0u8];
        let t: RawFeatureVector = BinarySD::deserialize(&v[..]).unwrap();
        println!("{:?}", t);
    }
}
