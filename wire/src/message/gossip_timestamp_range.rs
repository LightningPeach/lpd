use super::types::Sha256;

use std::ops::Range;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct GossipTimestampRange {
    pub chain_hash: Sha256,
    pub first_timestamp: u32,
    pub timestamp_range: u32
}

impl GossipTimestampRange {
    pub fn range(&self) -> Range<u32> {
        // warning, here might be one position error
        // the specification says
        // https://github.com/lightningnetwork/lightning-rfc/blob/master/07-routing-gossip.md
        // `less than first_timestamp plus timestamp_range`
        // but the lnd code says
        // `The receiving node MUST
        //	// NOT send any announcements that have a timestamp greater than
        //	// FirstTimestamp + TimestampRange`
        // the case that announcements have a timestamp *equal* FirstTimestamp + TimestampRange
        // is not covered in the lnd
        // this code respects the specification, rather than lnd implementation
        let end = self.first_timestamp + self.timestamp_range;
        self.first_timestamp..end
    }
}

#[cfg(test)]
mod test {
    use dependencies::hex;
    use dependencies::pretty_assertions;

    use binformat::BinarySD;
    use std::io::Cursor;
    use crate::{Message, GossipTimestampRange};
    use pretty_assertions::assert_eq;
    use common_types::Sha256;

    #[test]
    fn gossip_timestamp_range_test() {
        let msg_hex = "0109000b00000000000000000000000000000000000000000000000000000000000005f5e100000004d2";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = GossipTimestampRange {
            chain_hash: Sha256::from_hex("000b000000000000000000000000000000000000000000000000000000000000").unwrap(),
            first_timestamp: 100000000,
            timestamp_range: 1234,
        };
        let wrapped_msg_correct = Message::GossipTimestampRange(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }
}