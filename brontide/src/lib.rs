#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

extern crate rand;
extern crate secp256k1;
extern crate sha2;
extern crate byteorder;
extern crate chacha20_poly1305_aead;
extern crate hkdf;
extern crate hex;
extern crate crossbeam;
extern crate serde;

extern crate tokio;
extern crate bytes;

extern crate wire;

mod machine;
pub use self::machine::{Machine, HandshakeError, BrontideStream};

//#[cfg(test)]
//mod test_tcp_communication;
