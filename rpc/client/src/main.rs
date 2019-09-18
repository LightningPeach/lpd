mod network_graph;

use std::path::PathBuf;
use std::str::FromStr;
use std::fs::File;
use std::io::Error as IoError;
use std::net::SocketAddr;
use std::sync::Arc;

use dependencies::futures;
use dependencies::httpbis;
use dependencies::tls_api_rustls;
use dependencies::hex;

use grpc::Error as GrpcError;
use grpc::{Client, ClientStub};
use httpbis::Error as HttpbisError;
use httpbis::ClientTlsOption;
use structopt::StructOpt;
use tls_api_rustls::TlsConnector;
use futures::future::Future;

use interface::{
    routing_grpc::{RoutingServiceClient, RoutingService},
    routing::{ConnectPeerRequest, LightningAddress as LightningAddressRPC, ChannelGraphRequest},
    common::Void,
};
use build_info::get_build_info;

#[derive(Debug)]
enum Error {
    Grpc(GrpcError),
    Httpbis(HttpbisError),
    IoError{
        inner: IoError,
        description: String,
    },
}

impl Error {
    pub fn new_io_error(err: IoError, description: &str) -> Self {
        Error::IoError {
            inner: err,
            description: description.to_owned(),
        }
    }
}

// TODO(mkl): allow usage of domain names
#[derive(Debug)]
pub struct LightningAddress {
    pub pub_key: String,
    pub host: SocketAddr,
}

impl FromStr for LightningAddress {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("@").collect();
        if parts.len() !=2 {
            return Err(format!("Lightning address is expected in format: pubkey@host:port, got {}", s))
        }
        let pub_key = parts[0];
        if let Ok(pk) = hex::decode(pub_key) {
            if pk.len() != 33 {
                return Err(format!("Wrong length of public key. Got {}, want {}", pk.len(), 33))
            }
        } else {
            return Err(format!("Cannot decode hex-encoded public key"));
        }
        let addr = SocketAddr::from_str(parts[1]).map_err(|err|{
            format!("cannot parse socket address: {} : {:?}", parts[1], err)
        })?;
        Ok(LightningAddress {
            pub_key: pub_key.to_owned(),
            host: addr,
        })
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "lpd-client")]
/// Lightning Peach Client. Client for LPD (Lightning peach daemon)
pub struct Config {
    /// Address to connect
    #[structopt(long="addr", default_value="127.0.0.1:10009")]
    pub address: SocketAddr,

    /// Do not use TLS for rpc connections
    #[structopt(long="no-tls")]
    pub no_tls: bool,

    /// Path to cert file for TLS for RPC connections
    #[structopt(long="tls-cert-path", parse(from_os_str))]
    pub tls_cert_path: Option<PathBuf>,

    /// Print configuration information and exit. Useful for debugging
    #[structopt(long="print-config")]
    pub print_config: bool,

    /// Print detailed version
    #[structopt(long="version")]
    pub print_version: bool,

    #[structopt(subcommand)]
    pub command: Option<Command>,
}

#[derive(StructOpt, Debug)]
#[structopt()]
pub enum Command{
    /// Get general info
    #[structopt(name="get-info")]
    GetInfo,

    /// Connect to specified peer
    #[structopt(name="connect-peer")]
    ConnectPeer{
        #[structopt()]
        address: LightningAddress
    },

    /// Report graph info
    #[structopt(name="describe-graph")]
    DescribeGraph,

    /// Report graph info in dot format
    #[structopt(name="describe-graph-dot")]
    DescribeGraphDot,
}

impl Command {
    fn execute(&self, client: Arc<Client>) -> Result<(), Error> {
        let routing_service = RoutingServiceClient::with_client(client);
        match self {
            Self::GetInfo => {
                let response = routing_service
                    .get_info(Default::default(), Void::new())
                    .drop_metadata().wait().map_err(Error::Grpc)?;
                println!("{:?}", response);
                Ok(())
            },
            Self::ConnectPeer{address} => {
                let mut request = ConnectPeerRequest::new();

                let mut lightning_address_rpc = LightningAddressRPC::new();
                lightning_address_rpc.set_pubkey(address.pub_key.clone());
                lightning_address_rpc.set_host(format!("{}", address.host));

                request.set_address(lightning_address_rpc);
                let response = routing_service
                    .connect_peer(Default::default(), request)
                    .drop_metadata().wait().map_err(Error::Grpc)?;
                println!("{:?}", response);
                Ok(())
            },
            Self::DescribeGraph => {
                let mut request = ChannelGraphRequest::new();
                request.set_include_unannounced(false);
                let response = routing_service
                    .describe_graph(Default::default(), request)
                    .drop_metadata().wait().map_err(Error::Grpc)?;
                println!("{:?}", response);
                Ok(())
            },
            Self::DescribeGraphDot => {
                let mut request = ChannelGraphRequest::new();
                request.set_include_unannounced(false);
                let response = routing_service
                    .describe_graph(Default::default(), request)
                    .drop_metadata().wait().map_err(Error::Grpc)?;
                let dot = network_graph::dot_format(response);
                println!("{:?}", dot);
                Ok(())
            },
        }
    }
}

fn print_version(){
    println!("{:#?}", get_build_info!());
}

fn main() -> Result<(), Error> {
    let config: Config = Config::from_args();
    if config.print_version {
        print_version();
        return Ok(());
    }
    if config.print_config {
        println!("{:#?}", &config);
        return Ok(());
    }
    if config.command.is_none() {
        eprintln!("Please specify a subcommand");
        std::process::exit(1);
    }
    let client = if !config.no_tls {
//        let cert_bytes = {
//            let mut file = File::open(cert_path)
//                .map_err(|err|
//                    Error::new_io_error(err, &format!("cannot open certificate file: {:?}", &cert_path))
//                )?;
//            let mut vec = Vec::new();
//            file.read_to_end(&mut vec).map_err(|err|
//                Error::new_io_error(err, &format!("cannot read certificate file: {:?}", &cert_path))
//            )?;
//            vec
//        };
        panic!("TLS is currently not implemented");
    } else {
        Client::new_expl::<TlsConnector>(
            &config.address,
            config.address.ip().to_string().as_str(),
            ClientTlsOption::Plain,
            Default::default()
        ).map_err(Error::Grpc)?
    };
    if config.command.is_some() {
        config.command.unwrap().execute(Arc::new(client))
    } else {
        Ok(())
    }
}
