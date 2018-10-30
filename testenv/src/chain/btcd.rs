use super::super::{Home, cleanup};
use super::{BitcoinConfig, BitcoinInstance};

use std::process::{Command, Child};
use std::{io, fs};

pub struct Btcd {
    home: Home,
}

pub struct BtcdRunning {
    daemon: Btcd,
    instance: Child,
}

impl AsMut<Btcd> for BtcdRunning {
    fn as_mut(&mut self) -> &mut Btcd {
        &mut self.daemon
    }
}

impl AsRef<Btcd> for BtcdRunning {
    fn as_ref(&self) -> &Btcd {
        &self.daemon
    }
}

impl Drop for BtcdRunning {
    fn drop(&mut self) {
        self.instance.kill().unwrap()
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

    fn params(&self) -> Vec<String> {
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
                daemon: self,
                instance: instance,
            })
    }
}

impl BitcoinInstance for BtcdRunning {
    fn set_mining_address(self, address: String) -> Result<Self, io::Error> {
        use std::mem;

        let mut s = self;
        let daemon = Btcd::new("fake")?;
        mem::replace(&mut s.daemon, daemon)
            .run_internal(Some(address))
    }

    fn generate(&mut self, count: usize) -> Result<(), io::Error> {
        Command::new("btcctl")
            .args(&["--simnet", "--rpcuser=devuser", "--rpcpass=devpass"])
            .arg(format!("--rpccert={}", self.daemon.home.public_key_path().to_str().unwrap()))
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
