use super::{Home, cleanup};
use super::chain::BitcoinConfig;
use super::al::AbstractLightningNode;

use client::LightningPeach;
use std::{process::Child, io};
use lazycell::LazyCell;
use futures::Future;
use lnd_rust::rpc::LightningAddress;
use interface::routing::Info;

pub struct LpServer {
    peer_port: u16,
    rpc_port: u16,
    home: Home,
}

pub struct LpRunning {
    config: LpServer,
    instance: Child,
    client: LazyCell<LightningPeach>,
    info: LazyCell<Info>,
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
                    info: LazyCell::new(),
                }
            })
    }

}

impl LpRunning {
    // TODO: get rid of duplicated code
    fn obtain_info(&self) -> impl Future<Item=Info, Error=grpc::Error> {
        use interface::{routing_grpc::RoutingService, common::Void};
        use grpc::RequestOptions;

        self.client().routing().get_info(RequestOptions::new(), Void::new())
            .drop_metadata()
    }

    /// might panic
    pub fn client(&self) -> &LightningPeach {
        self.client.borrow().unwrap_or_else(|| {
            self.client.fill(LightningPeach::local(self.config.rpc_port).unwrap()).ok().unwrap();
            self.client()
        })
    }

    /// wait first time, might panic
    pub fn info(&self) -> &Info {
        self.info.borrow().unwrap_or_else(|| {
            self.info.fill(self.obtain_info().wait().unwrap()).ok().unwrap();
            self.info()
        })
    }

    pub fn connect_peer(&self, host: String, pubkey: String) -> impl Future<Item = ()> {
        use interface::{
            routing_grpc::RoutingService,
            routing::{ConnectPeerRequest, LightningAddress}
        };

        let mut address = LightningAddress::new();
        address.set_host(host);
        address.set_pubkey(pubkey);

        let mut request = ConnectPeerRequest::new();
        request.set_address(address);

        self.client().routing()
            .connect_peer(Default::default(), request)
            .drop_metadata()
            .map(|_| ())
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

impl AbstractLightningNode for LpRunning {
    fn address(&self) -> LightningAddress {
        let mut address = LightningAddress::new();
        address.set_host(format!("127.0.0.1:{}", self.config.peer_port));
        address.set_pubkey(self.info().get_identity_pubkey().to_owned());
        address
    }
}
