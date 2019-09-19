use super::super::{Home, cleanup};
use super::{BitcoinConfig, BitcoinInstance};
use crate::error::Error;
use crate::{new_io_error, new_bitcoin_rpc_error};

use std::process::{Command, Child};
use std::{io, fs};
use bitcoin_rpc_client::{Client};

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

    fn new(name: &str) -> Result<Self, Error> {
        Ok(Btcd {
            // TODO(mkl): fix this
            home: Home::new(name, false, true)?
//                .or_else(|e| if e.kind() == io::ErrorKind::AlreadyExists {
//                    cleanup("btcd");
//                    Home::new(name, true, true)
//                } else {
//                    Err(e)
//                })?,
        })
    }

    fn run(self) -> Result<Self::Instance, Error> {
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
    fn run_internal(self, mining_address: Option<String>) -> Result<BtcdRunning, Error> {
        fs::create_dir(self.home.ext_path("data"))
            .or_else(|e|
                if e.kind() == io::ErrorKind::AlreadyExists {
                    Ok(())
                } else {
                    Err(e)
                }
            )
            .map_err(|err| {
                new_io_error!(
                    err,
                    "cannot create data dir for btcd",
                    self.home.ext_path("data").to_string_lossy().into_owned()
                )
            })?;
        fs::create_dir(self.home.ext_path("logs"))
            .or_else(|e|
                if e.kind() == io::ErrorKind::AlreadyExists {
                    Ok(())
                } else {
                    Err(e)
                }
            )
            .map_err(|err| {
                new_io_error!(
                    err,
                    "cannot create logs dir for btcd",
                    self.home.ext_path("logs").to_string_lossy().into_owned()
                )
            })?;

        // TODO(mkl): refactor this seperate function
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

        // TODO(mkl): why simnet?
        Command::new("btcd")
            .args(&[
                "--simnet", "--txindex", "--rpcuser=devuser", "--rpcpass=devpass",
                "-deprecatedrpc=generate",
            ])
            .args(args)
            .spawn()
            .map(|instance| BtcdRunning {
                config: self,
                instance: instance,
            })
            .map_err(|err| {
                new_io_error!(err, "cannot launch btcd")
            })
    }
}

impl BitcoinInstance for BtcdRunning {
    fn rpc_client(&self) -> Result<Client, Error> {
        use bitcoin_rpc_client::Auth::UserPass;

        Client::new("http://localhost:18556".to_owned(), UserPass("devuser".to_owned(), "devpass".to_owned()))
            .map_err(|err| {
                new_bitcoin_rpc_error!(err, "cannot connect to btcd")
            })
    }
}
