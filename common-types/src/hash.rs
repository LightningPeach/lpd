use dependencies::hex;
use dependencies::hex_literal;
use dependencies::rand;
use dependencies::secp256k1;
use dependencies::bitcoin_hashes;
use dependencies::digest;
use dependencies::typenum;

use digest::generic_array::{GenericArray, ArrayLength};
use typenum::{U32, U64};

use bitcoin_hashes::sha256;
use bitcoin_hashes::Hash;
use bitcoin_hashes::HashEngine;

use hex as hex_mod;
use std::error::Error;
use serde::{Serialize, Deserialize};
use hex_literal::*;

/// Sha256 is implementation of sha256 hash. It is created as a separate type
/// to allow implementing foreign traits.
#[derive(Serialize, Deserialize, Eq, PartialEq, Copy, Clone, Default, PartialOrd, Ord, Hash)]
pub struct Sha256 {
    data: [u8; 32],
}

impl Sha256 {
    // TODO(mkl): add other blockchains: simnet, regtest, litecoin ...
    // TODO(mkl): maybe move to other file
    pub const BITCOIN_CHAIN_HASH: Self = Sha256 {
        data: hex!("6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000"),
    };

    pub const TEST_HASH: Self = Sha256 {
        data: hex!("38faad210ccb4b018c866049827661643433f1a261a54a8b3faa9e682341158d"),
    };

    pub fn from_hex(s: &str) -> Result<Sha256, Box<dyn Error>> {
        let b = hex_mod::decode(s)
            .map_err(|err| format!("cannot decode Hash256 from hex: {:?}", err))?;
        if b.len() != 32 {
            return Err(format!("error decoding Hash256, wrong byte length, got {}, want {}", b.len(), 32).into())
        }
        let mut h: [u8; 32] = [0; 32];
        h.copy_from_slice(&b[..]);
        Ok(Sha256 {
            data: h
        })
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.data[..])
    }

    pub fn from_slice(s: &[u8]) -> Result<Sha256, Box<dyn Error>> {
        if s.len() == 32 {
            let mut data =  [0u8; 32];
            data.copy_from_slice(s);
            Ok(Sha256{
                data
            })
        } else {
            Err(format!("incorrect byte slice length for creation Sha256, got {}, want {}", s.len(), 32).into())
        }
    }

    pub fn hash_mult(v: &[ &[u8] ]) -> Self {
        let mut engine = sha256::HashEngine::default();
        for s in v {
            engine.input(s);
        }
        let data = sha256::Hash::from_engine(engine).into_inner();

        Sha256{
            data
        }
    }
}

impl From<[u8; 32]> for Sha256 {
    fn from(v: [u8; 32]) -> Self {
        Sha256 {
            data: v
        }
    }
}

impl From<Sha256> for [u8; 32] {
    fn from(v: Sha256) -> [u8; 32] {
        v.data
    }
}

impl AsRef<[u8]> for Sha256 {
    fn as_ref(&self) -> &[u8] {
        &self.data[..]
    }
}

impl std::fmt::Debug for Sha256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Hash256({})", hex::encode(&self.data[0..]))
    }
}

// just hex, it is important
impl std::fmt::Display for Sha256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.data[0..]))
    }
}

impl std::fmt::LowerHex for Sha256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.data[0..]))
    }
}

impl std::borrow::Borrow<[u8]> for Sha256 {
    fn borrow(&self) -> &[u8] {
        &self.data[..]
    }
}

impl std::ops::Index<usize> for Sha256 {
    type Output = u8;

    fn index(&self, x: usize) -> &u8 {
        &self.data[x]
    }
}

impl std::ops::Index<std::ops::RangeFull>  for Sha256 {
    type Output = [u8];

    fn index(&self, x: std::ops::RangeFull) -> &[u8] {
        &self.data[x]
    }
}

impl std::ops::Index<std::ops::Range<usize>>  for Sha256 {
    type Output = [u8];

    fn index(&self, x: std::ops::Range<usize>) -> &[u8] {
        &self.data[x]
    }
}

impl std::ops::Index<std::ops::RangeTo<usize>>  for Sha256 {
    type Output = [u8];

    fn index(&self, x: std::ops::RangeTo<usize>) -> &[u8] {
        &self.data[x]
    }
}

impl std::ops::Index<std::ops::RangeFrom<usize>>  for Sha256 {
    type Output = [u8];

    fn index(&self, x: std::ops::RangeFrom<usize>) -> &[u8] {
        &self.data[x]
    }
}

impl std::ops::IndexMut<usize> for Sha256 {
    fn index_mut(&mut self, x: usize) -> &mut u8 {
        &mut self.data[x]
    }
}

mod secp256k1_m {
    use super::Sha256;
    use super::secp256k1::Message;

    impl From<Sha256> for Message {
        fn from(v: Sha256) -> Self {
            Message::from_slice(&v.data[..]).unwrap()
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod rand_m {
    use super::Sha256;
    use super::rand;

    use rand::distributions::{Distribution, Standard};
    use rand::Rng;

    impl Distribution<Sha256> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Sha256 {
            let mut rng = rng;
            let rnd_bytes: Vec<u8> = self.sample_iter(&mut rng).take(32).collect();
            let mut this = Sha256 { data: [0u8; 32], };
            this.data.copy_from_slice(rnd_bytes.as_slice());
            this
        }
    }
}

#[derive(Clone, Default)]
pub struct Sha256HashEngine {
    engine: sha256::HashEngine
}

impl bitcoin_hashes::HashEngine for Sha256HashEngine {
    const BLOCK_SIZE: usize = 64;
    type MidState = sha256::Midstate;

    fn midstate(&self) -> Self::MidState {
        self.engine.midstate()
    }

    fn input(&mut self, data: &[u8]) {
        self.engine.input(data);
    }
}

impl bitcoin_hashes::Hash for Sha256 {
    type Engine = Sha256HashEngine;
    type Inner = [u8; 32];
    const LEN: usize = 32;
    const DISPLAY_BACKWARD: bool = false;

    fn from_engine(e: Sha256HashEngine) -> Self {
        let data = sha256::Hash::from_engine(e.engine);
        Sha256{
            data: data.into_inner()
        }
    }

    fn from_slice(sl: &[u8]) -> Result<Self, bitcoin_hashes::Error> {
        if sl.len() == 32 {
            let mut data = [0u8; 32];
            data.copy_from_slice(sl);
            Ok(
                Sha256{
                    data
                }
            )
        } else {
            Err(bitcoin_hashes::Error::InvalidLength(32, sl.len()))
        }
    }

    fn into_inner(self) -> [u8; 32] {
        self.data
    }

    fn from_inner(data: [u8; 32]) -> Self {
        Sha256 {
            data
        }
    }
}

impl digest::Input for Sha256HashEngine {
    fn input<B: AsRef<[u8]>>(&mut self, data: B) {
        self.engine.input(data.as_ref());
    }
}

impl digest::BlockInput for Sha256HashEngine {
    type BlockSize = U64;
}

impl digest::FixedOutput for Sha256HashEngine {
    type OutputSize = U32;

    /// Retrieve result and consume hasher instance.
    fn fixed_result(self) -> GenericArray<u8, Self::OutputSize> {
        let data = Sha256::from_engine(self);
        *GenericArray::from_slice(&data.data[..])
    }
}

impl digest::Reset for Sha256HashEngine {
    fn reset(&mut self) {
        self.engine = sha256::HashEngine::default();
    }
}

// TODO(mkl): add some tests
// TODO(mkl): move hash and engine into seperate files