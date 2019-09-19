mod bitcoind;
pub use self::bitcoind::*;

mod btcd;
pub use self::btcd::*;

use std::io;
use bitcoin_rpc_client::{Client};

use crate::error::Error;

pub trait BitcoinConfig
where
    Self: Sized,
{
    type Instance: BitcoinInstance + AsRef<Self> + AsMut<Self>;

    fn new(name: &str) -> Result<Self, Error>;
    fn run(self) -> Result<Self::Instance, Error>;
    fn lnd_params(&self) -> Vec<String>;
}

pub trait BitcoinInstance
where
    Self: Sized,
{
    fn rpc_client(&self) -> Result<Client, Error>;
}
