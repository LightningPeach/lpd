extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

mod messages;

mod serde_facade;

pub use self::messages::Message;
pub use self::messages::types::RawFeatureVector;
pub use self::messages::types::FeatureBit;
pub use self::messages::types::FeatureVector;

pub use self::serde_facade::BinarySD;
