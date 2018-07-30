use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use std::result::Result;
use std::io;

use bincode::Error;
use bincode::LengthSDOptions;
use bincode::config;

use super::message::MessageSize;

/// LengthSDOptions is the delegate that overrides
/// serialization/deserialization of the length of some sequence
#[derive(Copy, Clone)]
struct LengthSD;

impl LengthSDOptions for LengthSD {
    fn serialized_length_size(&self, length: u64) -> Result<usize, Error> {
        let _ = length;
        Ok(2)
    }

    fn serialize_length<S: Serializer>(&self, s: S, length: usize) -> Result<S::Ok, S::Error> {
        let length = length as MessageSize;
        Serialize::serialize(&length, s)
    }

    fn deserialize_length<'de, D: Deserializer<'de>>(&self, d: D) -> Result<usize, D::Error> {
        Deserialize::deserialize(d).map(|l: MessageSize| l as _)
    }

    fn serialized_variant_size(&self, variant: u32) -> Result<usize, Error> {
        let _ = variant;
        Ok(2)
    }

    fn serialize_variant<S: Serializer>(&self, s: S, variant: u32) -> Result<S::Ok, S::Error> {
        let variant = variant as u16;
        Serialize::serialize(&variant, s)
    }

    fn deserialize_variant<'de, D: Deserializer<'de>>(&self, d: D) -> Result<u32, D::Error> {
        Deserialize::deserialize(d).map(|variant: u16| variant as _)
    }
}

/// Public facade object, provides serde interface with the proper configuration applied
pub struct BinarySD;

impl BinarySD {
    pub fn serialize<T: Serialize, W: io::Write>(w: W, value: &T) -> Result<(), Error> {
        let mut temp = config();
        let bc_config = temp.big_endian();

        bc_config.serialize_custom_length_into(w, value, LengthSD)
    }

    pub fn deserialize<T: DeserializeOwned, R: io::Read>(r: R) -> Result<T, Error> {
        let mut temp = config();
        let bc_config = temp.big_endian();

        bc_config.deserialize_custom_length_from(r, LengthSD)
    }
}
