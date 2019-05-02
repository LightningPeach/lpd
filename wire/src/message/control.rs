use std::mem;

use serde_derive::{Serialize, Deserialize};

use super::MessageSize;

/// Keep alive message. Two purposes:
/// allow for the existence of long-lived TCP connections,
/// obfuscate traffic pattern.
/// `pong_length` should be less or equal 2 ^ 16 - 5,
/// in order to the response message fit in the size limit.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Ping {
    pub pong_length: MessageSize,
    pub data: Vec<u8>,
}

impl Ping {
    pub fn new(length: MessageSize, pong_length: MessageSize) -> Result<Self, ()> {
        use rand::{Rng, thread_rng};
        Ping {
            pong_length: pong_length as _,
            data: (0..length).map(|_| thread_rng().gen()).collect(),
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
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Pong {
    pub data: Vec<u8>,
}

impl Pong {
    pub fn new(ping: &Ping) -> Self {
        use rand::{Rng, thread_rng};

        let length = ping.pong_length();
        Pong {
            data: (0..length).map(|_| thread_rng().gen()).collect(),
        }
    }

    pub fn length(&self) -> MessageSize {
        self.data.len() as _
    }
}


#[cfg(test)]
mod test {
    use binformat::BinarySD;
    use std::io::Cursor;
    use crate::Message;
    use pretty_assertions::assert_eq;
    use super::{Ping, Pong};

    #[test]
    fn ping_test() {
        let msg_hex = "0012000a000401020304";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = Ping {
            pong_length: 10,
            data: hex::decode("01020304").unwrap(),
        };
        let wrapped_msg_correct = Message::Ping(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }


    #[test]
    fn pong_test() {
        let msg_hex = "0013000201c8";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = Pong {
            data: hex::decode("01c8").unwrap(),
        };
        let wrapped_msg_correct = Message::Pong(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }


}