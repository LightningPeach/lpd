use super::{Home, cleanup};
use super::chain::BitcoinConfig;
use super::al::AbstractLightningNode;
use crate::home::create_file_for_redirect;
use crate::error::Error;
use crate::new_io_error;

use std::process::{Command, Child};

use std::io;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;

use lnd_rust::rpc::GetInfoResponse;
use lnd_rust::rpc::LightningAddress;
use lnd_rust::rpc::ConnectPeerResponse;
use lnd_rust::rpc::OpenStatusUpdate;

use lnd_rust::rpc_grpc::LightningClient;
use grpc;

use futures::Future;
use futures::Stream;

use lazycell::LazyCell;


/// LndConfig represents configuration for `lnd` (Lightning Network Daemon)
/// Some options are specified, some are derived
#[derive(Debug, Clone)]
pub struct LndConfig {
    /// Peer port. At this port `lnd` is listening for incoming peer connections
    pub peer_port: u16,

    /// RPC port. At this port `lnd` is listening for GRPC API connections
    pub rpc_port: u16,

    /// REST API port. At this port `lnd` is listening for HTTP REST API requests
    pub rest_port: u16,

    /// Working dir
    pub home: Home,
}

/// `LndProcess` represents running instance of `lnd`
pub struct LndProcess {
    /// Configuration info
    config: LndConfig,

    process: Child,

    rpc_client: LazyCell<LightningClient>,

    info: LazyCell<GetInfoResponse>,

    // we should keep file here, because its descriptor is used for redirects
    // if file is closed then descriptor becomes invalid (I guess)
    stdout: File,
    stderr: File
}

impl fmt::Debug for LndProcess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LnRunning")
            .field("config", &self.config)
            .field("instance", &self.process)
            .finish()
    }
}


impl LndConfig {
    // TODO(mkl): maybe change it to root path
    pub fn name(&self) -> &str {
        self.home.name()
    }

    pub fn data_dir_path(&self) -> PathBuf {
        self.home.ext_path("data")
    }

    pub fn log_dir_path(&self) -> PathBuf {
        self.home.ext_path("logs")
    }

    pub fn stdout_path(&self) -> PathBuf {
        self.home.ext_path("lnd.stdout")
    }

    pub fn stderr_path(&self) -> PathBuf {
        self.home.ext_path("lnd.stderr")
    }

    pub fn new(
        peer_port: u16, rpc_port: u16, rest_port: u16, name: &str
    ) -> Result<Self, Error> {
        // TODO(mkl): new should not do any changes
        let home = Home::new(name, false, false)?;

        Ok(LndConfig {
            peer_port: peer_port,
            rpc_port: rpc_port,
            rest_port: rest_port,
            home: home,
        })
    }

    /// Returns lnd arguments only connected with `lnd`
    /// They do not contains e.g. configuration for bitcoind connection
    pub fn get_pure_lnd_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec![];
        // TODO(mkl): make it configurable
        args.push("--noseedbackup".to_owned());

        // TODO(mkl): make it configurable
        args.push("--no-macaroons".to_owned());

        // TODO(mkl): make it configurable. Add enum for trace Leveles
        args.push("--debuglevel=trace".to_owned());

        // File path related stuff
        args.push( format!("--lnddir={}", self.home.path().to_str().unwrap()) );
        args.push( format!("--configfile={}", self.home.lnd_conf_path().to_str().unwrap()) );
        args.push( format!("--datadir={}", self.data_dir_path().to_str().unwrap()) );
        args.push( format!("--logdir={}", self.log_dir_path().to_str().unwrap()) );
        args.push( format!("--tlscertpath={}", self.home.public_key_path().to_str().unwrap()) );
        args.push( format!("--tlskeypath={}", self.home.private_key_path().to_str().unwrap()) );

        // Ports to listen
        args.push( format!("--listen=localhost:{}", self.peer_port) );
        args.push( format!("--rpclisten=localhost:{}", self.rpc_port) );
        args.push( format!("--restlisten=localhost:{}", self.rest_port) );

        args
    }

    pub fn run<B>(self, b: &B) -> Result<LndProcess, Error>
    where
        B: BitcoinConfig,
    {
        println!("lnd stdout: {:?}", self.stdout_path());
        println!("lnd stderr: {:?}", self.stderr_path());
        let (stdout, stdout_file) = create_file_for_redirect(self.stdout_path()).map_err(|err| {
            new_io_error!(
                err,
                "cannot create file for redirecting lnd stdout",
                self.stdout_path().to_string_lossy().into_owned()
            )
        })?;

        let (stderr, stderr_file) = create_file_for_redirect(self.stderr_path()).map_err(|err|{
            new_io_error!(
                err,
                "cannot create file for redirecting lnd stderr",
                self.stderr_path().to_string_lossy().into_owned()
            )
        })?;

        println!("self.home.ext_path(\"data\"): {:?}", self.data_dir_path());
        Command::new("lnd")
            .args(self.get_pure_lnd_args())
            .args(b.lnd_params())
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .map_err(|err|{
                new_io_error!(err, "cannot spawn lnd")
            })
            .map(|child_process| {
                LndProcess {
                    config: self,
                    process: child_process,
                    rpc_client: LazyCell::new(),
                    info: LazyCell::new(),
                    stdout: stdout_file,
                    stderr: stderr_file
                }
            })
    }
}

impl LndProcess {
    // errors ignored
    pub fn batch<B>(limit: u16, base_port: u16, b: &B) -> Vec<Self>
    where
        B: BitcoinConfig,
    {
        (0..limit).into_iter()
            .map(|index| -> Result<LndProcess, Error> {
                // TODO(mkl): move port determination logic into some file
                let p_peer = base_port + index * 10;
                let p_rpc = base_port + index * 10 + 1;
                let p_rest = base_port + index * 10 + 2;
                let name = format!("lnd-node-{}", index);
                LndConfig::new(
                    p_peer, p_rpc, p_rest, name.as_str()
                )?.run(b)
            })
            .inspect(|x| {
                println!("LND start result: {:?}", x);
            })
            .filter_map(Result::ok)
            .collect()
    }

    fn new_client(&self) -> Result<LightningClient, grpc::Error> {
        use std::{net::{SocketAddr, Ipv4Addr, IpAddr}, sync::Arc};
        use std::str::FromStr;
        use lnd_rust::tls_certificate::TLSCertificate;
        use grpc::ClientStub;

        let daemon = &self.config;

        let certificate = TLSCertificate::from_path(daemon.home.public_key_path())
            .map_err(grpc::Error::Io)?;
        let localhost = "127.0.0.1";
        let localhost_ip = IpAddr::V4(Ipv4Addr::from_str(localhost).unwrap());
        let tls = certificate.into_tls(localhost)
            .map_err(|e| grpc::Error::Io(e.into()))?;
        let socket_address = SocketAddr::new(localhost_ip, daemon.rpc_port);
        let conf = Default::default();
        let inner = grpc::Client::new_expl(&socket_address, localhost, tls, conf)?;
        Ok(LightningClient::with_client(Arc::new(inner)))
    }

    pub fn obtain_info(&self) -> impl Future<Item=GetInfoResponse, Error=grpc::Error> {
        use lnd_rust::rpc::GetInfoRequest;
        use lnd_rust::rpc_grpc::Lightning;
        use grpc::RequestOptions;

        self.client()
            .get_info(RequestOptions::new(), GetInfoRequest::new())
            .drop_metadata()
    }

    /// might panic
    pub fn client(&self) -> &LightningClient {
        self.rpc_client.borrow().unwrap_or_else(|| {
            self.rpc_client.fill(self.new_client().unwrap()).ok().unwrap();
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

    pub fn new_address(&self) -> impl Future<Item=String, Error=grpc::Error> {
        use lnd_rust::rpc;
        use lnd_rust::rpc_grpc::Lightning;
        use grpc::RequestOptions;

        let mut request = rpc::NewAddressRequest::new();
        request.set_field_type(rpc::AddressType::WITNESS_PUBKEY_HASH);
        self.client()
            .new_address(RequestOptions::new(), request)
            .drop_metadata()
            .map(|r| r.address)
    }

    pub fn connect_peer<N>(&self, peer: &N) -> impl Future<Item=ConnectPeerResponse, Error=grpc::Error>
    where
        N: AbstractLightningNode,
    {
        use lnd_rust::rpc;
        use lnd_rust::rpc_grpc::Lightning;
        use grpc::RequestOptions;

        let mut request = rpc::ConnectPeerRequest::new();
        request.set_addr(peer.address());
        self.client()
            .connect_peer(RequestOptions::new(), request)
            .drop_metadata()
    }

    // TODO:
    pub fn open_channel(&self, peer_pubkey: &str) -> impl Stream<Item=OpenStatusUpdate, Error=grpc::Error> {
        use lnd_rust::rpc;
        use lnd_rust::rpc_grpc::Lightning;
        use grpc::RequestOptions;
        use hex::FromHex;

        let mut request = rpc::OpenChannelRequest::new();
        request.set_node_pubkey_string(peer_pubkey.to_owned());
        request.set_node_pubkey(Vec::from_hex(peer_pubkey).unwrap());
        request.set_local_funding_amount(1000000);
        request.set_min_htlc_msat(10000);
        request.set_push_sat(1000);
        request.set_remote_csv_delay(144);
        request.set_sat_per_byte(12500);
        request.set_target_conf(6);
        request.set_private(false);
        self.client()
            .open_channel(RequestOptions::new(), request)
            .drop_metadata()
    }
}

impl Drop for LndProcess {
    fn drop(&mut self) {
        self.process.kill()
            .or_else(|e| match e.kind() {
                io::ErrorKind::InvalidInput => Ok(()),
                _ => Err(e),
            })
            .unwrap()
    }
}

impl AbstractLightningNode for LndProcess {
    fn address(&self) -> LightningAddress {
        let mut address = LightningAddress::new();
        address.set_host(format!("127.0.0.1:{}", self.config.peer_port));
        address.set_pubkey(self.info().get_identity_pubkey().to_owned());
        address
    }
}
