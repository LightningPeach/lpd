use super::lightning_address::LightningAddress;
use super::network_graph;

use std::io::Error as IoError;
use std::sync::Arc;

use dependencies::grpc::Error as GrpcError;
use dependencies::grpc::{Client, ClientStub};
use dependencies::httpbis::Error as HttpbisError;
use dependencies::futures::future::Future;
use structopt::StructOpt;

use interface::{
    routing_grpc::{RoutingServiceClient, RoutingService},
    routing::{ConnectPeerRequest, LightningAddress as LightningAddressRPC, ChannelGraphRequest},
    common::Void,
};

#[derive(Debug)]
pub enum Error {
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
        address: LightningAddress,
    },

    /// Report graph info
    #[structopt(name="describe-graph")]
    DescribeGraph,

    /// Report graph info in dot format
    #[structopt(name="describe-graph-dot")]
    DescribeGraphDot,
}

impl Command {
    pub fn execute(&self, client: Arc<Client>) -> Result<(), Error> {
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
