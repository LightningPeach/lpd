use super::types::Hash;
use super::types::Satoshi;
use super::types::MilliSatoshi;
use super::types::SatoshiPerKiloWeight;
use super::types::CsvDelay;
use super::types::PublicKey;
use super::types::Signature;
use super::types::OutputIndex;

mod funding;
pub use self::funding::*;

mod close;
pub use self::close::*;

mod operation;
pub use self::operation::*;

// TODO: use crate for bitflags
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum ChannelFlags {
    FFAnnounceChannel = 1 << 0,
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
pub struct OpenChannel {
    chain_hash: Hash,
    temporary_channel_id: ChannelId,
    funding: Satoshi,
    push: MilliSatoshi,
    dust_limit: Satoshi,
    max_in_flight: MilliSatoshi,
    channel_reserve: Satoshi,
    htlc_minimum: MilliSatoshi,
    fee: SatoshiPerKiloWeight,
    csv_delay: CsvDelay,
    max_accepted_htlc_number: u16,
    funding_pubkey: PublicKey,
    revocation_basepoint: PublicKey,
    payment_basepoint: PublicKey,
    delayed_payment_basepoint: PublicKey,
    htlc_basepoint: PublicKey,
    first_per_commitment_point: PublicKey,
    flags: u8,
    script: Vec<u8>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct OpenChannelShutdownScript {
    shutdown_script_pubkey: Vec<()>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct AcceptChannel {
    temporary_channel_id: ChannelId,
    dust_limit: Satoshi,
    max_htlc_value_in_flight: MilliSatoshi,
    chanel_reserve: Satoshi,
    htlc_minimum: MilliSatoshi,
    minimum_accept_depth: u32,
    csv_delay: CsvDelay,
    max_accepted_htlc_number: u16,
    funding_pubkey: PublicKey,
    revocation_point: PublicKey,
    payment_point: PublicKey,
    delayed_payment_point: PublicKey,
    htlc_point: PublicKey,
    first_per_commitment_point: PublicKey,
    script: Vec<u8>,
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

#[cfg(test)]
mod test {
    use super::super::types::*;
    use super::*;
    use ::BinarySD;

    #[test]
    fn open_channel_ser() {
        use std::mem::size_of;
        use rand::Rng;
        use rand::thread_rng;

        let mut rng = thread_rng();
        let mut vec = vec![];
        let msg = OpenChannel {
            chain_hash: rng.gen(),
            temporary_channel_id: rng.gen(),
            funding: Satoshi::default(),
            push: MilliSatoshi::default(),
            dust_limit: Satoshi::default(),
            max_in_flight: MilliSatoshi::default(),
            channel_reserve: Satoshi::default(),
            htlc_minimum: MilliSatoshi::default(),
            fee: SatoshiPerKiloWeight::default(),
            csv_delay: CsvDelay::default(),
            max_accepted_htlc_number: Default::default(),
            funding_pubkey: rng.gen(),
            revocation_basepoint: rng.gen(),
            payment_basepoint: rng.gen(),
            delayed_payment_basepoint: rng.gen(),
            htlc_basepoint: rng.gen(),
            first_per_commitment_point: rng.gen(),
            flags: ChannelFlags::FFAnnounceChannel as _,
            script: vec![],
        };

        // try to estimate size without aligning
        let estimated_size = size_of::<Hash>() + size_of::<ChannelId>()
            + size_of::<Satoshi>() * 3 + size_of::<MilliSatoshi>() * 3
            + size_of::<SatoshiPerKiloWeight>() + size_of::<CsvDelay>()
            + size_of::<u16>() + size_of::<PublicKey>() * 6 + size_of::<u8>();

        let _ = BinarySD::serialize(&mut vec, &msg).unwrap();
        println!("{:?} == {:?}", vec.len(), estimated_size);
        assert_eq!(vec.len(), estimated_size);

        let restored: OpenChannel = BinarySD::deserialize(vec.as_slice()).unwrap();
        assert_eq!(restored, msg);
    }
}
