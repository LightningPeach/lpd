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

/// This message contains information about a node and indicates its desire to set up
/// a new channel. This is the first step toward creating the funding transaction
/// and both versions of the commitment transaction.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct OpenChannel {
    /// Denotes the exact blockchain that the opened channel will reside within.
    /// This is usually the genesis hash of the respective blockchain. The existence
    /// of the `chain_hash` allows nodes to open channels across many distinct blockchains
    /// as well as have channels within multiple blockchains opened to the same peer
    /// (if it supports the target chains).
    pub chain_hash: Hash256,
    /// The `temporary_channel_id` is used to identify this channel on a per-peer basis
    /// until the funding transaction is established, at which point it is replaced
    /// by the channel_id, which is derived from the funding transaction.
    pub temporary_channel_id: ChannelId,
    /// The amount the sender is putting into the channel.
    pub funding: Satoshi,
    /// An amount of initial funds that the sender is unconditionally giving to the receiver.
    pub push: MilliSatoshi,
    /// The threshold below which outputs should not be generated
    /// for this node's commitment or HTLC transactions (i.e. HTLCs below this amount
    /// plus HTLC transaction fees are not enforceable on-chain).
    /// This reflects the reality that tiny outputs are not considered standard transactions
    /// and will not propagate through the Bitcoin network.
    pub dust_limit: Satoshi,
    /// A cap on total value of outstanding HTLCs, which allows a node
    /// to limit its exposure to HTLCs.
    pub max_in_flight: MilliSatoshi,
    /// The minimum amount that the other node is to keep as a direct payment.
    pub channel_reserve: Satoshi,
    /// Indicates the smallest value HTLC this node will accept.
    pub htlc_minimum: MilliSatoshi,
    /// Indicates the initial fee rate in satoshi per 1000-weight
    /// (i.e. 1/4 the more normally-used 'satoshi per 1000 vbytes')
    /// that this side will pay for commitment and HTLC transactions,
    /// as described in BOLT #3 (this can be adjusted later with an `update_fee` message).
    pub fee: SatoshiPerKiloWeight,
    /// The number of blocks that the other node's to-self outputs must be delayed,
    /// using `OP_CHECKSEQUENCEVERIFY` delays; this is how long it will have to wait
    /// in case of breakdown before redeeming its own funds.
    pub csv_delay: CsvDelay,
    /// Limits the number of outstanding HTLCs the other node can offer.
    pub max_accepted_htlc_number: u16,
    pub keys: ChannelKeys,
    pub flags: ChannelFlags,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct OpenChannelShutdownScript {
    shutdown_script_pubkey: Vec<()>,
}

/// This message contains information about a node and indicates its acceptance
/// of the new channel. This is the second step toward creating the funding transaction
/// and both versions of the commitment transaction.
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
    pub channel_id: ChannelId,

    /// A commitment number is a 48-bit incrementing counter for each commitment transaction;
    /// counters are independent for each peer in the channel and start at 0.
    /// They're only explicitly relayed to the other node in the case of re-establishment,
    /// otherwise they are implicit.
    pub next_local_commitment_number: u64,
    pub next_remote_revocation_number: u64,
    pub last_remote_commit_secret: [u8; 32],
    pub local_unrevoked_commit_point: RawPublicKey,
}

#[cfg(test)]
mod test {
    use super::*;
    use super::ChannelKeys;
    use binformat::BinarySD;
    use crate::message::channel::ChannelId;
    use crate::message::channel::operation::u8_32_from_hex;
    use crate::CsvDelay;
    use std::io::Cursor;
    use crate::Message;
    use pretty_assertions::assert_eq;

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
    fn open_channel_test() {
        let msg_hex = "\
            002000000c0000000000000000000000000000000000000000000000000000000000020000000000\
            000000000000000000000000000000000000000000000000000000000000000186a0000000000000\
            303500000000000000c8000000000000271000000000000003e800000000000003e80000000a000f\
            000a03aed565ae1dd10928cb333954d9d13326072451e247f73a7ec641272cff6e9a8a03a524d6aa\
            f0ab577a48665f783dad101e175fde3d6a6b82b4514d1620a248bdeb033e5ff9d4ec0a9537689c59\
            377c3fc1fab8c4d8473ff4d658f58464da855edf050384a8e93b5cec3771a679f0440883dc1afe9f\
            b57193dbb6f03b071e5037972a890293cc716c3039c6b089bbad8da01be38e66600c708a9a6d57c6\
            b34acde072c16a028e95ee83d07fa9f2927a8a65152917bb5d41253a7b0b56664b083c596d35178a\
            01";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = OpenChannel {
            chain_hash: Hash256::from_hex("00000c0000000000000000000000000000000000000000000000000000000000").unwrap(),
            temporary_channel_id: ChannelId::from_hex("0200000000000000000000000000000000000000000000000000000000000000").unwrap(),
            funding: Satoshi::from(100000),
            push: MilliSatoshi::from(12341),
            dust_limit: Satoshi::from(200),
            max_in_flight: MilliSatoshi::from(10000),
            channel_reserve: Satoshi::from(1000),
            htlc_minimum: MilliSatoshi::from(1000),
            fee: SatoshiPerKiloWeight::from(10),
            csv_delay: CsvDelay::from(15),
            max_accepted_htlc_number: 10,
            keys: ChannelKeys {
                funding: RawPublicKey::from_hex("03aed565ae1dd10928cb333954d9d13326072451e247f73a7ec641272cff6e9a8a").unwrap(),
                revocation: RawPublicKey::from_hex("03a524d6aaf0ab577a48665f783dad101e175fde3d6a6b82b4514d1620a248bdeb").unwrap(),
                payment: RawPublicKey::from_hex("033e5ff9d4ec0a9537689c59377c3fc1fab8c4d8473ff4d658f58464da855edf05").unwrap(),
                delayed_payment: RawPublicKey::from_hex("0384a8e93b5cec3771a679f0440883dc1afe9fb57193dbb6f03b071e5037972a89").unwrap(),
                htlc: RawPublicKey::from_hex("0293cc716c3039c6b089bbad8da01be38e66600c708a9a6d57c6b34acde072c16a").unwrap(),
                first_per_commitment: RawPublicKey::from_hex("028e95ee83d07fa9f2927a8a65152917bb5d41253a7b0b56664b083c596d35178a").unwrap(),
            },
            flags: ChannelFlags::from_u8(1),
        };
        let wrapped_msg_correct = Message::OpenChannel(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }

    #[test]
    fn accept_channel_test() {
        let msg_hex = "\
            0021000a000000000000000000000000000000000000000000000000000000000000000000000000\
            00640000000000018a88000000000000271000000000000003e900000002000a000702f4f54c706c\
            49df82c35453fafcbe3fe55268e274651f50d573f8eeeee8b3a31d032dc1b351406ab5404a2d1c05\
            dfeceb2fdee8228e3525a6be061bddf0a39bd6ad03d330de7e7e31acae3092babdc514570670b43f\
            df18d3ac0b397c9db2de52888f0297557fc325a8de27eca45e7f77db44f22b85d16d2ec5853adf7b\
            21464e3c363202c5871b00d8d1bdedb91db3fb487959291da00ce179ef5a9172042e1a563773c703\
            5281eef9aa59ce083ae6d614774bee20d586d2901262adfed1f8214dc5840e37";
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
        let msg_hex = "\
            00880100000000000000000000000000000000000000000000000000000000000000000000000000\
            000b0000000000000002000200000000000000000000000000000000000000000000000000000000\
            0000031de8e2207c6ad1d81f5458c40b9cb1b519448ad67b00983e411ef522cbb187b6";
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


