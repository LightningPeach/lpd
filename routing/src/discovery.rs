use wire::Hash256;
use super::node::LightningNode;
use super::topology::ChannelGraph;

use std::error::Error;

pub struct Config<G> where G: ChannelGraph {
    chain_hash: Hash256,
    router: G,
    // TODO(vlad):

}

pub struct Gossiper<G> where G: ChannelGraph {
    base_height: u32,
    config: Config<G>,
    // TODO(vlad):
}
