use wire::AnnouncementNode;
use wire::PublicKey;
use wire::Color;
use wire::Address;
use wire::NodeAlias;
use wire::SignError;

use specs::DenseVecStorage;
use specs::System;
use specs::Entities;
use specs::Read;
use specs::ReadStorage;
use specs::LazyUpdate;

use super::tools::UseOnce;

#[derive(Component, Eq, PartialEq, Debug, Clone)]
pub struct Node {
    timestamp: u32,
    node_id: PublicKey,
    color: Color,
    alias: NodeAlias,
    address: Vec<Address>,
}

// TODO: add rebroadcasting subsystem
impl<'a> System<'a> for UseOnce<AnnouncementNode> {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Node>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;
        use std::iter::Iterator;

        self.consume().map(|announcement_node| {
            let (entities, update, node_storage) = (&*data.0, &*data.1, data.2);

            // TODO: check features

            let announcement_node = match announcement_node.verify_owned(|s| &s.node_id) {
                Ok(s) => s.0,
                // TODO: fail the connection
                Err(SignError::IncorrectSignature) => return,
                Err(e) => panic!("error {:?}", e),
            };

            let node = Node {
                timestamp: announcement_node.timestamp,
                node_id: announcement_node.node_id,
                color: announcement_node.color,
                alias: announcement_node.alias,
                address: announcement_node.address.0,
            };

            if (&node_storage).join().find(|&n| n == &node).is_none() {
                let node_ref = entities.create();
                update.insert(node_ref, node);
            }
        });
    }
}

pub struct LogNodesSystem;

impl<'a> System<'a> for LogNodesSystem {
    type SystemData = (
        ReadStorage<'a, Node>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;

        println!("nodes: ");
        for node in (&data.0).join() {
            let space = "    ";
            println!("{} {:?}", space, node);
        }
    }
}

#[cfg(feature = "rpc")]
mod rpc {
    use interface::routing::{LightningNode, NodeAddress};
    use wire::BinarySD;
    use super::{Node, NodeAlias};

    impl From<Node> for LightningNode {
        fn from(v: Node) -> Self {
            let mut r = LightningNode::new();
            r.set_last_update(v.timestamp);
            r.set_pub_key(v.node_id.to_string());
            r.set_color(v.color.to_string());
            r.set_alias(v.alias.string());
            r.set_addresses(v.address.into_iter().map(|address|
                address.into_socket_address().map(|socket_address| {
                    let mut n = NodeAddress::new();
                    n.set_addr(socket_address.to_string());
                    n
                }).unwrap_or(NodeAddress::new())
            ).collect::<Vec<_>>().into());
            r
        }
    }
}
