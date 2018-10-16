#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

// The crate is bunch of public modules and tests module.
// Its structure will change.

extern crate secp256k1;
extern crate chrono;
#[macro_use]
extern crate wire;
extern crate brontide;
extern crate bitcoin_types;
extern crate common_types;
extern crate rand;
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate shred;
#[macro_use]
extern crate shred_derive;
extern crate rayon;

#[cfg(test)]
#[macro_use]
extern crate hex_literal;

#[cfg(test)]
extern crate hex;

extern crate tokio;

//pub mod tcp_connection;

//pub mod discovery;
//pub mod topology;
//pub mod synchronization;
//pub mod peer;
mod graph;

use wire::{
    Message, AnnouncementNode, AnnouncementChannel, UpdateChannel,
    MessageFiltered, MessageConsumer, WireError
};

use tokio::prelude::{Future, Sink};

pub use self::graph::Graph;

pub enum TopologyMessage {
    AnnouncementNode(AnnouncementNode),
    AnnouncementChannel(AnnouncementChannel),
    UpdateChannel(UpdateChannel),
}

impl MessageFiltered for TopologyMessage {
    fn filter(v: Message) -> Result<Self, Message> {
        match v {
            Message::AnnouncementNode(v) => Ok(TopologyMessage::AnnouncementNode(v)),
            Message::AnnouncementChannel(v) => Ok(TopologyMessage::AnnouncementChannel(v)),
            Message::UpdateChannel(v) => Ok(TopologyMessage::UpdateChannel(v)),
            v @ _ => Err(v),
        }
    }
}

impl MessageConsumer for Graph {
    type Message = TopologyMessage;

    fn consume<S>(mut self, sink: S, message: Self::Message) -> Box<dyn Future<Item=(Self, S), Error=WireError>>
    where
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        use tokio::prelude::IntoFuture;

        self.message(message);
        Box::new(Ok((self, sink)).into_future())
    }
}
