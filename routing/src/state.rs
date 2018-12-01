use specs::prelude::*;
use super::db::DB;
use rocksdb::Error as DBError;
use super::channel::{LoadChannels, StoreChannels, LogChannels, ChannelInfo};
use super::node::{LoadNodes, StoreNodes, LogNodes, Node};
use super::tools::GenericSystem;

use wire::{
    Message, Init, AnnouncementNode, AnnouncementChannel, UpdateChannel,
    MessageFiltered, MessageConsumer, WireError,
};

use std::fmt::Debug;
use std::path::Path;

#[cfg(feature = "rpc")]
use interface::routing::{ChannelEdge, LightningNode};

use tokio::prelude::{Future, Sink};

pub struct State {
    world: World,
}

impl State {
    pub fn new<P>(path: P) -> Result<Self, DBError>
    where
        P: AsRef<Path>,
    {
        let mut db = DB::new(path)?;
        db.register::<ChannelInfo>()?;
        db.register::<Node>()?;
        let mut world = World::new();
        world.setup::<<GenericSystem<AnnouncementChannel, ()> as System>::SystemData>();
        world.setup::<<GenericSystem<UpdateChannel, ()> as System>::SystemData>();
        world.setup::<<GenericSystem<AnnouncementNode, ()> as System>::SystemData>();
        world.setup::<<GenericSystem<LoadNodes, Result<(), DBError>> as System>::SystemData>();
        world.setup::<<GenericSystem<StoreNodes, Result<(), DBError>> as System>::SystemData>();
        world.setup::<<GenericSystem<LogNodes, ()> as System>::SystemData>();
        world.setup::<<GenericSystem<LoadChannels, Result<(), DBError>> as System>::SystemData>();
        world.setup::<<GenericSystem<StoreChannels, Result<(), DBError>> as System>::SystemData>();
        world.setup::<<GenericSystem<LogChannels, ()> as System>::SystemData>();
        world.add_resource(db);
        Ok(State {
            world: world,
        })
    }

    fn run<'a, Input, Output>(&'a mut self, input: Input) -> Output
    where
        GenericSystem<Input, Output>: System<'a>,
        Input: Debug,
        Output: Debug,
    {
        let mut system = GenericSystem::from(input);
        system.run_now(&mut self.world.res);
        system.output()
    }

    pub fn load(&mut self) -> Result<(), DBError> {
        self.run(LoadChannels)?;
        self.run(LoadNodes)
    }

    pub fn store(&mut self) -> Result<(), DBError> {
        self.run(StoreChannels)?;
        self.run(StoreNodes)
    }

    #[cfg(feature = "rpc")]
    pub fn describe(&self, include_unannounced: bool) -> (Vec<ChannelEdge>, Vec<LightningNode>) {
        use specs::Join;
        use super::channel::{ChannelId, ChannelParties, ChannelHistory};
        use super::node::Node;

        let _ = include_unannounced; // TODO: use it
        let (mut e, mut n) = (Vec::new(), Vec::new());

        let channel_data = (
            &self.world.read_storage::<ChannelId>(),
            &self.world.read_storage::<ChannelParties>(),
            &self.world.read_storage::<ChannelHistory>()
        );
        for (id, parties, history) in channel_data.join() {
            let info = ChannelInfo(id.clone(), parties.clone(), history.clone());
            e.push(info.into());
        }

        let node_data = &self.world.read_storage::<Node>();
        for node in node_data.join() {
            n.push(node.clone().into())
        }

        (e, n)
    }
}

pub enum TopologyMessage {
    Init(Init),
    AnnouncementNode(AnnouncementNode),
    AnnouncementChannel(AnnouncementChannel),
    UpdateChannel(UpdateChannel),
}

impl MessageFiltered for TopologyMessage {
    fn filter(v: Message) -> Result<Self, Message> {
        match v {
            Message::Init(v) => Ok(TopologyMessage::Init(v)),
            Message::AnnouncementNode(v) => Ok(TopologyMessage::AnnouncementNode(v)),
            Message::AnnouncementChannel(v) => Ok(TopologyMessage::AnnouncementChannel(v)),
            Message::UpdateChannel(v) => Ok(TopologyMessage::UpdateChannel(v)),
            v @ _ => Err(v),
        }
    }
}

impl MessageConsumer for State {
    type Message = TopologyMessage;

    fn consume<S>(mut self, sink: S, message: Self::Message) -> Box<dyn Future<Item=(Self, S), Error=WireError>>
    where
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        use tokio::prelude::IntoFuture;
        use self::TopologyMessage::*;

        match message {
            Init(_) => (),
            AnnouncementNode(v) => self.run(v),
            AnnouncementChannel(v) => self.run(v),
            UpdateChannel(v) => self.run(v),
        };
        self.world.maintain();
        Box::new(Ok((self, sink)).into_future())
    }
}
