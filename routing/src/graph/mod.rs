use specs::prelude::*;

use wire::AnnouncementNode;
use wire::Message;

mod channel;

#[derive(Component)]
pub struct NodeComponent {
    pub v: AnnouncementNode,
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
            Some(_) => println!("warning, unknown message type, ignoring"),
        }
    }
}

pub struct LogGraphSystem;

#[derive(SystemData)]
pub struct EnumerateNodesSystemData<'a> {
    nodes: ReadStorage<'a, NodeComponent>,
}

impl<'a> System<'a> for LogGraphSystem {
    type SystemData = EnumerateNodesSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        (&data.nodes).join().for_each(|n| println!("{:?}", n.v));
    }
}

pub struct Graph {
    world: World,
}

impl Graph {
    pub fn new() -> Self {
        use self::channel::AnnouncementChannelSystem;
        use self::channel::UpdateChannelSystem;

        let mut world = World::new();
        world.setup::<IncomingMessageSystemData>();
        world.setup::<EnumerateNodesSystemData>();
        world.setup::<<AnnouncementChannelSystem as System>::SystemData>();
        world.setup::<<UpdateChannelSystem as System>::SystemData>();

        Graph {
            world: world,
        }
    }

    pub fn message(&mut self, message: Message) {
        use self::Message::*;
        use self::channel::AnnouncementChannelSystem;
        use self::channel::UpdateChannelSystem;

        match message {
            AnnouncementChannel(v) => AnnouncementChannelSystem::from(v).run_now(&mut self.world.res),
            UpdateChannel(v) => UpdateChannelSystem::from(v).run_now(&mut self.world.res),
            message => IncomingMessageSystem::new(message).run_now(&mut self.world.res),
        }
        self.world.maintain();
    }

    pub fn enumerate_nodes(&mut self) {
        use self::channel::LogChannelsSystem;

        LogGraphSystem.run_now(&mut self.world.res);
        LogChannelsSystem.run_now(&mut self.world.res);
    }
}
