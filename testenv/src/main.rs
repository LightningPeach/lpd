//#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

mod home;
use self::home::Home;

mod chain;
pub use self::chain::*;

mod ln;
pub use self::ln::{LnDaemon, LnRunning};

mod lp;
pub use self::lp::{LpServer, LpRunning};

// abstract lightning node
mod al;
use self::al::AbstractLightningNode;

use std::error::Error;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn cleanup(process: &str) {
    use std::process::Command;

    Command::new("killall").arg(process).output().map(|_| ()).unwrap_or(());
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn cleanup(name: &str) {
    panic!("cannot stop other instance of `{}`, stop it manually", name)
}

fn main() -> Result<(), Box<dyn Error>> {
    // TODO(mkl): add possibility to connect to already existing testenv
    // TODO(mkl): add posibility to leave nodes working after tests ends
    println!("Starting testnev");
    use std::thread;
    use std::time::Duration;
    use futures::{Future, Stream};
    use bitcoin_rpc_client::BitcoinRpcApi;
    use bitcoin::Address;
    use std::str::FromStr;

    let btc_running = Bitcoind::new("b")?.run()?;
    println!("After starting bitcoind");

    // TODO(mkl): add checking state instead of flat wait
    println!("Start waiting for bitcoind");
    thread::sleep(Duration::from_secs(2));
    println!("End waiting for bitcoind");

    // Generate some blocks to activate segwit
    btc_running.rpc_client().generate(500).unwrap_or_else(|err|{
        println!("cannot mine initial blocks: {:?}", err);
        panic!(err);
    });

    // creating two nodes with base port 9800
    let nodes = LnRunning::batch(2, 9800, btc_running.as_ref());
    // TODO(mkl): add checking state instead of flat wait
    thread::sleep(Duration::from_secs(15));
    println!("nodes.len={}", nodes.len());

    nodes[0].obtain_info().wait()
        .map(|rez|{
            println!("getinfo: {:?}", rez);
            rez
        })
        .map_err(|err| {
            println!("ERROR getting getinfo: {:?}", err);
            err
        })?;

    println!("Before starting peach");
    let peach_node = LpServer::new(9735, 10009, "peach")
        .map_err(|err|{
            println!("cannot create LpServer: {:?}", err);
            err
        })?
        .run(btc_running.as_ref())
        .map_err(|err| {
            println!("cannot run LpServer: {:?}", err);
            err
        })?;
    println!("After starting peach");
    thread::sleep(Duration::from_secs(2));
    println!("peach_node info: {:?}", peach_node.info());



    let mining_address = nodes[0].new_address().wait()
        .map_err(|err|{
            println!("error getting new address: {:?}", err);
            err
        })?;
    let mining_address = Address::from_str(mining_address.as_str())
        .map_err(|err| {
            println!("error converting address {:?}", err);
            err
        })?;

    btc_running.rpc_client()
        .generate_to_address(400, &mining_address)
        .unwrap_or_else(|err|{
            println!("rpc error mining initial money for the first node: {:?}", err);
            panic!(err);
        })
        .unwrap_or_else(|err|{
            println!("client error mining money for the first node: {:?}", err);
            panic!(err);
        });
    println!("Before waiting for mining blocks for money for first node");
    thread::sleep(Duration::from_secs(5));
    println!("After waiting for mining blocks for money for first node");

    //let _ = nodes[0].connect_peer(&nodes[1]).wait()?;
    //let pk1 = nodes[1].address().pubkey;
    let _ = nodes[0].connect_peer(&peach_node).wait().map_err(|err| {
        println!("error connecting to peach node: {:?}", err);
        err
    })?;
    println!("After connecting to peach node");

    let pk_our = peach_node.address().pubkey;
    let update_stream = nodes[0].open_channel(pk_our.as_str());
    thread::sleep(Duration::from_secs(5));
    btc_running.rpc_client().generate_to_address(10, &mining_address).unwrap().unwrap();

    // TODO: run it
    let _ = update_stream.map(|i| {
        println!("{:?}", i);
    });

    fn pause() {
        use std::io;
        use std::io::prelude::*;

        println!("Enter any string to continue...");
        let _ = io::stdin().read(&mut [0u8]).unwrap();
    }

    pause();

    // keep it running until this line
    let _ = nodes;
    let _ = btc_running;
    Ok(())
}
