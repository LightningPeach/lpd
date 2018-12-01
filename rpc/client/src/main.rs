extern crate grpc;
extern crate tls_api;
extern crate tls_api_native_tls;
extern crate tls_api_stub;
extern crate httpbis;
extern crate futures;

extern crate interface;

use grpc::Error as GrpcError;
use httpbis::Error as HttpbisError;

#[derive(Debug)]
enum Error {
    Grpc(GrpcError),
    Httpbis(HttpbisError),
}

fn main() -> Result<(), Error> {
    use std::{sync::Arc, net::{SocketAddr, IpAddr, Ipv4Addr}};
    use grpc::{Client, ClientStub};
    use httpbis::ClientTlsOption;
    use tls_api_stub::TlsConnector;
    use interface::{
        routing_grpc::{RoutingServiceClient, RoutingService},
        routing::ChannelGraphRequest,
    };
    use futures::Future;
    use self::Error::*;

    let default_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9735);
    let client = Arc::new(Client::new_expl::<TlsConnector>(&default_address, default_address.ip().to_string().as_str(), ClientTlsOption::Plain, Default::default()).map_err(Grpc)?);
    let routing_service = RoutingServiceClient::with_client(client);

    let mut request = ChannelGraphRequest::new();
    request.set_include_unannounced(false);
    let graph = routing_service
        .describe_graph(Default::default(), request)
        .drop_metadata().wait().map_err(Grpc)?;

    println!("{:?}", graph);

    Ok(())
}
