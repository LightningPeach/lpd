use super::{Home, cleanup};

use std::path::PathBuf;
use std::process::Command;
use std::process::Child;
use std::{io, fs};
use std::convert::AsRef;
use std::convert::AsMut;

pub struct BtcDaemon {
    home: Home,
}

pub struct BtcRunning {
    daemon: BtcDaemon,
    instance: Child,
}

impl BtcDaemon {
    pub fn name(&self) -> &str {
        self.home.name()
    }

    pub fn public_key_path(&self) -> PathBuf {
        self.home.public_key_path()
    }

    pub fn new(name: &str) -> Result<Self, io::Error> {
        Ok(BtcDaemon {
            home: Home::new(name, false)
                .or_else(|e| if e.kind() == io::ErrorKind::AlreadyExists {
                    cleanup("btcd");
                    cleanup("bitcoind");
                    Home::new(name, true)
                } else {
                    Err(e)
                })?,
        })
    }

    fn run_internal(self, mining_address: Option<String>) -> Result<BtcRunning, io::Error> {
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
            .args(&["--simnet", "--txindex", "--rpcuser=devuser", "--rpcpass=devpass"])
            .args(args)
            .spawn()
            .map(|instance| BtcRunning {
                daemon: self,
                instance: instance,
            })
    }

    pub fn run(self) -> Result<BtcRunning, io::Error> {
        self.run_internal(None)
    }
}

impl BtcRunning {
    pub fn rerun_with_mining_address(self, address: String) -> Result<BtcRunning, io::Error> {
        use std::mem;

        let mut s = self;
        let daemon = BtcDaemon::new("fake")?;
        mem::replace(&mut s.daemon, daemon)
            .run_internal(Some(address))
    }

    pub fn generate(&mut self, count: usize) -> Result<(), io::Error> {
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
                    // TODO: process it
                    panic!()
                }
            )
    }
}

impl AsMut<BtcDaemon> for BtcRunning {
    fn as_mut(&mut self) -> &mut BtcDaemon {
        &mut self.daemon
    }
}

impl AsRef<BtcDaemon> for BtcRunning {
    fn as_ref(&self) -> &BtcDaemon {
        &self.daemon
    }
}

impl Drop for BtcRunning {
    fn drop(&mut self) {
        self.instance.kill().unwrap()
    }
}
