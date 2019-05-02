use std::error::Error;

use super::types::*;

mod funding;
pub use self::funding::*;

mod close;
pub use self::close::*;

mod operation;
pub use self::operation::*;

mod open;
pub use self::open::*;

mod keys;
pub use self::keys::*;

mod announcement;
pub use self::announcement::*;

mod update;
pub use self::update::*;

mod query_short_channel_ids;
pub use self::query_short_channel_ids::*;

mod query_channel_range;
pub use self::query_channel_range::*;

use binformat::PackSized;

use bitflags::bitflags;

use serde_derive::{Serialize, Deserialize};
//use nom::AsBytes;

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct ChannelFlags: u8 {
        /// Indicates whether the initiator of the funding flow wishes to advertise
        /// this channel publicly to the network, as detailed within BOLT #7.
        const FF_ANNOUNCE_CHANNEL = 0b00000001;
    }
}

/// The unique identifier of the channel. It's derived from the funding transaction
/// by combining the funding_txid and the funding_output_index, using big-endian exclusive-OR
/// (i.e. funding_output_index alters the last 2 bytes).
#[derive(Default, Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub struct ChannelId {
    pub data: [u8; 32],
}

impl ChannelId {
    // TODO(mkl): maybe rename
    // TODO(mkl): maybe add to_bytes/from_bytes methods to conversion from/to u32
    pub fn all() -> Self {
        ChannelId {
            data: [0; 32],
        }
    }

    pub fn from_hex(s: &str) -> Result<Self, Box<Error>> {
        let bytes = hex::decode(s.as_bytes())
            .map_err(|err| format!("cannot decode hex: {:?}", err))?;
        if bytes.len() != 32 {
            return Err(format!("incorrect byte length of ChannelId, got {}, want {}", bytes.len(), 32).into());
        }
        let mut data = [0; 32];
        data.copy_from_slice(&bytes);
        Ok(ChannelId{
            data
        })
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.data[..])
    }
}

impl std::fmt::Debug for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ChannelId({})", hex::encode(&self.data[..]))
    }
}

impl From<[u8; 32]> for ChannelId {
    fn from(x: [u8; 32]) -> Self {
        ChannelId {
            data: x,
        }
    }
}

/// The unique description of the funding transaction
// TODO(mkl): implement conversion from/to human-readable format, like 0:0:0
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ShortChannelId {
    pub block_height: u32,
    /// the transaction index within the block
    pub tx_index: u32,
    /// the output index that pays to the channel
    pub tx_position: u16,
}

impl PackSized for ShortChannelId {
    const SIZE: usize = 8;
}

impl ShortChannelId {
    pub fn from_u64(x: u64) -> Self {
        x.into()
    }

    pub fn to_u64(&self) -> u64 {
        self.clone().into()
    }
}

impl From<u64> for ShortChannelId {
    fn from(v: u64) -> Self {
        // WARNING: big endian
        type Mask = (u64, u64);
        let mask = |size: u64, p: u64| (((1 << size) - 1) << p, p);
        let apply = |value: u64, (m, p): Mask| (value & m) >> p;

        ShortChannelId {
            block_height: apply(v, mask(24, 40)) as _,
            tx_index: apply(v, mask(24, 16)) as _,
            tx_position: apply(v, mask(16, 0)) as _,
        }
    }
}

impl From<ShortChannelId> for u64 {
    fn from(s: ShortChannelId) -> Self {
        // WARNING: big endian
        type Mask = (u64, u64);
        let mask = |size: u64, p: u64| ((1 << size) - 1, p);
        let apply = |value: u64, (m, p): Mask| (value & m) << p;

        0
        | apply(s.block_height as _, mask(24, 40))
        | apply(s.tx_index as _, mask(24, 16))
        | apply(s.tx_position as _, mask(16, 0))
    }
}

mod serde_m {
    use super::ShortChannelId;

    use serde::Serialize;
    use serde::Serializer;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::de::Visitor;
    use serde::de::Error;
    use std::fmt;

    impl Serialize for ShortChannelId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
            serializer.serialize_u64(self.clone().into())
        }
    }

    impl<'de> Deserialize<'de> for ShortChannelId {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            struct V;

            impl<'de> Visitor<'de> for V {
                type Value = ShortChannelId;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "8 bytes unsigned integer")
                }

                fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: Error, {
                    Ok(ShortChannelId::from(v))
                }
            }

            deserializer.deserialize_u64(V)
        }
    }
}

#[cfg(test)]
mod rand_m {
    use super::ChannelId;
    use super::ShortChannelId;

    use rand::{Rng, Rand};

    impl Rand for ChannelId {
        fn rand<R: Rng>(rng: &mut R) -> Self {
            let rnd_bytes: Vec<u8> = rng.gen_iter().take(32).collect();
            let mut this = ChannelId { data: [0u8; 32], };
            this.data.copy_from_slice(rnd_bytes.as_slice());
            this
        }
    }

    impl Rand for ShortChannelId {
        fn rand<R: Rng>(rng: &mut R) -> Self {
            From::<u64>::from(rng.gen())
        }
    }
}

#[cfg(test)]
mod tests {
    use binformat::BinarySD;
    use super::ShortChannelId;
    use rand;

    #[test]
    fn short_channel_id_packing() {
        let value: u64 = rand::random();
        let short_channel_id = ShortChannelId::from(value);
        let restored = short_channel_id.into();
        assert_eq!(value, restored);
    }

    #[test]
    fn some_test() {
        let v = vec![0u8, 1, 145, 0, 0, 1, 0, 0, ];
        let t: ShortChannelId = BinarySD::deserialize(&v[..]).unwrap();
        println!("{:?}", t);
    }
}
