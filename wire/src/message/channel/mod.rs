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

mod query_short_channel_ids;
pub use self::query_short_channel_ids::*;

mod query_channel_range;
pub use self::query_channel_range::*;

use super::types::*;

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct ChannelFlags: u8 {
        const FF_ANNOUNCE_CHANNEL = 0b00000001;
    }
}

#[derive(Default, Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
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

impl From<[u8; 32]> for ChannelId {
    fn from(x: [u8; 32]) -> Self {
        ChannelId{
            data: x,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ShortChannelId {
    block_height: u32,
    tx_index: u32,
    tx_position: u16,
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

mod serde {
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
mod rand {
    use super::ChannelId;
    use super::ShortChannelId;

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

#[cfg(test)]
mod tests {
    use super::ShortChannelId;
    use rand;

    #[test]
    fn short_channel_id_packing() {
        let value: u64 = rand::random();
        let short_channel_id = ShortChannelId::from(value);
        let restored = short_channel_id.into();
        assert_eq!(value, restored);
    }
}
