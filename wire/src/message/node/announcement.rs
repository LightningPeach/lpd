use std::error::Error;

use super::RawFeatureVector;
use super::NodeAlias;
use super::Color;
use super::super::types::{RawSignature, RawPublicKey};

use common_types::secp256k1_m::{Signed, Data};

use binformat::SerdeVec;
use binformat::PackSized;

use std::net::SocketAddr;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use serde_derive::{Serialize, Deserialize};

pub type AnnouncementNode = Signed<Data<AnnouncementNodeData>, RawSignature>;

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct AnnouncementNodeData {
    pub features: RawFeatureVector,
    pub timestamp: u32,
    pub node_id: RawPublicKey,
    pub color: Color,
    pub alias: NodeAlias,
    pub address: SerdeVec<Address>,
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct IpV4 {
    pub ip: [u8; 4],
    pub port: u16,
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct IpV6 {
    pub ip: [u8; 16],
    pub port: u16,
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct TorV2 {
    pub onion: [u8; 10],
    pub port: u16,
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct TorV3 {
    pub public_key: [u8; 32],
    pub checksum: u16,
    pub version: u8,
    pub port: u16,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Address {
    None,
    IpV4(IpV4),
    IpV6(IpV6),
    TorV2(TorV2),
    TorV3(TorV3),
}

// TODO(mkl): add to_str function for Address with Tor support
// Actually port is not part of IP address. It is added for convenience
impl Address {
    fn from_str(s: &str) -> Result<Address, Box<Error>> {
        use std::str::FromStr;
        let socket_addr = SocketAddr::from_str(s)
            .map_err(|err| format!("cannot parse Address: {:?}", err))?;
        Ok(Address::from(socket_addr))
    }
}

impl From<SocketAddr> for Address {
    fn from(v: SocketAddr) -> Self {
        use self::IpAddr::*;

        match v.ip() {
            V4(ip) => Address::IpV4(IpV4 {
                ip: ip.octets(),
                port: v.port(),
            }),
            V6(ip) => Address::IpV6(IpV6 {
                ip: ip.octets(),
                port: v.port(),
            }),
        }
    }
}

impl Address {
    pub fn into_socket_address(self) -> Result<SocketAddr, Self> {
        match self {
            Address::IpV4(ip) =>
                Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip.ip)), ip.port)),
            Address::IpV6(ip) =>
                Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::from(ip.ip)), ip.port)),
            _ => Err(self),
        }
    }

    pub fn localhost(port: u16) -> Self {
        Address::IpV4(IpV4 {
            ip: [127, 0, 0, 1],
            port: port,
        })
    }
}

impl PackSized for Address {
    const SIZE: usize = 0;

    fn pack_size(&self) -> usize {
        use std::mem;

        match self {
            &Address::IpV4(_) => 1 + mem::size_of::<IpV4>(),
            &Address::IpV6(_) => 1 + mem::size_of::<IpV6>(),
            &Address::TorV2(_) => 1 + mem::size_of::<TorV2>(),
            &Address::TorV3(_) => 1 + mem::size_of::<TorV3>(),
            _ => 1,
        }
    }
}

mod serde_m {
    use super::Address;

    use serde::ser;
    use serde::de;
    use serde::Serialize;
    use serde::Serializer;
    use serde::Deserialize;
    use serde::Deserializer;
    use std::fmt;

    impl<'de> Deserialize<'de> for Address {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            use self::de::SeqAccess;

            struct V;

            impl<'de> de::Visitor<'de> for V {
                type Value = Address;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "address descriptor according to spec")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de>, {
                    use self::Address::*;

                    let mut seq = seq;
                    let d: u8 = seq.next_element()?
                        .ok_or(de::Error::custom(format!("expecting discriminant")))?;
                    match d {
                        0 => seq.next_element()?.map(|_v: ()| None)
                            .ok_or(de::Error::custom(format!("expecting void"))),
                        1 => seq.next_element()?.map(|v| IpV4(v))
                            .ok_or(de::Error::custom(format!("expecting ipv4"))),
                        2 => seq.next_element()?.map(|v| IpV6(v))
                            .ok_or(de::Error::custom(format!("expecting ipv6"))),
                        3 => seq.next_element()?.map(|v| TorV2(v))
                            .ok_or(de::Error::custom(format!("expecting tor v2 address"))),
                        4 => seq.next_element()?.map(|v| TorV3(v))
                            .ok_or(de::Error::custom(format!("expecting tor v3 address"))),
                        _ => Err(de::Error::custom(format!("unknown address type"))),
                    }
                }

            }

            deserializer.deserialize_tuple(2, V)
        }
    }

    impl Serialize for Address {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
            use self::ser::SerializeTuple;
            use self::Address::*;

            let mut tuple = serializer.serialize_tuple(2)?;

            match self {
                &None => {
                    tuple.serialize_element(&0u8)?;
                    tuple.serialize_element(&())?;
                },
                &IpV4(ref v) => {
                    tuple.serialize_element(&1u8)?;
                    tuple.serialize_element(&v)?;
                },
                &IpV6(ref v) => {
                    tuple.serialize_element(&2u8)?;
                    tuple.serialize_element(&v)?;
                },
                &TorV2(ref v) => {
                    tuple.serialize_element(&3u8)?;
                    tuple.serialize_element(&v)?;
                },
                &TorV3(ref v) => {
                    tuple.serialize_element(&4u8)?;
                    tuple.serialize_element(&v)?;
                },
            };

            tuple.end()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use binformat::BinarySD;
    use crate::message::channel::ChannelId;
    use std::io::{Cursor, Read, Seek, SeekFrom};
    use crate::{Message, RevokeAndAck, RawPublicKey, CommitmentSigned, RawSignature};
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn announcement_node() {
        let v = vec![
            79, 96, 163, 97, 39, 255, 19, 117, 81, 200, 19, 41, 245, 33, 183, 23,
            28, 117, 14, 1, 141, 75, 90, 129, 255, 60, 26, 118, 68, 184, 134, 161,
            126, 127, 211, 53, 155, 124, 197, 63, 10, 126, 221, 18, 251, 234, 168, 147,
            99, 251, 20, 136, 113, 32, 227, 211, 223, 70, 153, 226, 149, 241, 132, 217,

            0, 0,

            91, 146, 153, 17,

            3,
            106, 249, 251, 226, 207, 167, 82, 189, 243, 143, 88, 203, 80, 85, 198, 149,
            217, 117, 60, 137, 4, 146, 41, 195, 185, 21, 143, 141, 208, 29, 135, 204,

            51, 153, 255,

            48, 51, 54, 97, 102, 57, 102, 98, 101, 50, 99, 102, 97, 55, 53, 50,
            98, 100, 102, 51, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,

            0, 0
        ];
        let t: AnnouncementNode = BinarySD::deserialize(&v[..]).unwrap();
        // TODO(mkl): add automatic check
        println!("{:?}", t);
    }

    #[test]
    fn announcement_node_test() {
        let msg_hex = "01010000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000759e23203639e0e2447aaf1fee79699ac59d6166544bf8583c42dbeabd35fc45e63c0831721140a01000000000000000000000000000000000000000000000000000000000000000028017f0000012710010909090904d2010a0a65152b67021fff00000a8885a3000000000000ac1f0b55";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = Signed {
            signature: RawSignature::from_hex("3022021d0100000000000000000000000000000000000000000000000000000000020100").unwrap(),
            data: Data (
                AnnouncementNodeData {
                    features: RawFeatureVector::from_hex("0000").unwrap(),
                    timestamp: 123331122,
                    node_id: RawPublicKey::from_hex("03639e0e2447aaf1fee79699ac59d6166544bf8583c42dbeabd35fc45e63c08317").unwrap(),
                    color: Color::from_u32(554961408),
                    alias: NodeAlias::from_hex("0100000000000000000000000000000000000000000000000000000000000000").unwrap(),
                    address: SerdeVec(vec![
                        Address::from_str("127.0.0.1:10000").unwrap(),
                        Address::from_str("9.9.9.9:1234").unwrap(),
                        Address::from_str("10.10.101.21:11111").unwrap(),
                        Address::from_str("[1fff:0:a88:85a3::ac1f]:2901").unwrap(),
                    ]),
                }
            )
        };
        let wrapped_msg_correct = Message::AnnouncementNode(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }
}
