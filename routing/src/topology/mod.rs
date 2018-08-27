mod node;

pub use self::node::LightningNode;
pub use self::node::ChannelAuthProof;
pub use self::node::ChannelInfo;
pub use self::node::ChannelPolicy;

use wire::ShortChannelId;
use wire::PublicKey;
use wire::ChannelUpdateFlags;

use chrono::Utc;
use chrono::DateTime;

pub trait ChannelGraph {
    type Error;

    fn add_node(&mut self, node: LightningNode) -> Result<(), Self::Error>;

    fn add_edge(&mut self, edge: ChannelInfo) -> Result<(), Self::Error>;

    fn add_proof(
        &mut self,
        short_channel_id: ShortChannelId,
        proof: ChannelAuthProof,
    ) -> Result<(), Self::Error>;

    fn update_edge(&mut self, update: ChannelPolicy) -> Result<(), Self::Error>;

    fn is_stale_node(&self, public_key: PublicKey, time: DateTime<Utc>) -> bool;

    fn is_known_edge(&self, short_channel_id: ShortChannelId) -> bool;

    fn is_stale_edge_policy(
        &self,
        short_channel_id: ShortChannelId,
        time: DateTime<Utc>,
        flags: ChannelUpdateFlags
    ) -> bool;

    fn for_each_outgoing_channels<F>(&mut self, f: F) -> Result<(), Self::Error>
        where F: FnMut(ChannelInfo, ChannelPolicy) -> Result<(), Self::Error>;

    fn current_block_height(&self) -> Result<u32, Self::Error>;

    fn get_channel_by_id(
        &self,
        short_channel_id: ShortChannelId
    ) -> Result<(ChannelInfo, ChannelPolicy, ChannelPolicy), Self::Error>;

    fn for_each_nodes<F>(&mut self, f: F) -> Result<(), Self::Error>
        where F: FnMut(LightningNode) -> Result<(), Self::Error>;

    fn for_each_channel<F>(&mut self, f: F) -> Result<(), Self::Error>
        where F: FnMut(ChannelInfo, ChannelPolicy, ChannelPolicy) -> Result<(), Self::Error>;
}
