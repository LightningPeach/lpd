use super::Home;
use super::BtcDaemon;

use std::net::SocketAddr;
use std::net::Ipv4Addr;
use std::net::IpAddr;
use std::str::FromStr;

use std::process::Command;
use std::process::Child;
use std::io;
use std::fs::File;

use lnd_rust::rpc::LightningAddress;
use lnd_rust::rpc_grpc::LightningClient;
use lnd_rust::tls_certificate::TLSCertificate;
use grpc;

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

    pub fn client(&self) -> Result<LightningClient, io::Error> {
        let certificate = TLSCertificate::from_der_path(self.home.public_key_path())?;
        let localhost = "127.0.0.1";
        let conf = Default::default();
        let tls = certificate.into_tls(localhost)?;
        let socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str(localhost).unwrap()), self.rpc_port);
        let c = grpc::Client::new_expl(&socket_address, localhost, tls, conf)?;
        Ok(LightningClient::with_client(c))
    }

    pub fn address(&self) -> Result<LightningAddress, io::Error> {
        let mut address = LightningAddress::new();
        address.set_host(format!("127.0.0.1:{}", self.peer_port));
        let public_key = {
            let mut public_key_file = File::open(self.home.public_key_path())?;
            let mut public_key = String::new();
            io::Read::read_to_string(&mut public_key_file, &mut public_key)?;
            public_key
        };
        address.set_pubkey(public_key);
        Ok(address)
    }
}
