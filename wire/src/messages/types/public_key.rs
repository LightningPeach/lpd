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

    // TODO:
    impl Serialize for Signature {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
            use serde::ser::SerializeTuple;

            (0..64)
                .fold(serializer.serialize_tuple(64), |tuple, index|
                    tuple.and_then(|mut tuple|
                        SerializeTuple::serialize_element(&mut tuple, &self.data[index])
                            .map(|_| tuple)
                    )
                )
                .and_then(|tuple| tuple.end())
        }
    }

    impl<'de> Deserialize<'de> for Signature {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            let _ = deserializer;
            unimplemented!()
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
