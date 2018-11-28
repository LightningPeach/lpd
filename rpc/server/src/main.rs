extern crate grpc;
extern crate tls_api_native_tls;
extern crate tls_api;
extern crate httpbis;

extern crate implementation;

mod config;
use self::config::{Argument, Error as CommandLineReadError};

use grpc::Error as GrpcError;
use httpbis::Error as HttpbisError;

#[derive(Debug)]
enum Error {
    Grpc(GrpcError),
    Httpbis(HttpbisError),
    CommandLineRead(CommandLineReadError),
}

fn main() -> Result<(), Error> {
    use std::thread;
    use grpc::ServerBuilder;
    use implementation::{channel_service, routing_service, payment_service};
    use self::Error::*;

    let argument = Argument::from_env().map_err(CommandLineRead)?;

    let mut server = ServerBuilder::new();
    server.http.set_addr(argument.address).map_err(Httpbis)?;
    if let Some(acceptor) = argument.tls_acceptor {
        server.http.set_tls(acceptor);
    }
    server.http.set_cpu_pool_threads(4);
    server.add_service(channel_service());
    server.add_service(routing_service());
    server.add_service(payment_service());
    let _ = server.build().map_err(Grpc)?;
    loop {
        thread::park();
    }
}
