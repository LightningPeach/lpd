use super::route::HmacData;
use wire::{Satoshi, ShortChannelId};
use secp256k1::PublicKey;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde_derive::{Serialize, Deserialize};
use chacha::{ChaCha, KeyStream};
use std::ops::BitXorAssign;

#[derive(Debug, Eq, PartialEq)]
pub struct Hop {
    id: PublicKey,
    data: HopData,
}

impl Hop {
    /// Dummy constructor
    pub fn new(id: PublicKey, data: HopData) -> Self {
        Hop { id: id, data: data }
    }

    pub fn id(&self) -> &PublicKey {
        &self.id
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HopDataRealm {
    Bitcoin = 0,
}

impl From<u8> for HopDataRealm {
    fn from(v: u8) -> Self {
        use self::HopDataRealm::*;

        match v {
            0 => Bitcoin,
            _ => panic!("unknown hop realm"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct HopData {
    realm: HopDataRealm,
    next_address: ShortChannelId,
    forward_amount: Satoshi,
    // TODO: create type for the value
    outgoing_cltv: u32,
}

impl HopData {
    const PAD_SIZE: usize = 12;
    pub const SIZE: usize = 33;

    /// Dummy constructor
    pub fn new(
        realm: HopDataRealm,
        next_address: ShortChannelId,
        forward_amount: Satoshi,
        outgoing_cltv: u32,
    ) -> Self {
        HopData {
            realm: realm,
            next_address: next_address,
            forward_amount: forward_amount,
            outgoing_cltv: outgoing_cltv,
        }
    }
}

// we could not derive such implementation because padding
impl Serialize for HopData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut tuple = serializer.serialize_tuple(5)?;
        tuple.serialize_element(&(self.realm as u8))?;
        tuple.serialize_element(&self.next_address)?;
        tuple.serialize_element(&self.forward_amount)?;
        tuple.serialize_element(&self.outgoing_cltv)?;
        tuple.serialize_element(&[0u8; Self::PAD_SIZE])?;
        tuple.end()
    }
}

// we could not derive such implementation because padding
impl<'de> Deserialize<'de> for HopData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{Visitor, SeqAccess, Error};
        use std::fmt;

        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = HopData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "HopData {} bytes", HopData::SIZE)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let realm: u8 = seq
                    .next_element()?
                    .ok_or(Error::custom("expecting header byte, 0 for bitcoin"))?;
                let next_address = seq
                    .next_element()?
                    .ok_or(Error::custom("expecting addess"))?;
                let forward_amount = seq
                    .next_element()?
                    .ok_or(Error::custom("expecting satoshi amount"))?;
                let outgoing_cltv = seq.next_element()?.ok_or(Error::custom("expecting cltv"))?;
                let _: [u8; HopData::PAD_SIZE] = seq.next_element()?.ok_or(Error::custom(
                    format!("expecting padding {} bytes", HopData::PAD_SIZE),
                ))?;

                Ok(HopData {
                    realm: realm.into(),
                    next_address: next_address,
                    forward_amount: forward_amount,
                    outgoing_cltv: outgoing_cltv,
                })
            }
        }

        deserializer.deserialize_tuple(5, V)
    }
}

#[derive(Copy, Clone)]
pub struct HopBytes {
    data: [u8; HopData::SIZE],
    hmac: HmacData,
}

impl HopBytes {
    pub const SIZE: usize = HopData::SIZE + HmacData::SIZE;

    /// let's call it `zero` instead of `default` due to its semantic,
    /// such instance is not fully valid,
    /// it used only to generate an obfuscating padding
    pub fn zero() -> Self {
        HopBytes {
            data: [0; HopData::SIZE],
            hmac: HmacData::default(),
        }
    }

    pub fn new(hop: Hop, hmac: HmacData) -> Self {
        use wire::BinarySD;

        let mut r = HopBytes {
            data: [0; HopData::SIZE],
            hmac: hmac,
        };
        BinarySD::serialize(&mut r.data[..], &hop.data).unwrap();
        r
    }
}

impl Serialize for HopBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut tuple = serializer.serialize_tuple(3)?;
        tuple.serialize_element(&self.data[0])?;
        tuple.serialize_element(&self.data[1..])?;
        tuple.serialize_element(&self.hmac)?;
        tuple.end()
    }
}

impl<'a> BitXorAssign<&'a mut ChaCha> for HopBytes {
    fn bitxor_assign(&mut self, rhs: &'a mut ChaCha) {
        rhs.xor_read(&mut self.data[..]).unwrap();
        self.hmac ^= rhs;
    }
}
