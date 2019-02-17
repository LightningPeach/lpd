#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

mod crypto;
mod hop;
mod packet;
mod route;

#[cfg(test)]
mod tests;

pub use self::route::{OnionPacketVersion, OnionRoute};
pub use self::packet::{OnionPacket, ValidOnionPacket, Processed, OnionPacketProcessingError};
pub use self::hop::{Hop, HopData, BitcoinHopData};
