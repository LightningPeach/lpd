#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

// The crate is bunch of public modules and tests module.
// Its structure will change.

extern crate secp256k1;
extern crate wire;
extern crate brontide;
extern crate bitcoin_types;
extern crate common_types;

#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate hex;

pub mod node;
pub mod discovery;
pub mod topology;

#[cfg(test)]
mod tests {
    use brontide::MachineRead;
    use brontide::MachineWrite;
    use brontide::MessageConsumer;
    use brontide::MessageSource;
    use brontide::tcp_communication::Stream;

    use wire::Message;
    use wire::Init;
    use wire::RawFeatureVector;
    use wire::FeatureBit;

    #[test]
    fn it_works() {
        let mut stream = Stream::from_pair(
            "127.0.0.1:10011",
            "03638055e425d51f33c76780919e574d7e987772feb5d12cf57bb4fb4798c06b70"
        );

        let init = {
            use self::FeatureBit::*;

            let global_features = RawFeatureVector::new();
            let local_features = RawFeatureVector::new().set_bit(InitialRoutingSync);
            Init::new(global_features, local_features)
        };

        stream.as_write().send(Message::Init(init)).unwrap();
        let init = stream.as_read().receive().unwrap().as_init().unwrap();
        println!("{:?}", init);
    }
}
