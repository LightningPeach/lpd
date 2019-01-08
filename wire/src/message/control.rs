use std::mem;

use serde_derive::{Serialize, Deserialize};

use rand::thread_rng;
use rand::Rng;
use rand::distributions::Standard;

use super::MessageSize;

/// Keep alive message. Two purposes:
/// allow for the existence of long-lived TCP connections,
/// obfuscate traffic pattern.
/// `pong_length` should be less or equal 2 ^ 16 - 5,
/// in order to the response message fit in the size limit.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Ping {
    pong_length: MessageSize,
    data: Vec<u8>,
}

impl Ping {
    pub fn new(length: MessageSize, pong_length: MessageSize) -> Result<Self, ()> {
        Ping {
            pong_length: pong_length as _,
            data: thread_rng().sample_iter(&Standard).take(length as usize).collect(),
        }.validate()
    }

    pub fn validate(self) -> Result<Self, ()> {
        use std::u16;

        // 16-bit runtime type information and 16-bit actual size of the pong
        type PongEmbellishment = (MessageSize, MessageSize);
        // the `Ping` structure has one more field
        type PingEmbellishment = (MessageSize, MessageSize, MessageSize);

        let pong_limit =
            self.pong_length() + (mem::size_of::<PongEmbellishment>() as MessageSize) <= u16::MAX;
        let ping_limit =
            self.length() + (mem::size_of::<PingEmbellishment>() as MessageSize) <= u16::MAX;

        if pong_limit && ping_limit {
            Ok(self)
        } else {
            Err(())
        }
    }

    pub fn length(&self) -> MessageSize {
        self.data.len() as _
    }

    pub fn pong_length(&self) -> MessageSize {
        self.pong_length
    }
}

/// The response for the `Ping` message,
/// the length of the data should correspond to received `Ping`.
/// Should ignore the `Ping` message if required length
/// cause the whole message exceed 2 ^ 16 - 1 size limit.
/// Should fail the channel if
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Pong {
    data: Vec<u8>,
}

impl Pong {
    pub fn new(ping: &Ping) -> Self {
        let length = ping.pong_length();
        Pong {
            data: thread_rng().sample_iter(&Standard).take(length as usize).collect(),
        }
    }

    pub fn length(&self) -> MessageSize {
        self.data.len() as _
    }
}
