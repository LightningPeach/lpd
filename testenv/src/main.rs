#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

extern crate wire;
extern crate brontide;
extern crate lnd_rust;
extern crate grpc;
extern crate futures;
extern crate lazycell;
extern crate hex;

use std::process::Command;
use futures::Future;
use futures::Stream;

mod home;
use self::home::Home;

mod chain;
pub use self::chain::*;

mod ln;
pub use self::ln::LnDaemon;
pub use self::ln::LnRunning;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn cleanup(process: &str) {
    Command::new("killall").arg(process).output().map(|_| ()).unwrap_or(());
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn cleanup(name: &str) {
    panic!("cannot stop other instance of `{}`, stop it manually", name)
}

fn main() {
    use std::thread;
    use std::time::Duration;

    let btc_running = Btcd::new("b").unwrap().run().unwrap();
    thread::sleep(Duration::from_secs(5));

    // creating two nodes with base port 10000
    let nodes = LnRunning::batch(3, 10000, btc_running.as_ref());
    thread::sleep(Duration::from_secs(5));

    let mining_address = nodes[0].new_address().wait().unwrap();
    let mut btc_running = btc_running.set_mining_address(mining_address).unwrap();
    thread::sleep(Duration::from_secs(5));

    btc_running.generate(400).unwrap();
    thread::sleep(Duration::from_secs(5));

    let _ = nodes[0].connect_peer(&nodes[1]).wait().unwrap();
    let update_stream = nodes[0].open_channel(&nodes[1]);
    thread::sleep(Duration::from_secs(5));
    btc_running.generate(10).unwrap();

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
