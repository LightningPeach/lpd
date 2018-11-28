use specs::prelude::*;

use super::TopologyMessage;
use wire::MessageFiltered;

mod channel;
mod node;
mod tools;

#[cfg(feature = "rpc")]
use interface::routing::{ChannelEdge, LightningNode};

pub struct Graph {
    world: World,
}

impl Graph {
    pub fn new() -> Self {
        use self::channel::{AnnouncementChannelSystem, UpdateChannelSystem};
        use self::node::LogNodesSystem;
        use self::tools::UseOnce;
        use wire::AnnouncementNode;

        let mut world = World::new();
        world.setup::<<AnnouncementChannelSystem as System>::SystemData>();
        world.setup::<<UpdateChannelSystem as System>::SystemData>();
        world.setup::<<UseOnce<AnnouncementNode> as System>::SystemData>();
        world.setup::<<LogNodesSystem as System>::SystemData>();

        Graph {
            world: world,
        }
    }

    pub fn message(&mut self, message: TopologyMessage) {
        use self::TopologyMessage::*;
        use self::channel::{AnnouncementChannelSystem, UpdateChannelSystem};
        use self::tools::UseOnce;

        match message {
            Init(v) => (),  // TODO:
            AnnouncementChannel(v) => AnnouncementChannelSystem::from(v).run_now(&mut self.world.res),
            UpdateChannel(v) => UpdateChannelSystem::from(v).run_now(&mut self.world.res),
            AnnouncementNode(v) => UseOnce::from(v).run_now(&mut self.world.res),
        }
        self.world.maintain();
    }

    pub fn enumerate_nodes(&mut self) {
        use self::node::LogNodesSystem;
        use self::channel::LogChannelsSystem;

        LogNodesSystem.run_now(&mut self.world.res);
        LogChannelsSystem.run_now(&mut self.world.res);
    }

    #[cfg(feature = "rpc")]
    pub fn describe(&self, include_unannounced: bool) -> (Vec<ChannelEdge>, Vec<LightningNode>) {
        use specs::Join;
        use self::channel::{ChannelId, ChannelParties, ChannelPolicy, ChannelHistory, ChannelInfo};
        use self::node::Node;

        let _ = include_unannounced; // TODO: use it
        let (mut e, mut n) = (Vec::new(), Vec::new());
        let channel_data = (
            &self.world.read_storage::<ChannelId>(),
            &self.world.read_storage::<ChannelParties>(),
            &self.world.read_storage::<ChannelHistory>()
        );

        for (id, parties, history) in channel_data.join() {
            let space = "    ";
            let info = ChannelInfo(id, parties, history.current());
            e.push(info.into());
        }

        let node_data = &self.world.read_storage::<Node>();
        for node in node_data.join() {
            n.push(node.clone().into())
        }

        (e, n)
    }
}
