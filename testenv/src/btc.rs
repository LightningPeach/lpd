use super::Home;

use std::path::PathBuf;
use std::process::Command;
use std::process::Child;
use std::io;

pub struct BtcDaemon {
    home: Home,
    instance: Option<Child>,
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
            instance: None,
        })
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        self.terminate()?;

        self.instance = Some(Command::new("btcd")
            .args(&["--simnet", "--txindex", "--rpcuser=devuser", "--rpcpass=devpass"])
            .args(&[
                format!("--datadir={}", self.home.ext_path("data").to_str().unwrap()),
                format!("--logdir={}", self.home.ext_path("logs").to_str().unwrap()),
                format!("--rpccert={}", self.home.public_key_path().to_str().unwrap()),
                format!("--rpckey={}", self.home.private_key_path().to_str().unwrap()),
            ])
            .arg("--miningaddr=sb1qvc0mwkl35rl60memjwglxjnz0qsfxhaqq3nx4x")
            .spawn()?
        );

        Ok(())
    }

    pub fn terminate(&mut self) -> Result<(), io::Error> {
        if let Some(ref mut instance) = self.instance {
            instance.kill()?;
        }

        self.instance = None;

        Ok(())
    }

    pub fn with<F>(&mut self, op: F) -> Result<(), io::Error> where F: FnOnce(&Self) {
        self.run()?;
        op(self);
        self.terminate()
    }
}
