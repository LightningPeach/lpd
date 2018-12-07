extern crate grpc;
extern crate tls_api_native_tls;
extern crate tls_api;
extern crate httpbis;

extern crate implementation;

mod config;
use self::config::{Argument, Error as CommandLineReadError};

use grpc::Error as GrpcError;
use httpbis::Error as HttpbisError;
use implementation::DBError;
use std::io::Error as IoError;

#[derive(Debug)]
enum Error {
    Grpc(GrpcError),
    Httpbis(HttpbisError),
    CommandLineRead(CommandLineReadError),
    Database(DBError),
    Io(IoError),
}

fn main() -> Result<(), Error> {
    use std::{sync::{RwLock, Arc}, io::{Read, stdin, stdout, Write}};
    use grpc::ServerBuilder;
    use implementation::{channel_service, routing_service, payment_service, State};
    use self::Error::*;

    let argument = Argument::from_env().map_err(CommandLineRead)?;

    let state = Arc::new(RwLock::new(State::new(argument.db_path.as_str()).map_err(Database)?));
    state.write().unwrap().load().map_err(Database)?;

    let mut server = ServerBuilder::new();
    if let Some(acceptor) = argument.tls_acceptor {
        server.http.set_tls(acceptor);
    }
    server.http.set_addr(argument.address).map_err(Httpbis)?;
    server.http.set_cpu_pool_threads(4);
    server.add_service(routing_service(state));
    server.add_service(channel_service());
    server.add_service(payment_service());
    let server = server.build().map_err(Grpc)?;

    write!(stdout(), "\
        the lightning peach node is running at: {}, database at: {}\n\
        enter any to shutdown... ", argument.address, argument.db_path);
    stdout().flush().map_err(Io)?;
    stdin().read(&mut [0]).map_err(Io)?;

    let _ = server;
    Ok(())
}
