use super::{Home, cleanup};
use super::chain::BitcoinConfig;
use super::abstract_lightning_node::AbstractLightningNode;
use crate::error::Error;
use crate::{new_io_error, new_grpc_error};

use client::LightningPeach;
use std::{process::Child, io};
use std::thread;
use std::time::Duration;
use lazycell::LazyCell;
use futures::Future;
use lnd_rust::rpc::LightningAddress;
use interface::routing::Info;

pub struct LpServer {
    pub peer_port: u16,
    pub rpc_port: u16,
    pub home: Home,
    pub kill_in_the_end: bool,
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
    ) -> Result<Self, Error> {
        Ok(LpServer {
            peer_port: peer_port,
            rpc_port: rpc_port,
            kill_in_the_end: false,
            home: Home::new(name, false, false)?
            // TODO(mkl): why is this checks everywhere?
//                .or_else(|e| if e.kind() == io::ErrorKind::AlreadyExists {
//                    cleanup("lpd");
//                    Home::new(name, true, true)
//                } else {
//                    Err(e)
//                })?,
        })
    }

    pub fn run<B>(self, b: &B) -> Result<LpRunning, Error>
    where
        B: BitcoinConfig,
    {
        use std::process::Command;

        // TODO: pass lightning arguments into the node
        let _ = b;

        Command::new("cargo")
            .args(&["run", "--package", "server", "--"])
            .args(&[
                format!("--p2p-listen=127.0.0.1:{}", self.peer_port),
                format!("--rpc-listen=127.0.0.1:{}", self.rpc_port),
                format!("--db-path={}", self.home.path().to_string_lossy()),
                format!("--rpc-no-tls"),
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
            .map_err(|err| {
                new_io_error!(err, "cannot start lpd")
            })
    }

}

impl LpRunning {
    // TODO: get rid of duplicated code
    pub fn obtain_info(&self) -> impl Future<Item=Info, Error=grpc::Error> {
        use interface::{routing_grpc::RoutingService, common::Void};
        use grpc::RequestOptions;

        self.client().routing().get_info(RequestOptions::new(), Void::new())
            .drop_metadata()
    }

    pub fn wait_for_sync(&self, max_retries: i32) -> Result<(), Error> {
        let mut i = 0;
        loop {
            let info = self.obtain_info().wait();
            i += 1;
            match info {
                Ok(info) => {
                    return Ok(())
                },
                Err(err) => {
                    if i>= max_retries {
                        return Err(new_grpc_error!(err, "error connecting to lpd. Last error"))
                    }
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
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
        // TODO(mkl): do we really need caching here
        self.info.borrow().unwrap_or_else(|| {
            let data = self.obtain_info().wait()
                .unwrap_or_else(|err| {
                    println!("cannot getinfo lpd: {:?}", err);
                    panic!(err);
                });

            self.info.fill(data)
                .unwrap_or_else(|err|{
                    println!("cannot fill LazyCell with getinfo lpd data: {:?}", err);
                    panic!(err);
                });
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
        if self.config.kill_in_the_end {
            self.instance.kill()
                .or_else(|e| match e.kind() {
                    io::ErrorKind::InvalidInput => Ok(()),
                    _ => Err(e),
                })
                .unwrap()
        }
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
