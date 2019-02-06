use super::{Home, cleanup};
use super::chain::BitcoinConfig;

use client::LightningPeach;
use std::{process::Child, io};
use lazycell::LazyCell;

pub struct LpServer {
    peer_port: u16,
    rpc_port: u16,
    home: Home,
}

pub struct LpRunning {
    config: LpServer,
    instance: Child,
    client: LazyCell<LightningPeach>,
}

impl LpServer {
    pub fn name(&self) -> &str {
        self.home.name()
    }

    pub fn new(
        peer_port: u16, rpc_port: u16, name: &str
    ) -> Result<Self, io::Error> {
        Ok(LpServer {
            peer_port: peer_port,
            rpc_port: rpc_port,
            home: Home::new(name, false)
                .or_else(|e| if e.kind() == io::ErrorKind::AlreadyExists {
                    cleanup("lpd");
                    Home::new(name, true)
                } else {
                    Err(e)
                })?,
        })
    }

    pub fn run<B>(self, b: &B) -> Result<LpRunning, io::Error>
    where
        B: BitcoinConfig,
    {
        use std::process::Command;

        // TODO: pass lightning arguments into the node
        let _ = b;

        Command::new("lpd")
            .args(&[
                format!("--listen=localhost:{}", self.peer_port),
                format!("--rpclisten=localhost:{}", self.rpc_port),
            ])
            .spawn()
            .map(|instance| {
                LpRunning {
                    config: self,
                    instance: instance,
                    client: LazyCell::new(),
                }
            })
    }

}

impl LpRunning {
    /// might panic
    pub fn client(&self) -> &LightningPeach {
        self.client.borrow().unwrap_or_else(|| {
            self.client.fill(LightningPeach::local(self.config.rpc_port).unwrap()).ok().unwrap();
            self.client()
        })
    }
}

impl Drop for LpRunning {
    fn drop(&mut self) {
        self.instance.kill()
            .or_else(|e| match e.kind() {
                io::ErrorKind::InvalidInput => Ok(()),
                _ => Err(e),
            })
            .unwrap()
    }
}
