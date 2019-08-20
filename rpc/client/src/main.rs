use dependencies::futures;
use dependencies::httpbis;
use dependencies::tls_api_rustls;

use grpc::Error as GrpcError;
use httpbis::Error as HttpbisError;

#[derive(Debug)]
enum Error {
    Grpc(GrpcError),
    Httpbis(HttpbisError),
}

fn main() -> Result<(), Error> {
    // TODO(mkl): add aditional commands
    // use some lib for argument parsing
    use std::{sync::Arc, net::{SocketAddr, IpAddr, Ipv4Addr}};
    use grpc::{Client, ClientStub};
    use httpbis::ClientTlsOption;
    use tls_api_rustls::TlsConnector;
    use interface::{
        routing_grpc::{RoutingServiceClient, RoutingService},
        routing::{ConnectPeerRequest, LightningAddress, ChannelGraphRequest},
        common::Void,
    };
    use futures::Future;
    use self::Error::*;
    use std::env;

    let default_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10009);
    let client = Arc::new(Client::new_expl::<TlsConnector>(&default_address, default_address.ip().to_string().as_str(), ClientTlsOption::Plain, Default::default()).map_err(Grpc)?);
    let routing_service = RoutingServiceClient::with_client(client);

    let mut args = env::args();
    args.next();

    let command = args.next().unwrap();
    if command.eq("getinfo") {
        let response = routing_service
            .get_info(Default::default(), Void::new())
            .drop_metadata().wait().map_err(Grpc)?;
        println!("{:?}", response);
    } else if command.eq("connect") {
        let mut request = ConnectPeerRequest::new();
        let mut lightning_address = LightningAddress::new();
        let address = args.next().expect("target address expected in format: pubkey@host:port");
        let parts: Vec<&str> = address.split("@").collect();
        assert_eq!(parts.len(), 2);
        lightning_address.set_pubkey(parts[0].to_owned());
        lightning_address.set_host(parts[1].to_owned());
        request.set_address(lightning_address);
        let response = routing_service
            .connect_peer(Default::default(), request)
            .drop_metadata().wait().map_err(Grpc)?;
        println!("{:?}", response);
    } else if command.eq("describegraph") {
        let mut request = ChannelGraphRequest::new();
        request.set_include_unannounced(false);
        let response = routing_service
            .describe_graph(Default::default(), request)
            .drop_metadata().wait().map_err(Grpc)?;
        println!("{:?}", response);
    } else {
        panic!("wrong command");
    }

    Ok(())
}
