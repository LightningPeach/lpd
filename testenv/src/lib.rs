#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

extern crate wire;
extern crate brontide;
extern crate lnd_rust;
extern crate grpc;
extern crate futures;
extern crate lazycell;

mod home;
use self::home::Home;

mod btc;
pub use self::btc::BtcDaemon;

mod ln;
pub use self::ln::LnDaemon;
pub use self::ln::LnRunning;

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
        use lnd_rust::rpc;

        let btcd = BtcDaemon::new("btcd").unwrap().run().unwrap();

        // creating two nodes with base port 10000
        let nodes = LnRunning::batch(2, 10000, btcd.as_ref());

        thread::sleep(Duration::from_secs(10));
        let _ = nodes[0].connect_peer(&nodes[1]).wait().unwrap();
        let update_stream = nodes[0].open_channel(&nodes[1]);
        let _ = update_stream.map(|i| {
            println!("{:?}", i);
        });
    }
}
