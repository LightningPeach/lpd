#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct PublicKey {
    data: [u8; 32],
    last: u8,
}
pub struct Signature {
    data: [u8; 64],
}

mod serde {
    use super::Signature;

    use serde::Serialize;
    use serde::Serializer;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::de::Visitor;
    use serde::de::SeqAccess;
    use serde::de::Error;
    use std::fmt;

    const SIGNATURE_SIZE: usize = 64;

    impl Serialize for Signature {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
            use serde::ser::SerializeTuple;

            let mut tuple = serializer.serialize_tuple(SIGNATURE_SIZE)?;
            for i in 0..SIGNATURE_SIZE {
                tuple.serialize_element(&self.data[i])?;
            }

            tuple.end()
        }
    }

    impl<'de> Deserialize<'de> for Signature {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            struct V;

            impl<'de> Visitor<'de> for V {
                type Value = Signature;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "64 bytes")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                    let mut seq = seq;
                    let mut signature = Signature {
                        data: [0; SIGNATURE_SIZE],
                    };
                    for i in 0..SIGNATURE_SIZE {
                        if let Some(value) =  seq.next_element()? {
                            signature.data[i] = value;
                        } else {
                            return Err(Error::custom("unexpected end"));
                        }
                    }

                    Ok(signature)
                }
            }

            deserializer.deserialize_tuple(64, V)
        }
    }
}

mod eq {
    use super::Signature;

    use std::cmp::Eq;
    use std::cmp::PartialEq;

    impl PartialEq for Signature {
        fn eq(&self, other: &Self) -> bool {
            (0..64).fold(true, |acc, index| acc && self.data[index] == other.data[index])
        }
    }

    impl Eq for Signature {
    }
}

mod debug {
    use super::Signature;

    use std::fmt::Debug;
    use std::fmt::Formatter;
    use std::fmt::Result;

    impl Debug for Signature {
        fn fmt(&self, f: &mut Formatter) -> Result {
            let (mut _0, mut _1) = ([0u8; 32], [0u8; 32]);
            _0.copy_from_slice(&self.data[0..32]);
            _1.copy_from_slice(&self.data[32..32]);
            write!(f, "Signature [{:?}, {:?}]", _0, _1)
        }
    }
}

#[cfg(test)]
mod rand {
    use super::PublicKey;
    use super::Signature;

    use rand::distributions::Distribution;
    use rand::distributions::Standard;
    use rand::Rng;

    impl Distribution<PublicKey> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PublicKey {
            let mut rng = rng;
            let rnd_bytes: Vec<u8> = self.sample_iter(&mut rng).take(32).collect();
            let mut this = PublicKey { data: [0u8; 32], last: rng.gen() };
            this.data.copy_from_slice(rnd_bytes.as_slice());
            this
        }
    }

    impl Distribution<Signature> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Signature {
            let mut rng = rng;
            let rnd_bytes: Vec<u8> = self.sample_iter(&mut rng).take(64).collect();
            let mut this = Signature { data: [0u8; 64] };
            this.data.copy_from_slice(rnd_bytes.as_slice());
            this
        }
    }
}
