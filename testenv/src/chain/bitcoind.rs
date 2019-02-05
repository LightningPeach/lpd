use super::super::{Home, cleanup};
use super::{BitcoinConfig, BitcoinInstance};

use std::process::{Command, Child};
use std::{io, fs};
use bitcoin_rpc_client::BitcoinCoreClient;

pub struct Bitcoind {
    home: Home,
}

pub struct BitcoindRunning {
    config: Bitcoind,
    instance: Child,
}

impl AsMut<Bitcoind> for BitcoindRunning {
    fn as_mut(&mut self) -> &mut Bitcoind {
        &mut self.config
    }
}

impl AsRef<Bitcoind> for BitcoindRunning {
    fn as_ref(&self) -> &Bitcoind {
        &self.config
    }
}

impl Drop for BitcoindRunning {
    fn drop(&mut self) {
        self.instance.kill().unwrap()
    }
}

impl BitcoinConfig for Bitcoind {
    type Instance = BitcoindRunning;

    fn new(name: &str) -> Result<Self, io::Error> {
        Ok(Bitcoind {
            home: Home::new(name, false)
                .or_else(|e| if e.kind() == io::ErrorKind::AlreadyExists {
                    cleanup("bitcoind");
                    Home::new(name, true)
                } else {
                    Err(e)
                })?,
        })
    }

    fn run(self) -> Result<Self::Instance, io::Error> {
        self.run_internal()
    }

    fn lnd_params(&self) -> Vec<String> {
        vec![
            "--bitcoin.active".to_owned(),
            "--bitcoin.regtest".to_owned(),
            "--bitcoin.node=bitcoind".to_owned(),
            "--bitcoind.rpcuser=devuser".to_owned(),
            "--bitcoind.rpcpass=devpass".to_owned(),
            "--bitcoind.zmqpubrawblock=tcp://127.0.0.1:18501".to_owned(),
            "--bitcoind.zmqpubrawtx=tcp://127.0.0.1:18501".to_owned(),
            "--bitcoind.rpchost=localhost:18443".to_owned(),
            format!("--bitcoind.dir={}", self.home.path().to_str().unwrap())
        ]
    }
}

impl Bitcoind {
    fn run_internal(self) -> Result<BitcoindRunning, io::Error> {
        fs::create_dir(self.home.ext_path("data")).or_else(|e|
            if e.kind() == io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(e)
            }
        )?;
        let args = vec![
            format!("-datadir={}", self.home.ext_path("data").to_str().unwrap()),
        ];

        Command::new("bitcoind")
            .args(&[
                "-regtest", "-server", "-txindex", "-rpcuser=devuser", "-rpcpassword=devpass",
                "-rpcport=18443",
                "-zmqpubrawblock=tcp://127.0.0.1:18501", "-zmqpubrawtx=tcp://127.0.0.1:18501",
            ])
            .args(args)
            .spawn()
            .map(|instance| BitcoindRunning {
                config: self,
                instance: instance,
            })
    }
}

impl BitcoinInstance for BitcoindRunning {
    fn rpc_client(&self) -> BitcoinCoreClient {
        BitcoinCoreClient::new("tcp://localhost:18443", "devuser", "devpass")
    }
}
