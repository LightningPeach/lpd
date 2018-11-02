use super::*;

use tokio::codec::{Encoder, Decoder};
use bytes::BytesMut;
use wire::{BinarySD, Message, WireError};
use serde::{Serialize, de::DeserializeOwned};

impl Machine {
    pub fn write<T>(&mut self, item: T, dst: &mut BytesMut) -> Result<(), WireError>
    where
        T: Serialize,
    {
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

        let tag = self.send_cipher.as_mut().unwrap().encrypt(
            &[],
            &mut dst.writer(),
            &length_buffer[..]
        )?;
        dst.put_slice(&tag[..]);

        let tag = self.send_cipher.as_mut().unwrap().encrypt(
            &[],
            &mut dst.writer(),
            &self.message_buffer.borrow()[..length]
        )?;
        dst.put_slice(&tag[..]);

        Ok(())
    }

    pub fn read<T>(&mut self, src: &mut BytesMut) -> Result<Option<T>, WireError>
    where
        T: DeserializeOwned,
    {
        use chacha20_poly1305_aead::DecryptError;
        use serde::ser::Error;

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
                self.recv_cipher.as_mut().unwrap().decrypt(
                    &[],
                    &mut plain.as_mut(),
                    cipher.as_ref(),
                    tag
                ).map_err(|e| {
                    match e {
                        DecryptError::IoError(e) => WireError::from(e),
                        DecryptError::TagMismatch => WireError::custom("tag")
                    }
                })?;

                let length: u16 = BinarySD::deserialize(&plain[..])?;
                length as usize
            };

            if src.len() < length + MAC_SIZE {
                Ok(None)
            } else {
                let cipher = src.split_to(length);
                let tag = tag(src);

                self.recv_cipher.as_mut().unwrap().decrypt(
                    &[],
                    &mut self.message_buffer.borrow_mut().as_mut(),
                    cipher.as_ref(),
                    tag
                ).map_err(|e| {
                    match e {
                        DecryptError::IoError(e) => WireError::from(e),
                        DecryptError::TagMismatch => WireError::custom("tag")
                    }
                })?;

                BinarySD::deserialize(self.message_buffer.borrow().as_ref())
                    .map(Some)
            }
        }
    }
}

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
