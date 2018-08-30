use wire::GossipTimestampRange;
use wire::QueryChannelRange;
use wire::QueryShortChannelIds;
use wire::Message;
use wire::Hash256;

use super::peer::*;

pub enum SynchronizationState {
    Synchronizing,
    WaitingQueryRangeReply,
    QueryNewChannels,
    WaitingQueryChannelReply,
    Synchronized,
}

pub struct Synchronization {
    //remote_update_horizon: GossipTimestampRange,
    //local_update_horizon: GossipTimestampRange,
    //state: SynchronizationState,
}

impl Synchronization {
    pub fn sync_channels<P>(self, peer: &mut P) where P: Peer, {
        let msg = Message::QueryChannelRange(QueryChannelRange::new(
            Hash256::TEST_HASH, 0, 120));
        let response = peer.synchronous_message(msg);
        println!("{:?}", response.ok().unwrap());
    }
}
