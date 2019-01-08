use binformat::PackSized;

use secp256k1::PublicKey as Secp256k1PublicKey;
use secp256k1::SecretKey as Secp256k1SecretKey;
use secp256k1::Signature as Secp256k1Signature;
pub use secp256k1::Error as Secp256k1Error;

pub use secp256k1::constants::PUBLIC_KEY_SIZE;
pub use secp256k1::constants::SECRET_KEY_SIZE;
pub const SIGNATURE_SIZE: usize = 64;

#[derive(Eq, PartialEq, Clone, PartialOrd, Ord)]
pub struct PublicKey {
    raw: Secp256k1PublicKey,
}

impl AsRef<Secp256k1PublicKey> for PublicKey {
    fn as_ref(&self) -> &Secp256k1PublicKey {
        &self.raw
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct SecretKey {
    raw: Secp256k1SecretKey,
}

impl AsRef<Secp256k1SecretKey> for SecretKey {
    fn as_ref(&self) -> &Secp256k1SecretKey {
        &self.raw
    }
}

#[derive(Clone)]
pub struct Signature {
    data: Secp256k1Signature,
}

impl AsRef<Secp256k1Signature> for Signature {
    fn as_ref(&self) -> &Secp256k1Signature {
        &self.data
    }
}

impl PackSized for Signature {
}

mod serde_m {
    use super::Signature;
    use super::SIGNATURE_SIZE;
    use super::Secp256k1Signature;

    use super::PublicKey;
    use super::PUBLIC_KEY_SIZE;
    use super::Secp256k1PublicKey;

    use super::SecretKey;
    use super::Secp256k1SecretKey;
    use super::SECRET_KEY_SIZE;

    use serde::Serialize;
    use serde::Serializer;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::de::Visitor;
    use serde::de::SeqAccess;
    use serde::de::Error;
    use std::fmt;

    impl Serialize for Signature {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
            use serde::ser::SerializeTuple;

            let mut tuple = serializer.serialize_tuple(SIGNATURE_SIZE)?;
            let data = self.data.serialize_compact();
            for i in 0..SIGNATURE_SIZE {
                tuple.serialize_element(&data[i])?;
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
                    write!(formatter, "{} bytes", SIGNATURE_SIZE)
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                    let mut seq = seq;
                    let mut data = [0u8; SIGNATURE_SIZE];
                    for i in 0..SIGNATURE_SIZE {
                        data[i] = seq.next_element()?
                            .ok_or(<A::Error as Error>::custom("unexpected end"))?;
                    }

                    Secp256k1Signature::from_compact(&data)
                        .map(Into::into)
                        .map_err(A::Error::custom)
                }
            }

            deserializer.deserialize_tuple(SIGNATURE_SIZE, V)
        }
    }

    impl Serialize for PublicKey {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
            use serde::ser::SerializeTuple;

            let mut tuple = serializer.serialize_tuple(PUBLIC_KEY_SIZE)?;
            let array = self.raw.serialize();
            for i in 0..PUBLIC_KEY_SIZE {
                tuple.serialize_element(&array[i])?;
            }

            tuple.end()
        }
    }

    impl<'de> Deserialize<'de> for PublicKey {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            struct V;

            impl<'de> Visitor<'de> for V {
                type Value = PublicKey;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "{} bytes", PUBLIC_KEY_SIZE)
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                    let mut seq = seq;
                    let mut data = [0; PUBLIC_KEY_SIZE];

                    for i in 0..PUBLIC_KEY_SIZE {
                        data[i] = seq.next_element()?
                            .ok_or(<A::Error as Error>::custom("unexpected end"))?;
                    }

                    Secp256k1PublicKey::from_slice(&data[..])
                        .map(|v| PublicKey { raw: v })
                        .map_err(|e| <A::Error as Error>::custom(format!("secp256k1::PublicKey cannot be created from such data: {:?}", e)))
                }
            }

            deserializer.deserialize_tuple(PUBLIC_KEY_SIZE, V)
        }
    }

    impl<'de> Deserialize<'de> for SecretKey {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            struct V;

            impl<'de> Visitor<'de> for V {
                type Value = SecretKey;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "{} bytes", SECRET_KEY_SIZE)
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                    let mut seq = seq;
                    let mut data = [0; SECRET_KEY_SIZE];

                    for i in 0..PUBLIC_KEY_SIZE {
                        data[i] = seq.next_element()?
                            .ok_or(<A::Error as Error>::custom("unexpected end"))?;
                    }

                    Secp256k1SecretKey::from_slice(&data[..])
                        .map(|v| SecretKey { raw: v })
                        .map_err(|e| <A::Error as Error>::custom(format!("secp256k1::SecretKey cannot be created from such data: {:?}", e)))
                }
            }

            deserializer.deserialize_tuple(PUBLIC_KEY_SIZE, V)
        }
    }
}

mod eq {
    use super::Signature;

    use std::cmp::Eq;
    use std::cmp::PartialEq;

    impl PartialEq for Signature {
        fn eq(&self, other: &Self) -> bool {
            self.data == other.data
        }
    }

    impl Eq for Signature {
    }
}

mod debug {
    use super::Signature;
    use super::PublicKey;
    use super::SecretKey;

    use std::fmt::{Debug, Display};
    use std::fmt::Formatter;
    use std::fmt::Result;

    use hex::encode;

    impl Debug for Signature {
        fn fmt(&self, f: &mut Formatter) -> Result {
            write!(f, "Signature [ {} ]", encode(&self.data.serialize_compact()[0..]))
        }
    }

    impl Display for Signature {
        fn fmt(&self, f: &mut Formatter) -> Result {
            write!(f, "{}", encode(&self.data.serialize_compact()[..]))
        }
    }

    impl Debug for PublicKey {
        fn fmt(&self, f: &mut Formatter) -> Result {
            write!(f, "{:?}", self.raw)
        }
    }

    impl Debug for SecretKey {
        fn fmt(&self, f: &mut Formatter) -> Result {
            write!(f, "{:?}", self.raw)
        }
    }

    impl Display for PublicKey {
        fn fmt(&self, f: &mut Formatter) -> Result {
            write!(f, "{}", encode(&self.raw.serialize()[..]))
        }
    }
}

mod secp256k1_m {
    use super::PublicKey as LpdPublicKey;
    use super::SecretKey as LpdPrivateKey;
    use super::Signature as LpdSignature;

    use secp256k1::Secp256k1;
    use secp256k1::PublicKey;
    use secp256k1::SecretKey;
    use secp256k1::Signature;

    impl LpdPublicKey {
        pub fn paired(private: &LpdPrivateKey) -> Self {
            let ctx = Secp256k1::new();
            PublicKey::from_secret_key(&ctx, &private.raw).into()
        }
    }

    impl From<PublicKey> for LpdPublicKey {
        fn from(v: PublicKey) -> Self {
            LpdPublicKey {
                raw: v,
            }
        }
    }

    impl From<SecretKey> for LpdPrivateKey {
        fn from(v: SecretKey) -> Self {
            LpdPrivateKey {
                raw: v,
            }
        }
    }

    impl From<LpdPublicKey> for PublicKey {
        fn from(v: LpdPublicKey) -> Self {
            v.raw
        }
    }

    impl From<Signature> for LpdSignature {
        fn from(v: Signature) -> Self {
            LpdSignature {
                data: v,
            }
        }
    }

    impl From<LpdSignature> for Signature {
        fn from(v: LpdSignature) -> Self {
            v.data
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod rand_m {
    use super::PublicKey;
    use super::SecretKey;
    use super::Signature;

    use super::PUBLIC_KEY_SIZE;
    use super::SECRET_KEY_SIZE;
    use super::SIGNATURE_SIZE;

    use rand::distributions::Distribution;
    use rand::distributions::Standard;
    use rand::Rng;

    impl Distribution<PublicKey> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PublicKey {
            use super::Secp256k1PublicKey;

            let mut rng = rng;
            let mut rnd_bytes: Vec<u8> = self.sample_iter(&mut rng).take(PUBLIC_KEY_SIZE).collect();
            rnd_bytes[0] = 2;
            PublicKey {
                raw: Secp256k1PublicKey::from_slice(rnd_bytes.as_slice()).unwrap(),
            }
        }
    }

    impl Distribution<SecretKey> for Standard {

        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SecretKey {
            use super::Secp256k1SecretKey;

            let mut rng = rng;
            let rnd_bytes: Vec<u8> = self.sample_iter(&mut rng).take(SECRET_KEY_SIZE).collect();
            SecretKey {
                raw: Secp256k1SecretKey::from_slice(rnd_bytes.as_slice()).unwrap(),
            }
        }
    }

    impl Distribution<Signature> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Signature {
            use super::Secp256k1Signature;

            let mut rng = rng;
            let mut try_inner = || {
                let rnd_bytes: Vec<u8> = self.sample_iter(&mut rng).take(SIGNATURE_SIZE).collect();
                Secp256k1Signature::from_compact(&rnd_bytes.as_slice())
            };
            let mut inner: Option<Secp256k1Signature> = None;
            for _ in 0..8 {
                inner = try_inner().ok();
                if inner.is_some() {
                    break;
                }
            }

            Signature {
                data: inner.unwrap(),
            }
        }
    }
}

#[cfg(feature = "testing")]
#[macro_export]
macro_rules! public_key {
    ($value:expr) => { {
        use secp256k1::PublicKey;
        use secp256k1::Secp256k1;
        PublicKey::from_slice(&Secp256k1::new(), &hex!($value)).unwrap().into()
    } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binformat::BinarySD;

    #[test]
    fn signature() {
        let v = vec! [
            169u8, 177, 196, 25, 57, 80, 208, 176, 113, 192, 129, 194, 129, 60, 75, 12,
            21, 77, 188, 167, 162, 88, 249, 147, 231, 18, 208, 195, 174, 189, 240, 95,
            66, 108, 150, 147, 28, 77, 128, 69, 220, 78, 55, 45, 9, 120, 107, 254,
            154, 144, 165, 228, 138, 174, 67, 16, 90, 251, 148, 174, 188, 40, 216, 163,
        ];

        let t: Signature = BinarySD::deserialize(&v[..]).unwrap();
        println!("{:?}", t);

    }
}
