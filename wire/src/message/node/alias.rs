#[derive(Clone, Eq, PartialEq, Debug)]
pub struct NodeAlias(String);

const SIZE: usize = 32;

impl NodeAlias {
    pub fn string(self) -> String {
        self.0
    }

    pub fn try_from_string(s: String) -> Option<Self> {
        if s.len() == SIZE {
            Some(NodeAlias(s))
        } else {
            None
        }
    }
}

mod serde {
    use super::NodeAlias;
    use super::SIZE;

    use serde::ser;
    use serde::de;
    use serde::Serialize;
    use serde::Serializer;
    use serde::Deserialize;
    use serde::Deserializer;
    use std::fmt;

    impl<'de> Deserialize<'de> for NodeAlias {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            struct V;

            impl<'de> de::Visitor<'de> for V {
                type Value = NodeAlias;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "{} byte valid utf8 string", SIZE)
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error> where A: de::SeqAccess<'de>, {
                    let mut seq = seq;
                    let v: [u8; 32] = seq.next_element()?
                        .ok_or(<A::Error as de::Error>::custom(format!("expected 32 bytes")))?;
                    let len = v.iter().enumerate()
                        .fold((0, true), |(a, f), (i, &b)| (if f { i } else { a }, f & (b != 0))).0;
                    String::from_utf8(Vec::from(&v[0..len]))
                        .map(|s| NodeAlias(s))
                        .map_err(|e| <A::Error as de::Error>::custom(format!("utf8 error: {}", e)))
                }

            }

            deserializer.deserialize_tuple(32, V)
        }
    }

    impl Serialize for NodeAlias {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
            let &NodeAlias(ref s) = self;
            let v = s.as_bytes();

            if s.len() != SIZE {
                Err(<S::Error as ser::Error>::custom(format!("the size {} is not equal to {}", v.len(), SIZE)))
            } else {
                serializer.serialize_bytes(v)
            }
        }
    }
}
