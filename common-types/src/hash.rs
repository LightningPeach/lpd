use hex as hex_mod;
use std::error::Error;
use serde::{Serialize, Deserialize};
use hex_literal::*;

#[derive(Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub struct Hash256 {
    data: [u8; 32],
}

impl Hash256 {
    // TODO(mkl): add other blockchains: simnet, regtest, litecoin ...
    pub const BITCOIN_CHAIN_HASH: Self = Hash256 {
        data: hex!("6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000"),
    };

    pub const TEST_HASH: Self = Hash256 {
        data: hex!("38faad210ccb4b018c866049827661643433f1a261a54a8b3faa9e682341158d"),
    };

    pub fn from_hex(s: &str) -> Result<Hash256, Box<Error>> {
        let b = hex_mod::decode(s)
            .map_err(|err| format!("cannot decode Hash256 from hex: {:?}", err))?;
        if b.len() != 32 {
            return Err(format!("error decoding Hash256, wrong byte length, got {}, want {}", b.len(), 32).into())
        }
        let mut h: [u8; 32] = [0; 32];
        h.copy_from_slice(&b[..]);
        Ok(Hash256{
            data: h
        })
    }
}

impl From<[u8; 32]> for Hash256 {
    fn from(v: [u8; 32]) -> Self {
        Hash256 {
            data: v
        }
    }
}

impl AsRef<[u8]> for Hash256 {
    fn as_ref(&self) -> &[u8] {
        &self.data[..]
    }
}

impl<'a> From<&'a [u8]> for Hash256 {
    fn from(v: &'a [u8]) -> Self {
        Self::from(&[v][..])
    }
}

impl<'a> From<&'a [&'a [u8]]> for Hash256 {
    fn from(v: &'a [&'a [u8]]) -> Self {
        use sha2::{Sha256, Digest};

        let hash = v.iter()
            .fold(Sha256::default(), |mut hasher, &a| {
                hasher.input(a);
                hasher
            })
            .result();

        let mut array: [u8; 32] = [0; 32];
        array.copy_from_slice(&hash);
        array.into()
    }
}

mod debug {
    use super::Hash256;

    use std::fmt;

    impl fmt::Debug for Hash256 {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Hash256({})", hex::encode(&self.data[0..]))
        }
    }

    // just hex, it is important
    impl fmt::Display for Hash256 {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", hex::encode(&self.data[0..]))
        }
    }
}

mod secp256k1_m {
    use super::Hash256;
    use secp256k1::Message;

    impl From<Hash256> for Message {
        fn from(v: Hash256) -> Self {
            Message::from_slice(&v.data[..]).unwrap()
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod rand_m {
    use super::Hash256;

    use rand::distributions::{Distribution, Standard};
    use rand::Rng;

    impl Distribution<Hash256> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Hash256 {
            let mut rng = rng;
            let rnd_bytes: Vec<u8> = self.sample_iter(&mut rng).take(32).collect();
            let mut this = Hash256 { data: [0u8; 32], };
            this.data.copy_from_slice(rnd_bytes.as_slice());
            this
        }
    }
}
