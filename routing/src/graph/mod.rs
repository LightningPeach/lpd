use specs::prelude::*;

use super::TopologyMessage;
use wire::MessageFiltered;

mod channel;
mod node;
mod tools;

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
            _ => (),
        }
        self.world.maintain();
    }

    pub fn enumerate_nodes(&mut self) {
        use self::node::LogNodesSystem;
        use self::channel::LogChannelsSystem;

        LogNodesSystem.run_now(&mut self.world.res);
        LogChannelsSystem.run_now(&mut self.world.res);
    }
}
