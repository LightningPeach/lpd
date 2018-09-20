use specs::prelude::*;

use wire::AnnouncementNode;
use wire::AnnouncementChannel;
use wire::UpdateChannel;
use wire::Message;

#[derive(Component)]
pub struct NodeComponent {
    pub v: AnnouncementNode,
}

#[derive(Component)]
pub struct ChannelComponent {
    pub v: AnnouncementChannel,
}

#[derive(Component)]
pub struct ChannelPolicyComponent {
    pub v: UpdateChannel,
}

pub struct IncomingMessageSystem {
    data: Option<Message>,
}

impl IncomingMessageSystem {
    pub fn new(v: Message) -> Self {
        IncomingMessageSystem {
            data: Some(v),
        }
    }
}

#[derive(SystemData)]
pub struct IncomingMessageSystemData<'a> {
    entities: Entities<'a>,
    updates: Read<'a, LazyUpdate>,
}

impl<'a> System<'a> for IncomingMessageSystem {
    type SystemData = IncomingMessageSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        use self::Message::*;
        use std::mem;

        let mut temp = None;
        mem::swap(&mut temp, &mut self.data);

        match temp {
            None => println!("warning, looks like IncomingMessageSystem reused, ignoring"),
            Some(AnnouncementNode(v)) => {
                let component = NodeComponent { v: v };
                let entity = (&data.entities).create();
                data.updates.insert(entity, component);
            },
            Some(AnnouncementChannel(v)) => {
                let component = ChannelComponent { v: v };
                let entity = (&data.entities).create();
                data.updates.insert(entity, component);
            },
            Some(UpdateChannel(v)) => {
                let component = ChannelPolicyComponent { v: v };
                let entity = (&data.entities).create();
                data.updates.insert(entity, component);
            },
            Some(_) => println!("warning, unknown message type, ignoring"),
        }
    }
}

pub struct EnumerateNodesSystem;

#[derive(SystemData)]
pub struct EnumerateNodesSystemData<'a> {
    nodes: ReadStorage<'a, NodeComponent>,
    channels: ReadStorage<'a, ChannelComponent>,
    policies: ReadStorage<'a, ChannelPolicyComponent>,
}

impl<'a> System<'a> for EnumerateNodesSystem {
    type SystemData = EnumerateNodesSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        (&data.nodes).join().for_each(|n| println!("{:?}", n.v))
    }
}

pub struct Graph {
    world: World,
}

impl Graph {
    pub fn new() -> Self {
        let mut world = World::new();
        world.setup::<IncomingMessageSystemData>();
        world.setup::<EnumerateNodesSystemData>();

        Graph {
            world: world,
        }
    }

    pub fn message(&mut self, message: Message) {
        IncomingMessageSystem::new(message)
            .run_now(&mut self.world.res);
        self.world.maintain();
    }

    pub fn enumerate_nodes(&mut self) {
        EnumerateNodesSystem
            .run_now(&mut self.world.res);
    }
}
