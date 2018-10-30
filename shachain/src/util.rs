use hex;

use std::ops::{Index, IndexMut};

const SHA256_HASH_SIZE: usize = 32;
const START_INDEX: LeafIndex = LeafIndex((1 << MAX_HEIGHT) - 1);
pub const MAX_HEIGHT: usize = 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sha256Hash([u8; SHA256_HASH_SIZE]);

impl From<[u8; SHA256_HASH_SIZE]> for Sha256Hash {
    fn from(bytes: [u8; SHA256_HASH_SIZE]) -> Self {
        Sha256Hash(bytes)
    }
}

impl Into<[u8; SHA256_HASH_SIZE]> for Sha256Hash {
    fn into(self) -> [u8; SHA256_HASH_SIZE] {
        self.0
    }
}

// TODO(evg): impl Display for Sha256Hash?
// TODO(evg): impl Into<String> forSha256Hash?

impl Index<usize> for Sha256Hash {
    type Output = u8;

    fn index(&self, n: usize) -> &Self::Output {
        &self.0[n]
    }
}

impl IndexMut<usize> for Sha256Hash {
    fn index_mut(&mut self, n: usize) -> &mut u8 {
        &mut self.0[n]
    }
}

impl Default for Sha256Hash {
    fn default() -> Self {
        Sha256Hash([0; SHA256_HASH_SIZE])
    }
}

impl Sha256Hash {
    pub fn from_hex(hex_hash: &str) -> Result<Self, hex::FromHexError> {
        let mut hash = [0; SHA256_HASH_SIZE];
        hash.copy_from_slice(&hex::decode(hex_hash)?);
        Ok(Sha256Hash(hash))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn copy_from_slice(&mut self, src: &[u8]) {
        self.0.copy_from_slice(src)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LeafIndex(pub u64);

impl Into<u64> for LeafIndex {
    fn into(self) -> u64 {
        self.0
    }
}

impl LeafIndex {
    pub fn new(v: u64) -> Self {
        LeafIndex(START_INDEX.0 - v)
    }

    pub fn incr(&mut self) {
        self.0 -= 1;
    }
}

pub fn get_nth_bit(value: u64, n: usize) -> bool {
    ((value >> n) & 1) == 1
}

pub fn count_trailing_zeroes(value: u64) -> usize {
    for n in 0..MAX_HEIGHT {
        if get_nth_bit(value, n) {
            return n
        }
    }
    MAX_HEIGHT
}