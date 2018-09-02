use super::Home;

use std::path::PathBuf;
use std::process::Command;
use std::process::Child;
use std::io;
use std::convert::AsRef;

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
            home: Home::new(name)?,
        })
    }

    pub fn run(self) -> Result<BtcRunning, io::Error> {
        Command::new("btcd")
            .args(&["--simnet", "--txindex", "--rpcuser=devuser", "--rpcpass=devpass"])
            .args(&[
                format!("--datadir={}", self.home.ext_path("data").to_str().unwrap()),
                format!("--logdir={}", self.home.ext_path("logs").to_str().unwrap()),
                format!("--rpccert={}", self.home.public_key_path().to_str().unwrap()),
                format!("--rpckey={}", self.home.private_key_path().to_str().unwrap()),
            ])
            .arg("--miningaddr=sb1qvc0mwkl35rl60memjwglxjnz0qsfxhaqq3nx4x")
            .spawn()
            .map(|instance| BtcRunning {
                daemon: self,
                instance: instance,
            })
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
