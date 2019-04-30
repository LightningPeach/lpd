use super::types::RawFeatureVector;
use super::channel::ChannelId;

use serde_derive::{Serialize, Deserialize};

/// The first message reveals the features supported or required by this node,
/// even if this is a reconnection
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Init {
    pub global_features: RawFeatureVector,
    pub local_features: RawFeatureVector,
}

impl Init {
    pub fn new(global_features: RawFeatureVector, local_features: RawFeatureVector) -> Self {
        Init {
            global_features: global_features as _,
            local_features: local_features as _,
        }
    }
}

/// The channel is referred to by `channel_id`,
/// unless `channel_id` is 0 (i.e. all bytes are 0), in which case it refers to all channels.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Error {
    pub channel_id: ChannelId,
    pub data: Vec<u8>,
}

#[cfg(test)]
mod test {
    use binformat::BinarySD;

    use super::Init;
    use super::super::types::RawFeatureVector;
    use super::super::types::FeatureBit;

    use crate::message::channel::ChannelId;
    use std::io::{Cursor, Read, Seek, SeekFrom};
    use crate::{Message, RevokeAndAck, RawPublicKey, CommitmentSigned, RawSignature};
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_init_serde() {
        use self::FeatureBit::*;

        let init = Init::new(
            RawFeatureVector::new()
                .set_bit(DataLossProtectRequired),
            RawFeatureVector::new()
                .set_bit(DataLossProtectOptional)
                .set_bit(GossipQueriesOptional)
        );

        let mut data = Vec::<u8>::new();
        BinarySD::serialize(&mut data, &init).unwrap();

        println!("{:?}", data);
        let new = BinarySD::deserialize(&data[..]).unwrap();

        assert_eq!(init, new);
    }


    #[test]
    fn init_test() {
        let msg_hex = "001000000001cb";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = Init {
            global_features: RawFeatureVector::from_hex("0000").unwrap(),
            local_features: RawFeatureVector::from_hex("0001cb").unwrap(),
        };
        let wrapped_msg_correct = Message::Init(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }
}
