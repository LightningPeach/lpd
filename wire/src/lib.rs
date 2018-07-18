#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

#[cfg(test)]
extern crate rand;

mod messages;

mod serde_facade;

pub use self::messages::Message;
pub use self::messages::types::*;
pub use self::messages::channel::*;
pub use self::messages::setup::*;
pub use self::messages::control::*;

pub use self::serde_facade::BinarySD;
