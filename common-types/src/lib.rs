#![forbid(unsafe_code)]

pub extern crate dependencies;

mod hash;
pub use self::hash::{Sha256, Sha256HashEngine};

mod crypto_types;
pub use self::crypto_types::{RawPublicKey, RawSignature};

// abstract cryptography
pub mod ac;

// implementation ac for secp256k1
pub mod secp256k1_m;
