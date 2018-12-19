#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

extern crate secp256k1;
extern crate chrono;
extern crate wire;
extern crate binformat;
extern crate brontide;
extern crate bitcoin_types;
extern crate common_types;
extern crate rand;
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate shred;
extern crate rayon;

#[cfg(test)]
#[macro_use]
extern crate hex_literal;

#[cfg(any(feature = "rpc", test))]
extern crate hex;

extern crate tokio;

extern crate rocksdb;

extern crate serde;
extern crate serde_derive;

#[cfg(feature = "rpc")]
extern crate interface;

mod state;
mod node;
mod channel;
mod db;
mod tools;

pub use self::state::{State, TopologyMessage};
pub use rocksdb::Error as DBError;
