#![forbid(unsafe_code)]

extern crate wire;
extern crate brontide;

mod home;
use self::home::Home;

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

pub struct LnDaemon<'a> {
    peer_port: u16,
    rpc_port: u16,
    rest_port: u16,
    home: Home,
    btcd: &'a BtcDaemon,
    instance: Option<Child>,
}

impl<'a> LnDaemon<'a> {
    pub fn name(&self) -> &str {
        self.home.name()
    }

    pub fn new(
        peer_port: u16, rpc_port: u16, rest_port: u16, name: &str, btcd: &'a BtcDaemon
    ) -> Result<Self, io::Error> {
        Ok(LnDaemon {
            peer_port: peer_port,
            rpc_port: rpc_port,
            rest_port: rest_port,
            home: Home::new(name)?,
            btcd: btcd,
            instance: None,
        })
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        self.terminate()?;

        self.instance = Some(Command::new("lnd")
            .args(&[
                "--bitcoin.active", "--bitcoin.simnet", "--noencryptwallet", "--no-macaroons",
                "--btcd.rpcuser=devuser", "--btcd.rpcpass=devpass"
            ])
            .args(&[
                format!("--datadir={}", self.home.ext_path("data").to_str().unwrap()),
                format!("--logdir={}", self.home.ext_path("logs").to_str().unwrap()),
                format!("--tlscertpath={}", self.home.public_key_path().to_str().unwrap()),
                format!("--tlskeypath={}", self.home.private_key_path().to_str().unwrap()),
                format!("--btcd.rpccert={}", self.btcd.public_key_path().to_str().unwrap()),
            ])
            .args(&[
                format!("--listen=localhost:{}", self.peer_port),
                format!("--rpclisten=localhost:{}", self.rpc_port),
                format!("--restlisten=localhost:{}", self.rest_port),
            ])
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

    // errors ignored
    pub fn with<F, T>(limit: u16, base_port: u16, btcd: &'a BtcDaemon, op: F) -> T where
        F: FnOnce(&mut Vec<Self>) -> T
    {
        let mut nodes = (0..limit).into_iter()
            .map(|index| -> Result<LnDaemon, io::Error> {
                let p_peer = base_port + index * 10;
                let p_rpc = base_port + index * 10 + 1;
                let p_rest = base_port + index * 10 + 2;
                let name = format!("lnd-node-{}", index);
                let mut lnd = LnDaemon::new(
                    p_peer, p_rpc, p_rest, name.as_str(), &btcd
                )?;
                lnd.run()?;
                Ok(lnd)
            })
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        let value = op(&mut nodes);
        nodes.iter_mut().for_each(|node| node.terminate().unwrap_or(()));
        value
    }
}

#[cfg(test)]
mod tests {
    use super::BtcDaemon;
    use super::LnDaemon;

    #[test]
    fn run_btcd_lnd() {
        use std::thread;
        use std::time::Duration;

        BtcDaemon::new("btcd")
            .unwrap()
            .with(|btcd| {
                LnDaemon::with(5, 10000, btcd, |_|
                    thread::sleep(Duration::from_secs(100))
                )
            })
            .unwrap();
    }
}
