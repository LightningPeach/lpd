#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

mod home;
use self::home::Home;

mod chain;
pub use self::chain::*;

mod ln;
pub use self::ln::{LnDaemon, LnRunning};

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn cleanup(process: &str) {
    use std::process::Command;

    Command::new("killall").arg(process).output().map(|_| ()).unwrap_or(());
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn cleanup(name: &str) {
    panic!("cannot stop other instance of `{}`, stop it manually", name)
}

fn main() {
    use std::thread;
    use std::time::Duration;
    use futures::{Future, Stream};
    use bitcoin_rpc_client::BitcoinRpcApi;
    use bitcoin::Address;
    use std::str::FromStr;

    let btc_running = Bitcoind::new("b").unwrap().run().unwrap();
    thread::sleep(Duration::from_secs(5));

    // creating two nodes with base port 10000
    let nodes = LnRunning::batch(3, 9800, btc_running.as_ref());
    thread::sleep(Duration::from_secs(5));

    let mining_address = nodes[0].new_address().wait().unwrap();
    let mining_address = Address::from_str(mining_address.as_str()).unwrap();

    btc_running.rpc_client().generate_to_address(400, &mining_address).unwrap().unwrap();
    thread::sleep(Duration::from_secs(5));

    let _ = nodes[0].connect_peer(&nodes[1]).wait().unwrap();
    let update_stream = nodes[0].open_channel(&nodes[1]);
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
}
