use wire::{
    Hash256, ShortChannelId, PublicKey, MilliSatoshi, ChannelUpdateFlags, RawFeatureVector,
    AnnouncementChannel, DataToSign, UpdateChannel,
};

use specs::prelude::*;

use rocksdb::Error as DBError;
use state::{DB, DBValue};
use dijkstras_search::Edge;

use super::tools::GenericSystem;
use super::node::{NodeRef, Node, NodeLinks};

use serde_derive::{Serialize, Deserialize};

#[derive(Component, Eq, PartialEq)]
pub struct Peer {
    id: PublicKey,
}

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Blacklisted;

#[derive(Clone, Debug)]
pub struct ChannelRef(pub Entity);

#[derive(Component, Default)]
pub struct ChannelLinks(pub Option<NodeRef>, pub Option<NodeRef>);

#[derive(Component, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelId {
    hash: Hash256,
    short_channel_id: ShortChannelId,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct ChannelParties {
    lightning: (PublicKey, PublicKey),
    origin: (PublicKey, PublicKey),
}

pub enum Side {
    Left {
        other: PublicKey,
    },
    Right {
        other: PublicKey,
    },
}

impl ChannelParties {
    pub fn other(&self, id: &PublicKey) -> Option<Side> {
        match (self.lightning.0.eq(id), self.lightning.1.eq(id)) {
            (true, _) => Some(Side::Left { other: self.lightning.1.clone() }),
            (_, true) => Some(Side::Right { other: self.lightning.0.clone() }),
            _ => None,
        }
    }
}

#[derive(Component, Clone, Default, Serialize, Deserialize)]
pub struct ChannelHistory {
    records: Vec<ChannelPolicy>,
}

impl ChannelHistory {
    pub fn current(&self) -> Option<&ChannelPolicy> {
        self.records.last()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPolicy {
    timestamp: u32,
    flags: ChannelUpdateFlags,
    time_lock_delta: u16,
    htlc_minimum: MilliSatoshi,
    base_fee: u32,
    fee_rate: u32,
}

impl Edge for ChannelRef {
    type Cost = u32;
    type Context = World;

    fn cost(&self, context: &World) -> Self::Cost {
        // find the history in the world, fetch the last record, return max int if nothing
        let max_int = 0xffffffff_u32;
        context.read_storage::<ChannelHistory>().get(self.0.clone())
            .map(|history| {
                history.current()
                    .map(|policy| policy.fee_rate.clone())
                    .unwrap_or(max_int)
            })
            .unwrap_or(max_int)
    }
}

// TODO: add subsystem to poll if founding output is still there
// TODO: add rebroadcasting subsystem
impl<'a> System<'a> for GenericSystem<AnnouncementChannel, ()> {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        Read<'a, RawFeatureVector>,
        WriteStorage<'a, Peer>,
        WriteStorage<'a, Blacklisted>,
        WriteStorage<'a, ChannelId>,
        WriteStorage<'a, ChannelParties>,
        ReadStorage<'a, Node>,
        WriteStorage<'a, NodeLinks>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        use specs::Join;

        self.run_func(|announcement_channel| {
            let (
                entities,
                update,
                features,
                mut peer,
                blacklist_mark,
                channel_id,
                mut channel_parties,
                node,
                mut node_links
            ) = (&*data.0, &*data.1, &*data.2, data.3, &mut data.4, &mut data.5, data.6, data.7, data.8);

            if let Err(()) = announcement_channel.check_features(features) {
                return;
            }

            let announcement_channel = match announcement_channel.check_signatures() {
                Err(()) => {
                    // TODO: fail the connection
                    return;
                },
                Ok(s) => s,
            };

            // TODO: check channel id, check if chain hash known

            // check if nodes is not blacklisted
            for (_, peer) in (blacklist_mark, &peer).join() {
                let (ref left, ref right) = announcement_channel.node_id;
                if left.eq(&peer.id) || right.eq(&peer.id) {
                    // fail the connection
                    return;
                }
            }

            let id = ChannelId {
                hash: announcement_channel.hash().clone(),
                short_channel_id: announcement_channel.id().clone(),
            };

            let this_parties = ChannelParties {
                lightning: announcement_channel.node_id.clone(),
                origin: announcement_channel.bitcoin_key.clone(),
            };

            // check if nodes should be blacklisted
            let mut shell_blacklist_channel_parties = None;
            for (channel_parties, channel_id) in (&mut channel_parties, channel_id).join() {
                if (&*channel_id).eq(&id) && !channel_parties.lightning.eq(&this_parties.lightning) {
                    shell_blacklist_channel_parties = Some(channel_parties.clone());
                    break;
                }
            }

            if let Some(shell_blacklist_channel_parties) = shell_blacklist_channel_parties {
                let shell_blacklist_id = |id: &PublicKey| -> bool {
                    false
                        || id.eq(&shell_blacklist_channel_parties.lightning.0)
                        || id.eq(&shell_blacklist_channel_parties.lightning.1)
                        || id.eq(&this_parties.lightning.0)
                        || id.eq(&this_parties.lightning.1)
                };
                let shell_blacklist_peer = |peer: &Peer| shell_blacklist_id(&peer.id);
                let shell_blacklist_channel = |parties: &ChannelParties| -> bool {
                    false
                        || shell_blacklist_id(&parties.lightning.0)
                        || shell_blacklist_id(&parties.lightning.1)
                };

                for (entity, peer) in (entities, &mut peer).join() {
                    if shell_blacklist_peer(peer) {
                        update.insert(entity, Blacklisted);
                    }
                }

                for (entity, channel_parties) in (entities, &mut channel_parties).join() {
                    if shell_blacklist_channel(channel_parties) {
                        update.remove::<ChannelParties>(entity);
                        update.remove::<ChannelId>(entity);
                        update.remove::<ChannelHistory>(entity);
                        update.remove::<ChannelLinks>(entity);
                    }
                }

                // fail the connection
                return;
            }

            let channel_ref = entities.create();

            // try link
            let mut links = ChannelLinks::default();
            for (entity, node, mut node_links) in (entities, &node, &mut node_links).join() {
                if node.id() == this_parties.lightning.0.clone() {
                    links.0 = Some(NodeRef(entity));
                }
                if node.id() == this_parties.lightning.1.clone() {
                    links.1 = Some(NodeRef(entity));
                }
                node_links.0.push(ChannelRef(channel_ref));
            }

            update.insert(channel_ref, links);
            update.insert(channel_ref, id);
            update.insert(channel_ref, this_parties);
            update.insert(channel_ref, ChannelHistory::default());
        });
    }
}

impl<'a> System<'a> for GenericSystem<UpdateChannel, ()> {
    type SystemData = (
        ReadStorage<'a, ChannelId>,
        ReadStorage<'a, ChannelParties>,
        WriteStorage<'a, ChannelHistory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;

        self.run_func(|update_channel| {
            let (
                channel_id,
                channel_parties,
                mut channel_history,
            ) = data;


            for (id, parties, history) in (&channel_id, &channel_parties, &mut channel_history).join() {

                if update_channel.as_ref_data().id().eq(&id.short_channel_id)
                    && update_channel.as_ref_data().hash().eq(&id.hash) {
                    let update_channel = match update_channel.verify_any_of_two(&parties.lightning) {
                        Ok(d) => d.0,
                        Err(_) => break,
                    };
                    history.records.push(ChannelPolicy {
                        timestamp: update_channel.timestamp,
                        flags: update_channel.flags,
                        time_lock_delta: update_channel.time_lock_delta,
                        htlc_minimum: update_channel.htlc_minimum,
                        base_fee: update_channel.base_fee,
                        fee_rate: update_channel.fee_rate,
                    });

                    break;
                }
            }
        });
    }
}

#[derive(Debug)]
pub struct LogChannels;

impl<'a> System<'a> for GenericSystem<LogChannels, ()> {
    type SystemData = (
        ReadStorage<'a, ChannelId>,
        ReadStorage<'a, ChannelParties>,
        ReadStorage<'a, ChannelHistory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;

        self.run_func(|_| {
            println!("channels: ");
            for (id, parties, history) in (&data.0, &data.1, &data.2).join() {
                let space = "    ";
                println!("{} {:?}", space, id.short_channel_id);
                println!("{} {:?}", space, parties);
                println!("{} {:?}", space, history.records.last());
            }
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChannelInfo(pub ChannelId, pub ChannelParties, pub ChannelHistory);

impl DBValue for ChannelInfo {
    type Extension = ();

    fn extend(self, e: Self::Extension) -> Self {
        let _ = e;
        self
    }

    fn cf_name() -> &'static str {
        "channel"
    }
}

#[derive(Debug)]
pub struct LoadChannels;

impl<'a> System<'a> for GenericSystem<LoadChannels, Result<(), DBError>> {
    type SystemData = (
        Read<'a, DB>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            db,
            entities,
            update,
        ) = (&*data.0, &*data.1, &*data.2);

        self.run_func(|_| {
            for (_, ChannelInfo(id, parties, history)) in db.get_all::<usize, ChannelInfo>()?.into_iter() {
                let channel_ref = entities.create();
                update.insert(channel_ref, id);
                update.insert(channel_ref, parties);
                update.insert(channel_ref, history);
            }
            Ok(())
        })
    }
}

#[derive(Debug)]
pub struct StoreChannels;

impl<'a> System<'a> for GenericSystem<StoreChannels, Result<(), DBError>> {
    type SystemData = (
        Write<'a, DB>,
        ReadStorage<'a, ChannelId>,
        ReadStorage<'a, ChannelParties>,
        ReadStorage<'a, ChannelHistory>,
    );

    fn run(&mut self, mut data: Self::SystemData) {

        self.run_func(|_| {
            let db = &mut *data.0;
            let mut i = 0usize;
            for (id, parties, history) in (&data.1, &data.2, &data.3).join() {
                db.put(&i, ChannelInfo(id.clone(), parties.clone(), history.clone()))?;
                i = i + 1;
            }
            Ok(())
        })
    }
}

#[cfg(feature = "rpc")]
mod rpc {
    use interface::{routing::{ChannelEdge, RoutingPolicy}, common::MilliSatoshi};
    use binformat::BinarySD;
    use super::{ChannelPolicy, ChannelInfo};

    impl From<ChannelPolicy> for RoutingPolicy {
        fn from(v: ChannelPolicy) -> Self {
            let mut r = RoutingPolicy::new();
            r.set_time_lock_delta(v.time_lock_delta as _);
            r.set_min_htlc(u64::from(v.htlc_minimum) as _);
            r.set_fee_base_msat(v.base_fee as _);
            let mut fee = MilliSatoshi::new();
            fee.set_value(v.fee_rate as _);
            r.set_fee_rate_milli(fee);
            r.set_disabled(false);
            r
        }
    }

    impl From<ChannelInfo> for ChannelEdge {
        fn from(v: ChannelInfo) -> Self {
            use std::mem;

            let ChannelInfo(id, parties, policy) = v;
            let mut r = ChannelEdge::new();

            let mut buffer = [0u8; mem::size_of::<u64>()];
            BinarySD::serialize(&mut buffer[..], &id.short_channel_id).unwrap();
            r.set_channel_id(buffer.iter().fold(0, |v, &b| (v | (b as u64)) << 8));
            r.set_chan_point(id.hash.to_string());

            //r.set_capacity(??);

            if let Some(policy) = policy.current() {
                r.set_last_update(policy.timestamp as _);
                let policy: RoutingPolicy = policy.clone().into();
                r.set_node1_pub(parties.lightning.0.to_string());
                r.set_node2_pub(parties.lightning.1.to_string());
                r.set_node1_policy(policy.clone());
                r.set_node2_policy(policy);
            }

            r
        }
    }
}
