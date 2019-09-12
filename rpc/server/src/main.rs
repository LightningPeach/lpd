mod config;
use self::config::{Config, Error as CommandLineReadError, create_tls_acceptor};

mod wallet;

use dependencies::httpbis;
use dependencies::futures;
use dependencies::ctrlc;
use dependencies::secp256k1;

use structopt::StructOpt;

use grpc::Error as GrpcError;
use httpbis::Error as HttpbisError;
use std::io::Error as IoError;
use implementation::wallet_lib::error::WalletError;
use std::path::PathBuf;

use build_info::get_build_info;

#[derive(Debug)]
enum Error {
    Grpc(GrpcError),
    Httpbis(HttpbisError),
    CommandLineRead(CommandLineReadError),
    Io(IoError),
    WalletError(WalletError, String),
    SendError(ctrlc::Error),
    TransportError(connection::TransportError),
    FileNotSpecified {
        description: String,
    }
}

fn print_version() {
    println!("{:#?}", get_build_info!());
}

fn main() -> Result<(), Error> {
    use std::{sync::{Mutex, RwLock, Arc}, path::PathBuf};
    use grpc::ServerBuilder;
    use implementation::{Node, Command, routing_service, channel_service, payment_service, wallet_service};
    use futures::{sync::mpsc, Future, Sink};
    use self::Error::*;
    use self::wallet::create_wallet;

    let config: Config = Config::from_args();

    if config.print_config {
        println!("{:#?}", config);
        return Ok(());
    }

    if config.print_version {
        print_version();
        return Ok(());
    }

    let wallet = {
        let mut wallet_db_path = PathBuf::from(config.db_path.clone());
        wallet_db_path.push("wallet");
        let wallet = create_wallet(&wallet_db_path.as_path()).map_err(|err| {
            WalletError(err, "cannot create bitcoin onchain wallet".to_owned())
        })?;
        Arc::new(Mutex::new(wallet))
    };

    let (node, tx, rx) = {
        let (tx, rx) = mpsc::channel(1);

        let tx_wait = tx.clone();
        ctrlc::set_handler(move || {
            println!("received termination command");
            tx_wait.clone().send(Command::Terminate).wait().unwrap();
            println!("the command is propagated, terminating...");
        }).map_err(SendError)?;

        // TODO: generate and store it somehow
        let secret = [
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
        ];

        let ctx = secp256k1::Secp256k1::new();
        let priv_key = secp256k1::SecretKey::from_slice(&secret).unwrap();
        let pub_key = secp256k1::PublicKey::from_secret_key(&ctx, &priv_key);

        println!("RPC listen at: {}", config.rpc_address);
        println!("peer listen at: {}", config.p2p_address);
        println!("db_path: {:?}", config.db_path);
        println!("Identity pub_key: {}", &pub_key);
        println!("URI: {}@{}", &pub_key, config.p2p_address);

        let mut node_db_path = PathBuf::from(config.db_path.clone());
        node_db_path.push("node");

        (Arc::new(RwLock::new(Node::new(wallet.clone(), secret, node_db_path))), tx, rx)
    };

    let server = {
        let mut server = ServerBuilder::new();
        if !config.rpc_no_tls {
            let cert_path = config.rpc_tls_cert_path.ok_or ({
                Error::FileNotSpecified {
                    description: "RPC TLS certificate file".to_owned(),
                }
            })?;
            let key_path = config.rpc_tls_key_path.ok_or ({
                Error::FileNotSpecified {
                    description: "RPC TLS key file".to_owned(),
                }
            })?;
            let acceptor = create_tls_acceptor(&cert_path, &key_path)
                .map_err(Error::CommandLineRead)?;
            server.http.set_tls(acceptor);
        }
        server.http.set_addr(config.rpc_address).map_err(Httpbis)?;
        // TODO(mkl): make it configurable
        server.http.set_cpu_pool_threads(4);
        server.add_service(wallet_service(wallet.clone(), tx.clone()));
        server.add_service(routing_service(node.clone(), tx.clone()));
        server.add_service(channel_service(node.clone(), tx.clone()));
        server.add_service(payment_service());
        server.build().map_err(Grpc)?
    };

    Node::listen(node, &config.p2p_address, rx)
        .map_err(|err| {
            Error::TransportError(err)
        })?;

    // TODO: handle double ctrl+c
    //panic!("received termination command during processing termination command, terminate immediately");

    let _ = server;
    Ok(())
}
