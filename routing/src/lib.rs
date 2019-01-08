#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

extern crate secp256k1;
extern crate chrono;
extern crate wire;
extern crate state;
extern crate dijkstras_search;
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

mod graph_state;
mod node;
mod channel;
mod tools;

pub use self::graph_state::{State, SharedState};
pub use rocksdb::Error as DBError;
