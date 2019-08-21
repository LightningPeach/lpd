use common_types::Sha256;
use std::sync::{Arc, Mutex};
use wallet_lib::interface::Wallet;

pub struct Blockchain {
    wallet: Arc<Mutex<Box<dyn Wallet + Send>>>,
    block_hash: Sha256,
    block_height: u32,
}

impl Blockchain {
    pub fn bitcoin(wallet: Arc<Mutex<Box<dyn Wallet + Send>>>) -> Self {
        Blockchain {
            wallet: wallet,
            block_height: 0,
            block_hash: Sha256::TEST_HASH,
        }
    }

    pub fn height(&self) -> u32 {
        self.block_height
    }

    pub fn hash(&self) -> Sha256 {
        self.block_hash.clone()
    }

    pub fn sync(&mut self) {
        let mut wallet = self.wallet.lock().unwrap();
        let _ = wallet.sync_with_tip();
        self.block_height = wallet.wallet_lib().get_last_seen_block_height_from_memory() as _;
    }
}
