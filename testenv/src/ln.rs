use super::Home;
use super::BtcDaemon;

use std::process::Command;
use std::process::Child;
use std::io;

use lnd_rust::rpc::GetInfoResponse;
use lnd_rust::rpc::LightningAddress;
use lnd_rust::rpc::ConnectPeerResponse;
use lnd_rust::rpc::OpenStatusUpdate;

use lnd_rust::rpc_grpc::LightningClient;
use grpc;

use futures::Future;
use futures::Stream;

use lazycell::LazyCell;

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
    client: LazyCell<LightningClient>,
    info: LazyCell<GetInfoResponse>,
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
            .map(|instance| {
                LnRunning {
                    daemon: self,
                    instance: instance,
                    client: LazyCell::new(),
                    info: LazyCell::new(),
                }
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

    fn new_client(&self) -> Result<LightningClient, grpc::Error> {
        use std::net::SocketAddr;
        use std::net::Ipv4Addr;
        use std::net::IpAddr;
        use std::str::FromStr;
        use lnd_rust::tls_certificate::TLSCertificate;

        let daemon = &self.daemon;

        let certificate = TLSCertificate::from_path(daemon.home.public_key_path())
            .map_err(grpc::Error::Io)?;
        let localhost = "127.0.0.1";
        let localhost_ip = IpAddr::V4(Ipv4Addr::from_str(localhost).unwrap());
        let tls = certificate.into_tls(localhost)
            .map_err(|e| grpc::Error::Io(e.into()))?;
        let socket_address = SocketAddr::new(localhost_ip, daemon.rpc_port);
        let conf = Default::default();
        let inner = grpc::Client::new_expl(&socket_address, localhost, tls, conf)?;
        Ok(LightningClient::with_client(inner))
    }

    fn obtain_info(&self) -> impl Future<Item=GetInfoResponse, Error=grpc::Error> {
        use lnd_rust::rpc::GetInfoRequest;
        use lnd_rust::rpc_grpc::Lightning;
        use grpc::RequestOptions;

        self.client()
            .get_info(RequestOptions::new(), GetInfoRequest::new())
            .drop_metadata()
    }

    /// might panic
    pub fn client(&self) -> &LightningClient {
        self.client.borrow().unwrap_or_else(|| {
            self.client.fill(self.new_client().unwrap()).ok().unwrap();
            self.client()
        })
    }

    /// wait first time, might panic
    pub fn info(&self) -> &GetInfoResponse {
        self.info.borrow().unwrap_or_else(|| {
            self.info.fill(self.obtain_info().wait().unwrap()).ok().unwrap();
            self.info()
        })
    }

    pub fn address(&self) -> LightningAddress {
        let mut address = LightningAddress::new();
        address.set_host(format!("127.0.0.1:{}", self.daemon.peer_port));
        address.set_pubkey(self.info().get_identity_pubkey().to_owned());
        address
    }

    pub fn connect_peer(&self, peer: &Self) -> impl Future<Item=ConnectPeerResponse, Error=grpc::Error> {
        use lnd_rust::rpc;
        use lnd_rust::rpc_grpc::Lightning;
        use grpc::RequestOptions;

        let mut request = rpc::ConnectPeerRequest::new();
        request.set_addr(peer.address());
        self.client()
            .connect_peer(RequestOptions::new(), request)
            .drop_metadata()
    }

    pub fn open_channel(&self, peer: &Self) -> impl Stream<Item=OpenStatusUpdate, Error=grpc::Error> {
        use lnd_rust::rpc;
        use lnd_rust::rpc_grpc::Lightning;
        use grpc::RequestOptions;

        let peer_address = peer.address();
        let mut request = rpc::OpenChannelRequest::new();
        request.set_node_pubkey_string(peer_address.get_pubkey().to_owned());
        request.set_local_funding_amount(100000);
        request.set_min_htlc_msat(1000);
        request.set_push_sat(0);
        request.set_remote_csv_delay(144);
        request.set_sat_per_byte(12500);
        request.set_target_conf(0);
        self.client()
            .open_channel(RequestOptions::new(), request)
            .drop_metadata()
    }
}

impl<'a> Drop for LnRunning<'a> {
    fn drop(&mut self) {
        self.instance.kill().unwrap()
    }
}
