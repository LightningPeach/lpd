use wire::Hash256;
use wire::ShortChannelId;
use wire::PublicKey;
use wire::MilliSatoshi;
use wire::ChannelUpdateFlags;
use wire::RawFeatureVector;

use wire::AnnouncementChannel;
use wire::UpdateChannel;

use specs::DenseVecStorage;
use specs::NullStorage;
use specs::System;
use specs::Entities;
use specs::Read;
use specs::ReadStorage;
use specs::WriteStorage;
use specs::LazyUpdate;

#[derive(Component, Eq, PartialEq)]
pub struct Peer {
    id: PublicKey,
}

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Blacklisted;

#[derive(Component, Eq, PartialEq)]
pub struct ChannelId {
    hash: Hash256,
    short_channel_id: ShortChannelId,
}

#[derive(Component, Clone, Debug)]
pub struct ChannelParties {
    lightning: (PublicKey, PublicKey),
    origin: (PublicKey, PublicKey),
}

#[derive(Component, Default)]
pub struct ChannelHistory {
    records: Vec<ChannelPolicy>,
}

#[derive(Debug)]
pub struct ChannelPolicy {
    timestamp: u32,
    flags: ChannelUpdateFlags,
    time_lock_delta: u16,
    htlc_minimum: MilliSatoshi,
    base_fee: u32,
    fee_rate: u32,
}

// TODO: add subsystem to poll if founding output is still there
// TODO: add rebroadcasting subsystem
#[derive(Debug)]
pub struct AnnouncementChannelSystem {
    data: Option<AnnouncementChannel>,
}

impl From<AnnouncementChannel> for AnnouncementChannelSystem {
    fn from(v: AnnouncementChannel) -> Self {
        AnnouncementChannelSystem {
            data: Some(v),
        }
    }
}

impl AnnouncementChannelSystem {
    pub fn consume(&mut self) -> Option<AnnouncementChannel> {
        use std::mem;

        let mut temp = None;
        mem::swap(&mut temp, &mut self.data);
        if temp.is_none() {
            println!("{:?} should not be used twice, ignoring", self);
        }
        temp
    }
}

impl<'a> System<'a> for AnnouncementChannelSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        Read<'a, RawFeatureVector>,
        WriteStorage<'a, Peer>,
        WriteStorage<'a, Blacklisted>,
        WriteStorage<'a, ChannelId>,
        WriteStorage<'a, ChannelParties>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        use specs::Join;

        self.consume().map(|announcement_channel| {
            let (
                entities,
                update,
                features,
                mut peer,
                blacklist_mark,
                channel_id,
                mut channel_parties,
            ) = (&*data.0, &*data.1, &*data.2, data.3, &mut data.4, &mut data.5, data.6, );

            if let Err(()) = announcement_channel.check_features(features) {
                return;
            }

            if let Err(()) = announcement_channel.check_signatures() {
                // TODO: fail the connection
                return;
            }

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
                    }
                }

                // fail the connection
                return;
            }

            let channel_ref = entities.create();
            update.insert(channel_ref, id);
            update.insert(channel_ref, this_parties);
            update.insert(channel_ref, ChannelHistory::default());
        });
    }
}

#[derive(Debug)]
pub struct UpdateChannelSystem {
    data: Option<UpdateChannel>,
}

impl From<UpdateChannel> for UpdateChannelSystem {
    fn from(v: UpdateChannel) -> Self {
        UpdateChannelSystem {
            data: Some(v),
        }
    }
}

impl UpdateChannelSystem {
    pub fn consume(&mut self) -> Option<UpdateChannel> {
        use std::mem;

        let mut temp = None;
        mem::swap(&mut temp, &mut self.data);
        if temp.is_none() {
            println!("{:?} should not be used twice, ignoring", self);
        }
        temp
    }
}

impl<'a> System<'a> for UpdateChannelSystem {
    type SystemData = (
        ReadStorage<'a, ChannelId>,
        WriteStorage<'a, ChannelHistory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;

        self.consume().map(|update_channel| {
            let (
                channel_id,
                mut channel_history,
            ) = data;

            for (id, history) in (&channel_id, &mut channel_history).join() {
                if update_channel.value.id().eq(&id.short_channel_id)
                    && update_channel.value.hash().eq(&id.hash) {
                    history.records.push(ChannelPolicy {
                        timestamp: update_channel.value.timestamp,
                        flags: update_channel.value.flags,
                        time_lock_delta: update_channel.value.time_lock_delta,
                        htlc_minimum: update_channel.value.htlc_minimum,
                        base_fee: update_channel.value.base_fee,
                        fee_rate: update_channel.value.fee_rate,
                    });

                    break;
                }
            }
        });
    }
}

pub struct LogChannelsSystem;

impl<'a> System<'a> for LogChannelsSystem {
    type SystemData = (
        ReadStorage<'a, ChannelId>,
        ReadStorage<'a, ChannelParties>,
        ReadStorage<'a, ChannelHistory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;

        println!("channels: ");
        for (id, parties, history) in (&data.0, &data.1, &data.2).join() {
            let space = "    ";
            println!("{} {:?}", space, id.short_channel_id);
            println!("{} {:?}", space, parties);
            println!("{} {:?}", space, history.records.last());
        }
    }
}
