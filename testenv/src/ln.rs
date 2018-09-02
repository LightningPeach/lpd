use super::Home;
use super::BtcDaemon;

use std::net::SocketAddr;
use std::net::Ipv4Addr;
use std::net::IpAddr;
use std::str::FromStr;

use std::process::Command;
use std::process::Child;
use std::io;

use lnd_rust::rpc::LightningAddress;
use lnd_rust::rpc_grpc::LightningClient;
use grpc;
use futures::Future;

pub struct LnDaemon<'a> {
    peer_port: u16,
    rpc_port: u16,
    rest_port: u16,
    home: Home,
    btcd: &'a BtcDaemon,
}

pub struct LnRunning<'a> {
    daemon: LnDaemon<'a>,
    instance: Child,
    client: LightningClient,
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
        })
    }

    pub fn run(self) -> Result<LnRunning<'a>, io::Error> {
        Command::new("lnd")
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
            .spawn()
            .and_then(|instance| {
                use lnd_rust::tls_certificate::TLSCertificate;

                let certificate = TLSCertificate::from_path(self.home.public_key_path())?;
                let localhost = "127.0.0.1";
                let tls = certificate.into_tls(localhost)?;
                let socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str(localhost).unwrap()), self.rpc_port);
                let conf = Default::default();
                let c = grpc::Client::new_expl(&socket_address, localhost, tls, conf)?;

                Ok(LnRunning {
                    daemon: self,
                    instance: instance,
                    client: LightningClient::with_client(c),
                })
            })
    }
}

impl<'a> LnRunning<'a> {
    // errors ignored
    pub fn batch(limit: u16, base_port: u16, btcd: &'a BtcDaemon) -> Vec<Self> {
        (0..limit).into_iter()
            .map(|index| -> Result<LnRunning, io::Error> {
                let p_peer = base_port + index * 10;
                let p_rpc = base_port + index * 10 + 1;
                let p_rest = base_port + index * 10 + 2;
                let name = format!("lnd-node-{}", index);
                LnDaemon::new(
                    p_peer, p_rpc, p_rest, name.as_str(), &btcd
                )?.run()
            })
            .filter_map(Result::ok)
            .collect()
    }

    pub fn client(&self) -> &LightningClient {
        &self.client
    }

    pub fn address(&self) -> impl Future<Item=LightningAddress, Error=grpc::Error> {
        use lnd_rust::rpc;
        use lnd_rust::rpc_grpc::Lightning;
        use grpc::RequestOptions;

        let mut address = LightningAddress::new();
        address.set_host(format!("127.0.0.1:{}", self.daemon.peer_port));

        self.client
            .get_info(RequestOptions::new(), rpc::GetInfoRequest::new())
            .drop_metadata()
            .map(move |response| {
                address.set_pubkey(response.identity_pubkey);
                address
            })
    }
}

impl<'a> Drop for LnRunning<'a> {
    fn drop(&mut self) {
        self.instance.kill().unwrap()
    }
}
