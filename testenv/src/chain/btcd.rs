use super::super::{Home, cleanup};
use super::{BitcoinConfig, BitcoinInstance};

use std::process::{Command, Child};
use std::{io, fs};
use bitcoin_rpc_client::BitcoinCoreClient;

pub struct Btcd {
    home: Home,
}

pub struct BtcdRunning {
    config: Btcd,
    instance: Child,
}

impl AsMut<Btcd> for BtcdRunning {
    fn as_mut(&mut self) -> &mut Btcd {
        &mut self.config
    }
}

impl AsRef<Btcd> for BtcdRunning {
    fn as_ref(&self) -> &Btcd {
        &self.config
    }
}

impl Drop for BtcdRunning {
    fn drop(&mut self) {
        self.instance.kill()
            .or_else(|e| match e.kind() {
                io::ErrorKind::InvalidInput => Ok(()),
                _ => Err(e),
            })
            .unwrap()
    }
}

impl BitcoinConfig for Btcd {
    type Instance = BtcdRunning;

    fn new(name: &str) -> Result<Self, io::Error> {
        Ok(Btcd {
            home: Home::new(name, false)
                .or_else(|e| if e.kind() == io::ErrorKind::AlreadyExists {
                    cleanup("btcd");
                    Home::new(name, true)
                } else {
                    Err(e)
                })?,
        })
    }

    fn run(self) -> Result<Self::Instance, io::Error> {
        self.run_internal(None)
    }

    fn lnd_params(&self) -> Vec<String> {
        vec![
            "--bitcoin.active".to_owned(),
            "--bitcoin.simnet".to_owned(),
            "--btcd.rpcuser=devuser".to_owned(),
            "--btcd.rpcpass=devpass".to_owned(),
            format!("--btcd.rpccert={}", self.home.public_key_path().to_str().unwrap()),
        ]
    }
}

impl Btcd {
    fn run_internal(self, mining_address: Option<String>) -> Result<BtcdRunning, io::Error> {
        fs::create_dir(self.home.ext_path("data")).or_else(|e|
            if e.kind() == io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(e)
            }
        )?;
        fs::create_dir(self.home.ext_path("logs")).or_else(|e|
            if e.kind() == io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(e)
            }
        )?;
        let mut args = vec![
            format!("--datadir={}", self.home.ext_path("data").to_str().unwrap()),
            format!("--logdir={}", self.home.ext_path("logs").to_str().unwrap()),
            format!("--rpccert={}", self.home.public_key_path().to_str().unwrap()),
            format!("--rpckey={}", self.home.private_key_path().to_str().unwrap()),
            format!("--configfile={}", self.home.ext_path("btcd.conf").to_str().unwrap()),
        ];

        if let Some(mining_address) = mining_address {
            args.push(format!("--miningaddr={}", mining_address));
        }

        Command::new("btcd")
            .args(&[
                "--simnet", "--txindex", "--rpcuser=devuser", "--rpcpass=devpass"
            ])
            .args(args)
            .spawn()
            .map(|instance| BtcdRunning {
                config: self,
                instance: instance,
            })
    }
}

impl BitcoinInstance for BtcdRunning {
    fn rpc_client(&self) -> BitcoinCoreClient {
        BitcoinCoreClient::new("tcp://localhost:18556", "devuser", "devpass")
    }
}
