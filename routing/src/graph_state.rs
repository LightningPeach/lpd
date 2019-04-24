use specs::prelude::*;
use state::{DB, DBBuilder, DBError, DBUser};
use super::channel::{
    LoadChannels, StoreChannels, LogChannels, ChannelInfo,
    ChannelRef, ChannelLinks,
};
use super::node::{LoadNodes, StoreNodes, LogNodes, Node, NodeRef, NodeLinks};
use super::tools::GenericSystem;

use dijkstras_search::Graph;

use either::Either;

#[cfg(feature = "rpc")]
use secp256k1::PublicKey;

use wire::{Message, MessageExt, Init, AnnouncementNode, AnnouncementChannel, UpdateChannel};
use processor::{MessageFiltered, MessageConsumer, ConsumingFuture};

use binformat::WireError;

use std::fmt::Debug;
use std::sync::{Arc, RwLock};

#[cfg(feature = "rpc")]
use interface::routing::{ChannelEdge, LightningNode};

use tokio::prelude::*;

pub struct State {
    world: World,
}

impl DBUser for State {
    fn db_prepare(builder: DBBuilder) -> DBBuilder {
        builder
            .register::<ChannelInfo>()
            .register::<Node>()
    }
}

impl State {
    pub fn new(db: Arc<RwLock<DB>>) -> Self {
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

        State {
            world: world,
        }
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

    #[cfg(feature = "rpc")]
    pub fn path(&self, start: PublicKey, goal: PublicKey) -> Vec<(LightningNode, ChannelEdge)> {
        let entities = &self.world.entities();
        let nodes = &self.world.read_storage::<Node>();
        let (mut start_ref, mut goal_ref) = (None, None);
        for (entity, node) in (entities, nodes).join() {
            if node.id().eq(&start) {
                start_ref = Some(NodeRef(entity.clone()));
            } else if node.id().eq(&goal) {
                goal_ref = Some(NodeRef(entity.clone()));
            }
        }

        match (start_ref, goal_ref) {
            (Some(start_ref), Some(goal_ref)) => {
                self.shortest_path(&self.world, start_ref.clone())
                    .sequence(start_ref, goal_ref)
                    .into_iter()
                    .map(|(node_ref, channel_ref)| {
                        use super::channel::{ChannelId, ChannelParties, ChannelHistory};

                        let node = nodes.get(node_ref.0).unwrap().clone().into();
                        let id = self.world.read_storage::<ChannelId>().get(channel_ref.0).unwrap().clone();
                        let parties = self.world.read_storage::<ChannelParties>().get(channel_ref.0).unwrap().clone();
                        let history = self.world.read_storage::<ChannelHistory>().get(channel_ref.0).unwrap().clone();
                        let info = ChannelInfo(id, parties, history);
                        let channel = info.into();

                        (node, channel)
                    })
                    .collect()
            },
            _ => Vec::new(),
        }
    }
}

impl Graph for State {
    type Node = NodeRef;
    type Edge = ChannelRef;
    type Context = World;

    fn neighbors(&self, node: Self::Node) -> Vec<(Self::Node, Self::Edge)> {
        let mut system: GenericSystem<NodeRef, Vec<(NodeRef, ChannelRef)>> = node.into();
        system.run((self.world.read_storage(), self.world.read_storage()));
        system.output()
    }
}

impl<'a> System<'a> for GenericSystem<NodeRef, Vec<(NodeRef, ChannelRef)>> {
    type SystemData = (
        ReadStorage<'a, NodeLinks>,
        ReadStorage<'a, ChannelLinks>,
    );

    fn run(&mut self, data: Self::SystemData) {
        self.run_func(|node| {
            data.0.get(node.0)
                .map(|node_links| node_links.0.clone())
                .unwrap_or(Vec::new())
                .into_iter()
                .filter_map(|channel_ref| {
                    data.1.get(channel_ref.0)
                        .and_then(|channel_links| {
                            match channel_links {
                                &ChannelLinks(Some(ref left), Some(ref right)) => {
                                    if *left == node {
                                        Some((right.clone(), channel_ref))
                                    } else if *right == node {
                                        Some((left.clone(), channel_ref))
                                    } else {
                                        None
                                    }
                                },
                                _ => None,
                            }
                        })
                })
                .collect()
        })
    }
}

#[derive(Debug)]
pub enum TopologyMessage {
    Init(Init),
    AnnouncementNode(AnnouncementNode),
    AnnouncementChannel(AnnouncementChannel),
    UpdateChannel(UpdateChannel),
}

impl MessageFiltered for TopologyMessage {
    fn filter(v: MessageExt) -> Result<Self, MessageExt> {
        match v.message {
            Message::Init(v) => Ok(TopologyMessage::Init(v)),
            Message::AnnouncementNode(v) => Ok(TopologyMessage::AnnouncementNode(v)),
            Message::AnnouncementChannel(v) => Ok(TopologyMessage::AnnouncementChannel(v)),
            Message::UpdateChannel(v) => Ok(TopologyMessage::UpdateChannel(v)),
            _ => Err(v),
        }
    }
}

#[derive(Clone)]
pub struct SharedState(pub Arc<RwLock<State>>);

impl MessageConsumer for SharedState {
    type Message = TopologyMessage;
    type Relevant = ();

    fn consume<S>(self, sink: S, message: Either<Self::Message, Self::Relevant>) -> ConsumingFuture<Self, S>
    where
        S: Sink<SinkItem=MessageExt, SinkError=WireError> + Send + 'static,
    {
        use wire::{Init, RawFeatureVector, FeatureBit::*};

        match message.left().unwrap() {
            TopologyMessage::Init(_) => {
                let local = RawFeatureVector::new().set_bit(InitialRoutingSync);
                let init = Message::Init(Init::new(RawFeatureVector::new(), local));
                return ConsumingFuture::from_send(self, sink.send(init.into()));
            },
            TopologyMessage::AnnouncementNode(v) => self.0.write().unwrap().run(v),
            TopologyMessage::AnnouncementChannel(v) => self.0.write().unwrap().run(v),
            TopologyMessage::UpdateChannel(v) => self.0.write().unwrap().run(v),
        };
        self.0.write().unwrap().world.maintain();

        ConsumingFuture::ok(self, sink)
    }
}
