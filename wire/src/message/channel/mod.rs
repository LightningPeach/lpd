use super::types::*;

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
