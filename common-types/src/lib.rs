#![forbid(unsafe_code)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rand;
extern crate sha2;
#[macro_use]
extern crate hex_literal;
extern crate hex;

mod hash;
pub use self::hash::Hash256;
