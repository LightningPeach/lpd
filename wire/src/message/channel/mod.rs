mod funding;
pub use self::funding::*;

mod close;
pub use self::close::*;

mod operation;
pub use self::operation::*;

mod open;
pub use self::open::*;

mod announcement;
pub use self::announcement::*;

mod update;
pub use self::update::*;

use super::types::*;
use ::UncompressedData;

bitflags! {
    #[derive(Serialize, Deserialize)]
    struct ChannelFlags: u8 {
        const FF_ANNOUNCE_CHANNEL = 0b00000001;
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct ChannelId {
    data: [u8; 32],
}

impl ChannelId {
    pub fn all() -> Self {
        ChannelId {
            data: [0; 32],
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct ShortChannelId {
    block_height: u32,
    tx_index: u32,
    tx_position: u16,
}

// TODO: custom serde, variant should be 1 byte
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub enum ShortChannelIdEncoding {
    StoredPlain(Vec<ShortChannelId>),
    StoredZlib(UncompressedData<ShortChannelId>),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct QueryShortChannelIds {
    chain_hash: Hash256,
    ids: ShortChannelIdEncoding,
}

#[cfg(test)]
mod rand {
    use super::ChannelId;

    use rand::distributions::Distribution;
    use rand::distributions::Standard;
    use rand::Rng;

    impl Distribution<ChannelId> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ChannelId {
            let mut rng = rng;
            let rnd_bytes: Vec<u8> = self.sample_iter(&mut rng).take(32).collect();
            let mut this = ChannelId { data: [0u8; 32], };
            this.data.copy_from_slice(rnd_bytes.as_slice());
            this
        }
    }
}
