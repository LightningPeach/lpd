use super::Hash256;
use super::ChannelId;
use super::MilliSatoshi;
use super::Satoshi;
use super::SatoshiPerKiloWeight;
use super::CsvDelay;
use super::ChannelFlags;
use super::ChannelKeys;
use super::super::types::RawPublicKey;

#[cfg(test)]
use super::ChannelPrivateKeys;

use serde_derive::{Serialize, Deserialize};

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

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct OpenChannelShutdownScript {
    shutdown_script_pubkey: Vec<()>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ReestablishChannel {
    channel_id: ChannelId,
    next_local_commitment_number: u64,
    next_remote_revocation_number: u64,
    last_remote_commit_secret: [u8; 32],
    local_unrevoked_commit_point: RawPublicKey,
}

#[cfg(test)]
mod test {
    use super::*;
    use super::ChannelKeys;
    use binformat::BinarySD;
    use crate::message::channel::ChannelId;
    use crate::message::channel::operation::{UpdateFulfillHtlc, HtlcId, u8_32_from_hex};
    use crate::CsvDelay;
    use std::io::{Cursor, Read, Seek, SeekFrom};
    use crate::Message;
    use pretty_assertions::{assert_eq, assert_ne};
    use secp256k1::PublicKey;

    #[test]
    fn open_channel_ser() {
        use std::mem::size_of;
        use rand::Rng;
        use rand::thread_rng;

        let mut rng = thread_rng();
        let private: ChannelPrivateKeys = rng.gen();
        let mut vec = vec![];
        let msg = OpenChannel {
            chain_hash: Hash256::BITCOIN_CHAIN_HASH,//rng.gen(),
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
            keys: ChannelKeys::new(&private),
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

    #[test]
    fn accept_channel_test() {
        let msg_hex = "0021000a00000000000000000000000000000000000000000000000000000000000000000000000000640000000000018a88000000000000271000000000000003e900000002000a000702f4f54c706c49df82c35453fafcbe3fe55268e274651f50d573f8eeeee8b3a31d032dc1b351406ab5404a2d1c05dfeceb2fdee8228e3525a6be061bddf0a39bd6ad03d330de7e7e31acae3092babdc514570670b43fdf18d3ac0b397c9db2de52888f0297557fc325a8de27eca45e7f77db44f22b85d16d2ec5853adf7b21464e3c363202c5871b00d8d1bdedb91db3fb487959291da00ce179ef5a9172042e1a563773c7035281eef9aa59ce083ae6d614774bee20d586d2901262adfed1f8214dc5840e37";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = AcceptChannel {
            temporary_channel_id: ChannelId::from_hex("000a000000000000000000000000000000000000000000000000000000000000").unwrap(),
            dust_limit: Satoshi::from(100),
            max_htlc_value_in_flight: MilliSatoshi::from(101000),
            chanel_reserve: Satoshi::from(10000),
            htlc_minimum: MilliSatoshi::from(1001),
            minimum_accept_depth: 2,
            csv_delay: CsvDelay::from(10),
            max_accepted_htlc_number: 7,
            keys: ChannelKeys {
                funding: RawPublicKey::from_hex("02f4f54c706c49df82c35453fafcbe3fe55268e274651f50d573f8eeeee8b3a31d").unwrap(),
                revocation: RawPublicKey::from_hex("032dc1b351406ab5404a2d1c05dfeceb2fdee8228e3525a6be061bddf0a39bd6ad").unwrap(),
                payment: RawPublicKey::from_hex("03d330de7e7e31acae3092babdc514570670b43fdf18d3ac0b397c9db2de52888f").unwrap(),
                delayed_payment: RawPublicKey::from_hex("0297557fc325a8de27eca45e7f77db44f22b85d16d2ec5853adf7b21464e3c3632").unwrap(),
                htlc: RawPublicKey::from_hex("02c5871b00d8d1bdedb91db3fb487959291da00ce179ef5a9172042e1a563773c7").unwrap(),
                first_per_commitment: RawPublicKey::from_hex("035281eef9aa59ce083ae6d614774bee20d586d2901262adfed1f8214dc5840e37").unwrap(),
            }
        };
        let wrapped_msg_correct = Message::AcceptChannel(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }


    #[test]
    fn reestablish_channel_test() {
        let msg_hex = "00880100000000000000000000000000000000000000000000000000000000000000000000000000000b00000000000000020002000000000000000000000000000000000000000000000000000000000000031de8e2207c6ad1d81f5458c40b9cb1b519448ad67b00983e411ef522cbb187b6";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = ReestablishChannel {
            channel_id: ChannelId::from_hex("0100000000000000000000000000000000000000000000000000000000000000").unwrap(),
            next_local_commitment_number: 11,
            next_remote_revocation_number: 2,
            last_remote_commit_secret: u8_32_from_hex("0002000000000000000000000000000000000000000000000000000000000000").unwrap(),
            local_unrevoked_commit_point: RawPublicKey::from_hex("031de8e2207c6ad1d81f5458c40b9cb1b519448ad67b00983e411ef522cbb187b6").unwrap(),
        };
        let wrapped_msg_correct = Message::ReestablishChannel(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }
}


