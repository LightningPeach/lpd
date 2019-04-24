use super::handshake::Machine;

use tokio::codec::{Encoder, Decoder};
use bytes::BytesMut;
use binformat::WireError;
use wire::{Message, MessageExt};

impl Encoder for Machine {
    type Item = MessageExt;
    type Error = WireError;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.write(item.message, item.extra_data, dst)
    }
}

impl Decoder for Machine {
    type Item = MessageExt;
    type Error = WireError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.read(src)
            .map(|v|
                v.map(|(message, extra_data)| MessageExt {
                    message: message,
                    extra_data: extra_data,
                })
            )
    }
}
