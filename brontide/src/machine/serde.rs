use super::*;

use tokio::codec::{Encoder, Decoder};
use bytes::BytesMut;
use wire::{Message, WireError};

impl Encoder for Box<Machine> {
    type Item = Message;
    type Error = WireError;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.write(item, dst)
    }
}

impl Decoder for Box<Machine> {
    type Item = Message;
    type Error = WireError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.read(src)
    }
}
