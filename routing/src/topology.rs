use super::node::LightningNode;
use super::node::ChannelAuthProof;
use super::node::ChannelInfo;
use wire::ShortChannelId;

pub trait ChannelGraph {
    type Error;

    fn add_node(&mut self, node: LightningNode) -> Result<(), Self::Error>;
    fn add_edge(&mut self, edge: ChannelInfo) -> Result<(), Self::Error>;
    fn add_proof(
        &mut self,
        short_channel_id: ShortChannelId,
        proof: ChannelAuthProof,
    ) -> Result<(), Self::Error>;
    fn update_edge(&mut self) -> Result<(), Self::Error>;
}
