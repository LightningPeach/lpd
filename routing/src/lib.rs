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
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate shred;
#[macro_use]
extern crate shred_derive;
extern crate rayon;

#[cfg(test)]
#[macro_use]
extern crate hex_literal;

#[cfg(test)]
extern crate hex;

//pub mod discovery;
//pub mod topology;
//pub mod synchronization;
//pub mod peer;
pub mod graph;

use wire::Message;
use wire::AnnouncementNode;
use wire::AnnouncementChannel;
use wire::UpdateChannel;

pub struct Graph {
    nodes: Vec<AnnouncementNode>,
    channels: Vec<AnnouncementChannel>,
    policies: Vec<UpdateChannel>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: Vec::new(),
            channels: Vec::new(),
            policies: Vec::new(),
        }
    }

    pub fn message(&mut self, message: Message) {
        use self::Message::*;
        match message {
            AnnouncementNode(announcement_node) => {
                self.nodes.push(announcement_node);
            },
            AnnouncementChannel(announcement_channel) => {
                self.channels.push(announcement_channel);
            },
            UpdateChannel(update_channel) => {
                self.policies.push(update_channel);
            },
            _ => (),
        }
    }
}

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

        let mut tcp_self = TcpSelf::new();
        let key = public_key!(3, "02f2dda2232393a0a608f4ffa4c940b25b2ea4af8f9489f8e62afc0a729e973b0d");
        let mut tcp_peer = tcp_self.connect_peer(key, Address::localhost(10000)).unwrap();

        let init = Init::new(
            RawFeatureVector::new(),
            RawFeatureVector::new().set_bit(FeatureBit::InitialRoutingSync),
        );
        tcp_peer.send(Message::Init(init)).unwrap();
        let response = tcp_peer.receive().unwrap();
        println!("{:?}", response);

        //let s = Synchronization {};
        //s.sync_channels(&mut tcp_peer);

        thread::spawn(move || loop {
            println!("{:?}", tcp_peer.receive().unwrap())
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
