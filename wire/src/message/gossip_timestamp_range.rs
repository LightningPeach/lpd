use super::types::Hash256;

use std::ops::Range;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct GossipTimestampRange {
    chain_hash: Hash256,
    first_timestamp: u32,
    timestamp_range: u32
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
