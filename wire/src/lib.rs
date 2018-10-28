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
extern crate digest;
extern crate flate2;
extern crate common_types;
extern crate bitcoin_types;

extern crate hex;
extern crate tokio;

mod message;

mod serde_facade;
mod compression_facade;
mod message_processor;

pub use self::message::*;
pub use self::message::types::*;

pub use self::serde_facade::BinarySD;
pub use self::serde_facade::WireError;
pub use self::compression_facade::UncompressedData;
pub use self::compression_facade::SerdeVec;
pub use self::compression_facade::PackSized;
pub use self::message_processor::*;
