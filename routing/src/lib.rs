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
        let tcp_self = TcpSelf::new();
        let key = public_key!(2, "8faeeb36e7d82134b5bfedcaeb9b81e7be4260ddde1bcc97d5db7d2bd364f471");
        let mut tcp_peer = tcp_self.connect_peer(key, Address::localhost(10011)).unwrap();

        let init = Init::new(
            RawFeatureVector::new(),
            RawFeatureVector::new().set_bit(FeatureBit::GossipQueriesOptional).set_bit(FeatureBit::InitialRoutingSync),
        );
        let response = tcp_peer.synchronous_message(Message::Init(init)).unwrap();
        println!("{:?}", response);
        let s = Synchronization {};
        s.sync_channels(&mut tcp_peer)
    }
}
