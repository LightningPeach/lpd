use super::*;

use tokio::codec::{Encoder, Decoder};
use bytes::BytesMut;
use wire::{BinarySD, Message, WireError};

impl Encoder for Box<Machine> {
    type Item = Message;
    type Error = WireError;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        use bytes::BufMut;

        let length = {
            let mut buffer = self.message_buffer.borrow_mut();
            let mut cursor = io::Cursor::new(buffer.as_mut());
            BinarySD::serialize(&mut cursor, &item)?;
            cursor.position() as usize
        };
        let mut length_buffer = [0; LENGTH_HEADER_SIZE];
        BinarySD::serialize(&mut length_buffer.as_mut(), &(length as u16))?;

        dst.reserve(length + LENGTH_HEADER_SIZE + MAC_SIZE * 2);

        let tag = self.send_cipher.borrow_mut().encrypt(
            &[],
            &mut dst.writer(),
            &length_buffer[..]
        )?;
        dst.put_slice(&tag[..]);

        let tag = self.send_cipher.borrow_mut().encrypt(
            &[],
            &mut dst.writer(),
            &self.message_buffer.borrow()[..length]
        )?;
        dst.put_slice(&tag[..]);

        Ok(())
    }
}

impl Decoder for Box<Machine> {
    type Item = Message;
    type Error = WireError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < LENGTH_HEADER_SIZE + MAC_SIZE {
            Ok(None)
        } else {
            let tag = |src: &mut BytesMut| {
                let tag_bytes = src.split_to(MAC_SIZE);
                let mut tag = [0; MAC_SIZE];
                tag.copy_from_slice(tag_bytes.as_ref());
                tag
            };

            let length = {
                let cipher = src.split_to(LENGTH_HEADER_SIZE);
                let tag = tag(src);

                let mut plain = [0; LENGTH_HEADER_SIZE];
                self.recv_cipher.borrow_mut().decrypt(
                    &[],
                    &mut plain.as_mut(),
                    cipher.as_ref(),
                    tag
                ).map_err(WireError::from)?;

                let length: u16 = BinarySD::deserialize(&plain[..])?;
                length as usize
            };

            if src.len() < length + MAC_SIZE {
                Ok(None)
            } else {
                let cipher = src.split_to(length);
                let tag = tag(src);

                self.recv_cipher.borrow_mut().decrypt(
                    &[],
                    &mut self.message_buffer.borrow_mut().as_mut(),
                    cipher.as_ref(),
                    tag
                ).map_err(WireError::from)?;

                BinarySD::deserialize(self.message_buffer.borrow().as_ref())
                    .map(Some)
            }
        }
    }
}
