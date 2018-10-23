#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

extern crate wire;
extern crate brontide;
extern crate lnd_rust;
extern crate grpc;
extern crate futures;
extern crate lazycell;
extern crate hex;

mod home;
use self::home::Home;

mod btc;
pub use self::btc::BtcDaemon;

mod ln;
pub use self::ln::LnDaemon;
pub use self::ln::LnRunning;

#[cfg(target_os = "linux")]
pub fn cleanup(name: &str) {
    Command::new("killall").arg(name).output().unwrap();
}

#[cfg(not(target_os = "linux"))]
pub fn cleanup(name: &str) {
    panic!("cannot stop other instance of `{}`, stop it manually", name)
}

#[cfg(test)]
mod tests {
    use super::BtcDaemon;
    use super::LnRunning;

    use futures::Future;
    use futures::Stream;

    #[test]
    fn run_btcd_lnd() {
        use std::thread;
        use std::time::Duration;

        let btc_running = BtcDaemon::new("btcd").unwrap().run().unwrap();

        // creating two nodes with base port 10000
        let btcd_pubkey_path = btc_running.as_ref().public_key_path();
        let nodes = LnRunning::batch(2, 10000, btcd_pubkey_path.to_str().unwrap());
        thread::sleep(Duration::from_secs(10));

        let mining_address = nodes[0].new_address().wait().unwrap();
        let mut btc_running = btc_running.rerun_with_mining_address(mining_address).unwrap();
        thread::sleep(Duration::from_secs(10));

        btc_running.generate(400).unwrap();
        thread::sleep(Duration::from_secs(5));

        let _ = nodes[0].connect_peer(&nodes[1]).wait().unwrap();
        let update_stream = nodes[0].open_channel(&nodes[1]);
        thread::sleep(Duration::from_secs(5));
        btc_running.generate(10).unwrap();

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
}
