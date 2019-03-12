use common_types::Hash256;

pub struct Blockchain {
    block_hash: Hash256,
    block_height: u32,
}

impl Blockchain {
    pub fn bitcoin() -> Self {
        Blockchain {
            block_height: 0,
            block_hash: Hash256::TEST_HASH,
        }
    }

    pub fn height(&self) -> u32 {
        self.block_height
    }

    pub fn hash(&self) -> Hash256 {
        self.block_hash.clone()
    }

    pub fn sync(&mut self) {
        unimplemented!()
    }
}
