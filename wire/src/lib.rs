#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate secp256k1;
#[macro_use]
extern crate bitflags;
extern crate rand;
extern crate sha2;
extern crate flate2;
#[macro_use]
extern crate hex_literal;

#[cfg(test)]
extern crate hex;

mod message;

mod serde_facade;
mod compression_facade;

pub use self::message::*;
pub use self::message::types::*;

pub use self::serde_facade::BinarySD;
pub use self::compression_facade::UncompressedData;
pub use self::compression_facade::SerdeVec;
