#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

mod machine;
pub use self::machine::{Machine, HandshakeError, BrontideStream};

// brontide reexport the type in order to reduce dependencies
pub use binformat::WireError;

//#[cfg(test)]
//mod test_tcp_communication;
