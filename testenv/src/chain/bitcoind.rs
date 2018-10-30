use super::super::{Home, cleanup};
use super::{BitcoinConfig, BitcoinInstance};

use std::process::{Command, Child};
use std::{io, fs};

pub struct Bitcoind {
    home: Home,
}

pub struct BitcoindRunning {
    daemon: Bitcoind,
    instance: Child,
}

impl AsMut<Bitcoind> for BitcoindRunning {
    fn as_mut(&mut self) -> &mut Bitcoind {
        &mut self.daemon
    }
}

impl AsRef<Bitcoind> for BitcoindRunning {
    fn as_ref(&self) -> &Bitcoind {
        &self.daemon
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
        self.run_internal(None)
    }

    fn params(&self) -> Vec<String> {
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
    fn run_internal(self, mining_address: Option<String>) -> Result<BitcoindRunning, io::Error> {
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
            format!("-datadir={}", self.home.ext_path("data").to_str().unwrap()),
            format!("-logdir={}", self.home.ext_path("logs").to_str().unwrap()),
        ];

        if let Some(mining_address) = mining_address {
            args.push(format!("-miningaddr={}", mining_address));
        }

        Command::new("bitcoind")
            .args(&[
                "-regtest", "-server", "-txindex", "-rpcuser=devuser", "-rpcpassword=devpass",
                "-rpcport=18443",
                "-zmqpubrawblock=tcp://127.0.0.1:18501", "-zmqpubrawtx=tcp://127.0.0.1:18501",
            ])
            .args(args)
            .spawn()
            .map(|instance| BitcoindRunning {
                daemon: self,
                instance: instance,
            })
    }
}

impl BitcoinInstance for BitcoindRunning {
    fn set_mining_address(self, address: String) -> Result<Self, io::Error> {
        use std::mem;

        let mut s = self;
        let daemon = Bitcoind::new("fake")?;
        mem::replace(&mut s.daemon, daemon)
            .run_internal(Some(address))
    }

    fn generate(&mut self, count: usize) -> Result<(), io::Error> {
        Command::new("bitcoin-cli")
            .args(&["-regtest", "-rpcuser=devuser", "-rpcpassword=devpass"])
            .arg(format!("-datadir={}", self.daemon.home.ext_path("data").to_str().unwrap()))
            .arg(format!("-rpccert={}", self.daemon.home.public_key_path().to_str().unwrap()))
            .args(&["generate"])
            .args(&[format!("{}", count)])
            .output()
            .and_then(|output|
                if output.status.success() {
                    Ok(())
                } else {
                    panic!()
                }
            )
    }
}
