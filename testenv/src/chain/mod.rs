mod bitcoind;
pub use self::bitcoind::*;

mod btcd;
pub use self::btcd::*;

use std::io;

pub trait BitcoinConfig
where
    Self: Sized,
{
    type Instance: BitcoinInstance + AsRef<Self> + AsMut<Self>;

    fn new(name: &str) -> Result<Self, io::Error>;
    fn run(self) -> Result<Self::Instance, io::Error>;
    fn params(&self) -> Vec<String>;
}

pub trait BitcoinInstance
where
    Self: Sized,
{
    fn set_mining_address(self, address: String) -> Result<Self, io::Error>;
    fn generate(&mut self, count: usize) -> Result<(), io::Error>;
}
