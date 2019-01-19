#![forbid(unsafe_code)]

mod serde_facade;
mod compression_facade;

#[cfg(feature = "secp256k1")]
mod secp256k1_m;

pub use self::serde_facade::{BinarySD, WireError};
pub use self::compression_facade::{PackSized, SerdeVec, UncompressedData, SerdeRawVec};
