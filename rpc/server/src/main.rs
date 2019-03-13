extern crate grpc;
extern crate tls_api_native_tls;
extern crate tls_api;
extern crate httpbis;
extern crate ctrlc;

extern crate futures;

extern crate implementation;

mod config;
use self::config::{Argument, Error as CommandLineReadError};

use grpc::Error as GrpcError;
use httpbis::Error as HttpbisError;
use std::io::Error as IoError;
use implementation::wallet_lib::error::WalletError;

#[derive(Debug)]
enum Error {
    Grpc(GrpcError),
    Httpbis(HttpbisError),
    CommandLineRead(CommandLineReadError),
    Io(IoError),
    SendError(ctrlc::Error),
    WalletError(WalletError),
}

fn main() -> Result<(), Error> {
    use std::sync::{Mutex, RwLock, Arc};
    use grpc::ServerBuilder;
    use implementation::{Node, Command, routing_service, channel_service, payment_service, wallet_service};
    use implementation::wallet_lib::{
        electrumx::ElectrumxWallet, default::WalletWithTrustedFullNode,
        walletlibrary::{WalletConfig, DEFAULT_NETWORK, DEFAULT_SALT, DEFAULT_PASSPHRASE, WalletLibraryMode, KeyGenConfig},
        interface::Wallet,
    };
    use futures::{sync::mpsc, Future, Sink};
    use self::Error::*;

    let argument = Argument::from_env().map_err(CommandLineRead)?;

    let (node, tx, rx) = {
        println!("the lightning peach node is listening rpc at: {}, listening peers at: {}, has database at: {}",
                 argument.address, argument.p2p_address, argument.db_path);

        let (tx, rx) = mpsc::channel(1);

        let tx_wait = tx.clone();
        ctrlc::set_handler(move || {
            println!("received termination command");
            tx_wait.clone().send(Command::Terminate).wait().unwrap();
            println!("the command is propagated, terminating...");
        }).map_err(SendError)?;

        let secret = [
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
        ];

        (Arc::new(RwLock::new(Node::new(secret, argument.db_path.clone()))), tx, rx)
    };

    let config = WalletConfig::new(
        DEFAULT_NETWORK,
        DEFAULT_PASSPHRASE.to_owned(),
        DEFAULT_SALT.to_owned(),
        format!("{}/wallet_db", argument.db_path.clone()),
    );
    let (wallet, mnemonic) = ElectrumxWallet::new(config, WalletLibraryMode::Create(KeyGenConfig::default()))
        .map_err(WalletError)?;
    println!("{}", mnemonic.to_string());
    let wallet: Arc<Mutex<Box<dyn Wallet + Send>>> = Arc::new(Mutex::new(Box::new(wallet)));

    let server = {
        let mut server = ServerBuilder::new();
        if let Some(acceptor) = argument.tls_acceptor {
            server.http.set_tls(acceptor);
        }
        server.http.set_addr(argument.address).map_err(Httpbis)?;
        server.http.set_cpu_pool_threads(4);
        server.add_service(wallet_service(wallet.clone(), tx.clone()));
        server.add_service(routing_service(node.clone(), tx.clone()));
        server.add_service(channel_service());
        server.add_service(payment_service());
        server.build().map_err(Grpc)?
    };

    Node::listen(node, wallet.clone(), &argument.p2p_address, rx).map_err(Io)?;
    println!("done");

    // TODO: handle double ctrl+c
    //panic!("received termination command during processing termination command, terminate immediately");

    let _ = server;
    Ok(())
}
