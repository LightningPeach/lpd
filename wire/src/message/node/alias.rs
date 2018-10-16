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

    impl<'de> Deserialize<'de> for NodeAlias {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            let buffer: [u8; SIZE] = Deserialize::deserialize(deserializer)?;
            let len = buffer.iter()
                .try_fold(0, |state, &b| if b == 0 { Err(state) } else { Ok(state + 1) })
                .err().unwrap_or(SIZE);
            let string = String::from_utf8((&buffer[0..len]).to_owned())
                .map_err(<D::Error as de::Error>::custom)?;
            Ok(NodeAlias(string))
        }
    }

    impl Serialize for NodeAlias {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
            let &NodeAlias(ref s) = self;
            let v = s.as_bytes();

            if v.len() >= SIZE {
                Err(<S::Error as ser::Error>::custom(format!("the size {} of alias overflows {} limit", v.len(), SIZE - 1)))
            } else {
                let mut buffer = [0u8; SIZE];
                buffer[0..v.len()].copy_from_slice(v);
                buffer.serialize(serializer)
            }
        }
    }
}
