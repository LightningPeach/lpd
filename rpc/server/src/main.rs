extern crate grpc;
extern crate tls_api_native_tls;
extern crate tls_api;
extern crate httpbis;

extern crate futures;

extern crate implementation;

mod config;
use self::config::{Argument, Error as CommandLineReadError};

use grpc::Error as GrpcError;
use httpbis::Error as HttpbisError;
use std::io::Error as IoError;
use futures::sync::mpsc::SendError;

use implementation::Command;
use std::net::SocketAddr;
use std::any::Any;

#[derive(Debug)]
enum Error {
    Grpc(GrpcError),
    Httpbis(HttpbisError),
    CommandLineRead(CommandLineReadError),
    Io(IoError),
    SendError(SendError<Command<SocketAddr>>),
    ThreadError(Box<dyn Any + Send + 'static>),
}

fn main() -> Result<(), Error> {
    use std::{sync::{RwLock, Arc}, io::{Read, stdin, stdout, Write}, thread};
    use grpc::ServerBuilder;
    use implementation::{Node, Command, routing_service, channel_service, payment_service};
    use futures::sync::mpsc;
    use futures::Future;
    use futures::Sink;
    use self::Error::*;

    let argument = Argument::from_env().map_err(CommandLineRead)?;

    let (handle, node, rx, tx) = {
        // tui
        let (handle, rx, tx) = {
            write!(stdout(), "\
                the lightning peach node is listening rpc at: {}, listening peers at: {}, has database at: {}\n\
                enter any to shutdown... \n", argument.address, argument.p2p_address, argument.db_path).map_err(Io)?;
            stdout().flush().map_err(Io)?;

            let (tx, rx) = mpsc::channel(1);
            let tx_wait = tx.clone();
            (
                thread::spawn(move || {
                    let _ = stdin().read(&mut [0]).map_err(Io)?;
                    tx_wait.send(Command::Terminate).wait().map_err(SendError)
                }),
                rx,
                tx,
            )
        };

        let secret = [
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
        ];

        (handle, Arc::new(RwLock::new(Node::new(secret, argument.db_path))), rx, tx)
    };

    let server = {
        let mut server = ServerBuilder::new();
        if let Some(acceptor) = argument.tls_acceptor {
            server.http.set_tls(acceptor);
        }
        server.http.set_addr(argument.address).map_err(Httpbis)?;
        server.http.set_cpu_pool_threads(4);
        server.add_service(routing_service(node.clone(), tx));
        server.add_service(channel_service());
        server.add_service(payment_service());
        server.build().map_err(Grpc)?
    };

    Node::listen(node, &argument.p2p_address, rx).map_err(Io)?;

    handle.join().map_err(ThreadError)??;

    let _ = server;
    Ok(())
}
