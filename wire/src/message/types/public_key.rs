use ::PackSized;

pub const PUBLIC_KEY_SIZE: usize = 33;
pub const SIGNATURE_SIZE: usize = 64;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
pub struct PublicKey {
    header: u8,
    data: [u8; PUBLIC_KEY_SIZE - 1],
}

#[derive(Clone)]
pub struct Signature {
    data: [u8; SIGNATURE_SIZE],
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Signed<T> {
    pub signature: Signature,
    pub value: T,
}

impl PackSized for Signature {
}

impl PublicKey {
    pub fn new(header: u8, data: [u8; 32]) -> Self {
        PublicKey {
            header: header,
            data: data,
        }
    }
}

mod serde {
    use super::Signature;
    use super::SIGNATURE_SIZE;

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
                    write!(formatter, "{} bytes", SIGNATURE_SIZE)
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                    let mut seq = seq;
                    let mut signature = Signature {
                        data: [0; SIGNATURE_SIZE],
                    };
                    for i in 0..SIGNATURE_SIZE {
                        signature.data[i] = seq.next_element()?
                            .ok_or(<A::Error as Error>::custom("unexpected end"))?;
                    }

                    Ok(signature)
                }
            }

            deserializer.deserialize_tuple(SIGNATURE_SIZE, V)
        }
    }
}

mod eq {
    use super::Signature;
    use super::SIGNATURE_SIZE;

    use std::cmp::Eq;
    use std::cmp::PartialEq;

    impl PartialEq for Signature {
        fn eq(&self, other: &Self) -> bool {
            (0..SIGNATURE_SIZE)
                .fold(true, |acc, index| acc && self.data[index] == other.data[index])
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
            _1.copy_from_slice(&self.data[32..64]);
            write!(f, "Signature [{:?}, {:?}]", _0, _1)
        }
    }
}

mod secp256k1 {
    use super::PublicKey as LpdPublicKey;
    use super::PUBLIC_KEY_SIZE;

    use super::Signature as LpdSignature;

    use secp256k1::Secp256k1;
    use secp256k1::PublicKey;
    use secp256k1::Signature;

    impl From<PublicKey> for LpdPublicKey {
        fn from(v: PublicKey) -> Self {
            let mut pk = LpdPublicKey {
                header: 0,
                data: [0; PUBLIC_KEY_SIZE - 1],
            };
            let v_array = v.serialize();
            pk.header = v_array[0];
            pk.data.copy_from_slice(&v_array[1..]);
            pk
        }
    }

    impl From<LpdPublicKey> for PublicKey {
        fn from(v: LpdPublicKey) -> Self {
            // TODO: use TryFrom
            let mut v_array = [0u8; PUBLIC_KEY_SIZE];
            v_array[0] = v.header;
            v_array[1..].copy_from_slice(&v.data[..]);
            PublicKey::from_slice(&Secp256k1::new(), &v_array[..]).unwrap()
        }
    }

    impl From<Signature> for LpdSignature {
        fn from(v: Signature) -> Self {
            let v_array = v.serialize_compact(&Secp256k1::new());
            LpdSignature {
                data: v_array,
            }
        }
    }

    impl From<LpdSignature> for Signature {
        fn from(v: LpdSignature) -> Self {
            Signature::from_compact(&Secp256k1::new(), &v.data).unwrap()
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod rand {
    use super::PublicKey;
    use super::Signature;

    use super::PUBLIC_KEY_SIZE;
    use super::SIGNATURE_SIZE;

    use rand::distributions::Distribution;
    use rand::distributions::Standard;
    use rand::Rng;

    impl Distribution<PublicKey> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PublicKey {
            let mut rng = rng;
            let rnd_bytes: Vec<u8> = self.sample_iter(&mut rng).take(PUBLIC_KEY_SIZE - 1).collect();
            let mut this = PublicKey { header: rng.gen(), data: [0u8; PUBLIC_KEY_SIZE - 1] };
            this.data.copy_from_slice(rnd_bytes.as_slice());
            this
        }
    }

    impl Distribution<Signature> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Signature {
            let mut rng = rng;
            let rnd_bytes: Vec<u8> = self.sample_iter(&mut rng).take(SIGNATURE_SIZE).collect();
            let mut this = Signature { data: [0u8; SIGNATURE_SIZE] };
            this.data.copy_from_slice(rnd_bytes.as_slice());
            this
        }
    }
}

#[cfg(feature = "testing")]
mod literal {
    use hex_literal;
    use super::PublicKey;

    #[macro_export]
    macro_rules! public_key {
        ($header:expr, $value:expr) => { PublicKey::new($header, hex!($value)) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::BinarySD;

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
