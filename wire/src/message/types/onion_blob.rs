
pub const ONION_PACKET_SIZE: usize = 1366;

#[derive(Clone)]
pub struct OnionBlob {
    data: [u8; ONION_PACKET_SIZE],
}

mod serde_m {
    use super::OnionBlob;
    use super::ONION_PACKET_SIZE;

    use serde::Serialize;
    use serde::Serializer;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::de::Visitor;
    use serde::de::SeqAccess;
    use serde::de::Error;
    use std::fmt;

    impl Serialize for OnionBlob {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
            use serde::ser::SerializeTuple;

            let mut tuple = serializer.serialize_tuple(ONION_PACKET_SIZE)?;
            for i in 0..ONION_PACKET_SIZE {
                tuple.serialize_element(&self.data[i])?;
            }

            tuple.end()
        }
    }

    impl<'de> Deserialize<'de> for OnionBlob {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            struct V;

            impl<'de> Visitor<'de> for V {
                type Value = OnionBlob;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "{} bytes", ONION_PACKET_SIZE)
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                    let mut seq = seq;
                    let mut blob = OnionBlob {
                        data: [0; ONION_PACKET_SIZE],
                    };
                    for i in 0..ONION_PACKET_SIZE {
                        if let Some(value) =  seq.next_element()? {
                            blob.data[i] = value;
                        } else {
                            return Err(Error::custom("unexpected end"));
                        }
                    }

                    Ok(blob)
                }
            }

            deserializer.deserialize_tuple(ONION_PACKET_SIZE, V)
        }
    }
}

mod eq {
    use super::OnionBlob;
    use super::ONION_PACKET_SIZE;

    use std::cmp::Eq;
    use std::cmp::PartialEq;

    impl PartialEq for OnionBlob {
        fn eq(&self, other: &Self) -> bool {
            (0..ONION_PACKET_SIZE)
                .fold(true, |acc, index| acc && self.data[index] == other.data[index])
        }
    }

    impl Eq for OnionBlob {
    }
}

mod debug {
    use super::OnionBlob;

    use std::fmt::Debug;
    use std::fmt::Formatter;
    use std::fmt::Result;

    impl Debug for OnionBlob {
        fn fmt(&self, f: &mut Formatter) -> Result {
            self.data.fmt(f)
        }
    }
}
