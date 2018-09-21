use super::Hash256;
use super::ChannelId;
use super::MilliSatoshi;
use super::Satoshi;
use super::SatoshiPerKiloWeight;
use super::CsvDelay;
use super::PublicKey;
use super::ChannelFlags;
use super::ChannelKeys;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct OpenChannel {
    pub chain_hash: Hash256,
    pub temporary_channel_id: ChannelId,
    pub funding: Satoshi,
    pub push: MilliSatoshi,
    pub dust_limit: Satoshi,
    pub max_in_flight: MilliSatoshi,
    pub channel_reserve: Satoshi,
    pub htlc_minimum: MilliSatoshi,
    pub fee: SatoshiPerKiloWeight,
    pub csv_delay: CsvDelay,
    pub max_accepted_htlc_number: u16,
    pub keys: ChannelKeys,
    pub flags: ChannelFlags,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct OpenChannelShutdownScript {
    shutdown_script_pubkey: Vec<()>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct AcceptChannel {
    pub temporary_channel_id: ChannelId,
    pub dust_limit: Satoshi,
    pub max_htlc_value_in_flight: MilliSatoshi,
    pub chanel_reserve: Satoshi,
    pub htlc_minimum: MilliSatoshi,
    pub minimum_accept_depth: u32,
    pub csv_delay: CsvDelay,
    pub max_accepted_htlc_number: u16,
    pub keys: ChannelKeys,
}

impl AcceptChannel {
    pub fn accept(open_channel: &OpenChannel, keys: &ChannelKeys) -> Self {
        AcceptChannel {
            temporary_channel_id: open_channel.temporary_channel_id.clone(),
            dust_limit: open_channel.dust_limit.clone(),
            max_htlc_value_in_flight: open_channel.max_in_flight.clone(),
            chanel_reserve: open_channel.channel_reserve.clone(),
            htlc_minimum: open_channel.htlc_minimum.clone(),
            minimum_accept_depth: 1,
            csv_delay: open_channel.csv_delay.clone(),
            max_accepted_htlc_number: open_channel.max_accepted_htlc_number.clone(),
            keys: keys.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct ReestablishChannel {
    channel_id: ChannelId,
    next_local_commitment_number: u64,
    next_remote_revocation_number: u64,
    last_remote_commit_secret: [u8; 32],
    local_unrevoked_commit_point: PublicKey,
}

#[cfg(test)]
mod test {
    use super::*;
    use ::BinarySD;

    #[test]
    fn open_channel_ser() {
        use std::mem::size_of;
        use rand::Rng;
        use rand::thread_rng;

        let mut rng = thread_rng();
        let private: ChannelPrivateKeys = rng.gen();
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
            keys: ChannelKeys::new(&private).unwrap(),
            flags: ChannelFlags::FF_ANNOUNCE_CHANNEL,
        };

        // try to estimate size without aligning
        let estimated_size = size_of::<Hash256>() + size_of::<ChannelId>()
            + size_of::<Satoshi>() * 3 + size_of::<MilliSatoshi>() * 3
            + size_of::<SatoshiPerKiloWeight>() + size_of::<CsvDelay>()
            + size_of::<u16>() + 33 * 6 + size_of::<u8>();

        let _ = BinarySD::serialize(&mut vec, &msg).unwrap();
        println!("{:?} == {:?}", vec.len(), estimated_size);
        assert_eq!(vec.len(), estimated_size);

        let restored: OpenChannel = BinarySD::deserialize(vec.as_slice()).unwrap();
        assert_eq!(restored, msg);
    }
}
