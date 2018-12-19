extern crate grpc;
extern crate tls_api_native_tls;
extern crate tls_api;
extern crate httpbis;

extern crate implementation;

mod config;
use self::config::{Argument, Error as CommandLineReadError};

use grpc::Error as GrpcError;
use httpbis::Error as HttpbisError;
use std::io::Error as IoError;
use std::sync::mpsc::SendError;
use std::any::Any;

#[derive(Debug)]
enum Error {
    Grpc(GrpcError),
    Httpbis(HttpbisError),
    CommandLineRead(CommandLineReadError),
    Io(IoError),
    SendError(SendError<()>),
    ThreadError(Box<dyn Any + Send + 'static>),
}

fn main() -> Result<(), Error> {
    use std::{sync::{RwLock, Arc, mpsc}, io::{Read, stdin, stdout, Write}, thread, net::SocketAddr};
    use grpc::ServerBuilder;
    use implementation::{Node, routing_service, channel_service, payment_service};
    use self::Error::*;

    let argument = Argument::from_env().map_err(CommandLineRead)?;

    // tui
    let (handle, rx) = {
        write!(stdout(), "\
        the lightning peach node is running at: {}, database at: {}\n\
        enter any to shutdown... ", argument.address, argument.db_path).map_err(Io)?;
        stdout().flush().map_err(Io)?;

        let (tx, rx) = mpsc::channel();
        (
            thread::spawn(move || {
                let _ = stdin().read(&mut [0]).map_err(Io)?;
                tx.send(()).map_err(SendError)
            }),
            rx
        )
    };

    let node = {
        let secret = [
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
        ];

        Arc::new(RwLock::new(Node::new(secret)))
    };

    let server = {
        let mut server = ServerBuilder::new();
        if let Some(acceptor) = argument.tls_acceptor {
            server.http.set_tls(acceptor);
        }
        server.http.set_addr(argument.address).map_err(Httpbis)?;
        server.http.set_cpu_pool_threads(4);
        server.add_service(routing_service(node.clone()));
        server.add_service(channel_service());
        server.add_service(payment_service());
        server.build().map_err(Grpc)?
    };

    let address: SocketAddr = "127.0.0.1:10000".parse().unwrap();
    Node::listen(node, &address, rx);

    handle.join().map_err(ThreadError)??;

    let _ = server;
    Ok(())
}
