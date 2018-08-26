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

                fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                {
                    if v.len() != SIZE {
                        Err(E::custom(format!("the size {} is not equal to {}", v.len(), SIZE)))
                    } else {
                        String::from_utf8(Vec::from(v))
                            .map(|s| NodeAlias(s))
                            .map_err(|e| E::custom(format!("utf8 error: {}", e)))
                    }
                }

            }

            deserializer.deserialize_bytes(V)
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
