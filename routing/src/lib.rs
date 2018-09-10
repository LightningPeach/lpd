#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

// The crate is bunch of public modules and tests module.
// Its structure will change.

extern crate secp256k1;
extern crate chrono;
#[macro_use]
extern crate wire;
extern crate brontide;
extern crate bitcoin_types;
extern crate common_types;
extern crate rand;

#[cfg(test)]
#[macro_use]
extern crate hex_literal;

#[cfg(test)]
extern crate hex;

pub mod discovery;
pub mod topology;
pub mod synchronization;
pub mod peer;

#[cfg(test)]
mod tests {
    use super::peer::*;
    use super::synchronization::*;
    use wire::Message;
    use wire::Init;
    use wire::RawFeatureVector;
    use wire::FeatureBit;
    use wire::PublicKey;
    use wire::Address;

    #[test]
    fn test_channel_range() {
        use std::thread;

        let tcp_self = TcpSelf::new();
        let key = public_key!(3, "f0826b27005e139f158ed899abe1c58929db2764b49c8780998ff78d442c6726");
        let mut tcp_peer = tcp_self.connect_peer(key, Address::localhost(10000)).unwrap();

        let init = Init::new(
            RawFeatureVector::new(),
            RawFeatureVector::new().set_bit(FeatureBit::InitialRoutingSync),
        );
        tcp_peer.send(Message::Init(init)).unwrap();
        let response = tcp_peer.receive().unwrap();
        println!("{:?}", response);

        let s = Synchronization {};
        s.sync_channels(&mut tcp_peer);

        thread::spawn(move || loop {
            println!("{:?}", tcp_peer.receive())
        }).join().unwrap();

        fn pause() {
            use std::io;
            use std::io::prelude::*;

            println!("Enter any string to continue...");
            let _ = io::stdin().read(&mut [0u8]).unwrap();
        }

        pause();
    }
}
