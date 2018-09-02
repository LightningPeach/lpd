#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

extern crate wire;
extern crate brontide;
extern crate lnd_rust;
extern crate grpc;
extern crate futures;

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

    use lnd_rust::rpc_grpc::Lightning;
    use lnd_rust::rpc;
    use grpc::RequestOptions;

    use futures::Future;

    #[test]
    fn run_btcd_lnd() {
        use std::thread;
        use std::time::Duration;

        let btcd = BtcDaemon::new("btcd").unwrap().run().unwrap();

        // creating two nodes with base port 10000
        let nodes = LnRunning::batch(2, 10000, btcd.as_ref());

        thread::sleep(Duration::from_secs(10));
        let client = nodes[0].client().unwrap();
        let peer_address = nodes[1].address().unwrap().wait().unwrap();
        let mut connect_peer_request = rpc::ConnectPeerRequest::new();
        connect_peer_request.set_addr(peer_address);
        let options = RequestOptions::new();
        // try to connect first peer to second
        let response = Lightning::connect_peer(&client, options, connect_peer_request);
        let v = response.0.wait().unwrap().1.wait().unwrap().0;
        println!("{:?}", v);
        thread::sleep(Duration::from_secs(100));
    }
}
