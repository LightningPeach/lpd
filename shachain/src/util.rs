use serde::{Serialize, Deserialize};

const START_INDEX: LeafIndex = LeafIndex(((1u64 << (MAX_HEIGHT as u64)) - 1) as u64);
pub const MAX_HEIGHT: usize = 48;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Eq, PartialEq)]
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
    ((value >> (n as u64)) & 1) == 1
}

pub fn count_trailing_zeroes(value: u64) -> usize {
    for n in 0..MAX_HEIGHT {
        if get_nth_bit(value, n) {
            return n
        }
    }
    MAX_HEIGHT
}
