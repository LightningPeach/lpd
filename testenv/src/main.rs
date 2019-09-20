//#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

use std::thread;
use std::time::Duration;
use futures::{Future, Stream};
use bitcoin_rpc_client::RpcApi as _;
use bitcoin::Address;
use std::str::FromStr;

mod home;
use self::home::Home;

mod chain;
pub use self::chain::*;

mod lnd;
pub use self::lnd::{LndConfig, LndProcess};

mod lpd;
pub use self::lpd::{LpServer, LpRunning};

mod error;

// abstract lightning node
mod abstract_lightning_node;
// use self::al::AbstractLightningNode;

use error::Error;
use std::process::Command;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn cleanup(process: &str) {
    use std::process::Command;

    Command::new("killall").arg(process).output().map(|_| ()).unwrap_or(());
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn cleanup(name: &str) {
    panic!("cannot stop other instance of `{}`, stop it manually", name)
}

fn _main() -> Result<(), Error> {
    // TODO(mkl): add possibility to connect to already existing testenv
    // TODO(mkl): add posibility to leave nodes working after tests ends
    println!("Starting testnev");

    let bitcoind_process = BitcoindConfig::new("b")
        .map(|x| {
            println!("{:#?}", x.home.path());
            x
        })
        .map_err(|err| {
            new_error!(err, "cannot create instance bitcoind")
        })?
        .run()
        .map_err(|err|{
            new_error!(err, "cannot run bitcoind")
        })?;
    println!("After starting bitcoind:");

    // Start electrumx
    Command::new("electrs")
        .args(&[
            format!("-vvv"),
            format!("--network=regtest"),
            format!("--jsonrpc-import"),
            format!("--daemon-dir={}", bitcoind_process.config.data_dir_path().to_string_lossy()),
            format!("--db-dir={}", {
                let mut p = Home::sandbox().to_owned();
                p.push("electrs");
                p.to_string_lossy().to_string()
            }),
            format!("--cookie={}:{}", bitcoind_process.config.rpc_user, bitcoind_process.config.rpc_pass),
            format!("--daemon-rpc-addr=127.0.0.1:{}", bitcoind_process.config.rpc_port)
        ])
        .spawn()
        .map_err(|err| {
            new_io_error!(err, "cannot launch electrs")
        })?;

    // TODO(mkl): add checking state instead of flat wait
    println!("Start waiting for bitcoind");
    thread::sleep(Duration::from_secs(2));
    println!("End waiting for bitcoind");

    // Generate some blocks to activate segwit
    bitcoind_process
        .rpc_client()?
        .generate(500, None)
        .map_err(|err| {
            new_bitcoin_rpc_error!(err, "error, cannot mine initial blocks")
        })?;

    // creating two nodes with base port 9800
    let nodes = LndProcess::batch(2, 9800, bitcoind_process.as_ref());
    nodes[0].wait_for_sync(10)
        .map_err(|err|{
            new_error!(err, "fail waiting node 0")
        })?;
    nodes[1].wait_for_sync(10)
        .map_err(|err|{
            new_error!(err, "fail waiting node 0")
        })?;

    println!("nodes.len={}", nodes.len());

    // Mine some blocks
    bitcoind_process
        .rpc_client()?
        .generate(10, None)
        .map_err(|err| {
            new_bitcoin_rpc_error!(err, "error, cannot mine initial blocks")
        })?;

    nodes[0].obtain_info().wait()
        .map(|rez|{
            println!("getinfo: {:?}", rez);
            rez
        })
        .map_err(|err| {
            new_grpc_error!(err, "cannot `getinfo` from node 0")
        })?;

    let mining_address = nodes[0].new_address().wait()
        .map_err(|err|{
            new_grpc_error!(err, "error getting new address")
        })?;
    let mining_address = Address::from_str(mining_address.as_str())
        .map_err(|err| {
            new_bitcoin_encode_error!(err, "error converting address")
        })?;

    bitcoind_process.rpc_client()?
        .generate_to_address(110, &mining_address)
        .map_err(|err| {
            new_bitcoin_rpc_error!(err, "error mining initial money for the first node")
        })?;
    println!("Before waiting for mining blocks for money for first node");
    thread::sleep(Duration::from_secs(5));
    println!("After waiting for mining blocks for money for first node");

    println!("Before starting peach");
    let peach_node = LpServer::new(9735, 10009, "peach")
        .map_err(|err|{
            println!("cannot create LpServer: {:?}", err);
            err
        })?
        .run(bitcoind_process.as_ref())
        .map_err(|err| {
            println!("cannot run LpServer: {:?}", err);
            err
        })?;
    println!("After starting peach");
    peach_node
        .wait_for_sync(10)
        .map_err(|err| {
            new_error!(err, "fail to wait for lpd to sync")
        })?;
    let peach_node_info =  peach_node.obtain_info().wait().unwrap();
    println!("peach_node info: {:?}", &peach_node_info);

    //let _ = nodes[0].connect_peer(&nodes[1]).wait()?;
    //let pk1 = nodes[1].address().pubkey;
    let _ = nodes[0].connect_peer(&peach_node).wait().map_err(|err| {
        new_grpc_error!(err, "error connecting to peach node")
    })?;
    println!("After connecting to peach node");


    let pk_our = peach_node_info.identity_pubkey;
    let update_stream = nodes[0].open_channel(pk_our.as_str());
    thread::sleep(Duration::from_secs(5));
    bitcoind_process.rpc_client().unwrap().generate_to_address(10, &mining_address).unwrap();
    /*

    // TODO: run it
    let _ = update_stream.map(|i| {
        println!("{:?}", i);
    });*/

    fn pause() {
        use std::io;
        use std::io::prelude::*;

        println!("Enter any string to continue...");
        let _ = io::stdin().read(&mut [0u8]).unwrap();
    }

    pause();

    // keep it running until this line
    let _ = nodes;
    let _ = bitcoind_process;
    Ok(())
}

fn main() {
    match _main() {
        Ok(()) => {
            println!("Success")
        },
        Err(err) => {
            println!("ERROR: {:?}", err);
            thread::sleep(Duration::from_secs(10000));
        }
    }
}