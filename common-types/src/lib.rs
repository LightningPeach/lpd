#![forbid(unsafe_code)]

mod hash;
pub use self::hash::Hash256;

// abstract cryptography
pub mod ac;

// implementation ac for secp256k1
pub mod secp256k1_m;
