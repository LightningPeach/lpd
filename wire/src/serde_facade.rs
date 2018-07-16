use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use std::result;
use std::io;

use bincode::Result;
use bincode::LengthSDOptions;
use bincode::config;

#[derive(Copy, Clone)]
struct LengthSD;

impl LengthSDOptions for LengthSD {
    fn serialized_length_size(&self, length: u64) -> Result<usize> {
        let _ = length;
        Ok(2)
    }

    fn serialize_length<S: Serializer>(&self, s: S, length: usize) -> result::Result<S::Ok, S::Error> {
        let length = length as u16;
        Serialize::serialize(&length, s)
    }

    fn deserialize_length<'de, D: Deserializer<'de>>(&self, d: D) -> result::Result<usize, D::Error> {
        Deserialize::deserialize(d).map(|l: u16| l as _)
    }
}

pub struct BinarySD;

impl BinarySD {
    pub fn serialize<T: Serialize, W: io::Write>(w: W, value: &T) -> Result<()> {
        let mut temp = config();
        let bc_config = temp.big_endian();

        bc_config.serialize_custom_length_into(w, value, LengthSD)
    }

    pub fn deserialize<T: DeserializeOwned, R: io::Read>(r: R) -> Result<T> {
        let mut temp = config();
        let bc_config = temp.big_endian();

        bc_config.deserialize_custom_length_from(r, LengthSD)
    }
}
