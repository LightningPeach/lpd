#![forbid(unsafe_code)]

extern crate secp256k1;

extern crate wire;
extern crate common_types;
extern crate hmac;
extern crate chacha;
extern crate sha2;
extern crate serde;
extern crate serde_derive;

mod hop;
mod route;

pub use self::route::{OnionPacketVersion, OnionRoute, OnionPacket};
pub use self::hop::{Hop, HopData, HopDataRealm};

use serde_derive::{Serialize, Deserialize};

/// `HMAC_SIZE` is the length of the HMACs used to verify the integrity of
/// the onion. Any value lower than 32 will truncate the HMAC both
/// during onion creation as well as during the verification.
pub const HMAC_SIZE: usize = 32;

/// `NUM_MAX_HOPS` is the the maximum path length. This should be set to an
/// estimate of the upper limit of the diameter of the node graph.
pub const NUM_MAX_HOPS: usize = 20;
