use std::path::Path;
use rocksdb::{DB as RocksDB, Error as DBError};
use serde::{Serialize, de::DeserializeOwned};
use wire::BinarySD;

use std::ops::{DerefMut, Deref};

pub trait DBKey: Serialize + DeserializeOwned {
}

pub trait DBValue: Serialize + DeserializeOwned {
    fn cf_name() -> &'static str;
}

pub struct DB(Option<RocksDB>);

impl Default for DB {
    fn default() -> Self {
        DB(None)
    }
}

impl Deref for DB {
    type Target = RocksDB;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("missing database")
    }
}

impl DerefMut for DB {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect("missing database")
    }
}

impl DB {
    pub fn new<P>(path: P) -> Result<Self, DBError>
    where
        P: AsRef<Path>,
    {
        Ok(DB(Some(RocksDB::open_default(path)?)))
    }

    pub fn register<V>(&mut self) -> Result<(), DBError>
    where
        V: DBValue,
    {
        use rocksdb::Options;

        self.create_cf(V::cf_name(), &Options::default()).map(|_| ())
    }

    pub fn get_all<K, V>(&self) -> Result<Vec<(K, V)>, DBError>
    where
        V: DBValue,
        K: DBKey,
    {
        use rocksdb::IteratorMode::Start;

        let cf = self.cf_handle(V::cf_name()).expect("call `register` first");
        Ok(self.iterator_cf(cf, Start)?.map(|(key, value)| {
            (
                BinarySD::deserialize(key.as_ref()).unwrap(),
                BinarySD::deserialize(value.as_ref()).unwrap(),
            )
        }).collect::<Vec<(K, V)>>())
    }

    pub fn get<K, V>(&self, key: &K) -> Result<Option<V>, DBError>
    where
        V: DBValue,
        K: DBKey,
    {
        let cf = self.cf_handle(V::cf_name()).expect("call `register` first");
        let mut key_bytes = Vec::new();
        BinarySD::serialize(&mut key_bytes, key).unwrap();
        Ok(self.get_cf(cf, key_bytes.as_ref())?.and_then(|a| {
            BinarySD::deserialize(a.as_ref()).ok()
        }))
    }

    pub fn put<K, V>(&self, key: &K, value: V) -> Result<(), DBError>
    where
        V: DBValue,
        K: DBKey,
    {
        let cf = self.cf_handle(V::cf_name()).expect("call `register` first");
        let mut key_bytes = Vec::new();
        BinarySD::serialize(&mut key_bytes, key).unwrap();
        let mut value_bytes = Vec::new();
        BinarySD::serialize(&mut value_bytes, &value).unwrap();
        self.put_cf(cf, key_bytes.as_ref(), value_bytes.as_ref())
    }
}

impl DBKey for usize {}
