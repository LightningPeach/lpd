use std::path::Path;
use rocksdb::{DB as RocksDB, ColumnFamilyDescriptor};
use serde::{Serialize, de::DeserializeOwned};

use std::ops::{DerefMut, Deref};

pub use rocksdb::Error as DBError;

pub trait DBKey: Serialize + DeserializeOwned {
}

// It is easy to derive such trait, just add `_extension: Option<E>` where `E` is also DBValue
pub trait DBValue: Serialize + DeserializeOwned {
    // Extension is also further DBValue
    type Extension: DBValue;

    fn extend(self, e: Self::Extension) -> Self;
    fn cf_name() -> &'static str;
}

#[derive(Default)]
pub struct DBBuilder {
    cfs: Vec<ColumnFamilyDescriptor>,
}

impl DBBuilder {
    pub fn register<V>(self) -> Self
    where
        V: DBValue,
    {
        let mut s = self;
        s.cfs.push(ColumnFamilyDescriptor::new(V::cf_name(), Default::default()));
        s
    }

    pub fn build<P>(self, path: P) -> Result<DB, DBError>
    where
        P: AsRef<Path>,
    {
        use rocksdb::Options;

        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);
        Ok(DB(RocksDB::open_cf_descriptors(&options, path, self.cfs)?))
    }
}

pub struct DB(RocksDB);

impl Deref for DB {
    type Target = RocksDB;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for DB {
    fn default() -> Self {
        panic!()
    }
}

impl DerefMut for DB {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DB {
    pub fn get_all<K, V>(&self) -> Result<Vec<(K, V)>, DBError>
    where
        V: DBValue,
        K: DBKey,
    {
        use rocksdb::IteratorMode::Start;
        use binformat::BinarySD;

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
        use binformat::BinarySD;

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
        use binformat::BinarySD;

        let cf = self.cf_handle(V::cf_name()).expect("call `register` first");
        let mut key_bytes = Vec::new();
        BinarySD::serialize(&mut key_bytes, key).unwrap();
        let mut value_bytes = Vec::new();
        BinarySD::serialize(&mut value_bytes, &value).unwrap();
        self.put_cf(cf, key_bytes.as_ref(), value_bytes.as_ref())
    }

}

// The basis of the extension chain
impl DBValue for () {
    type Extension = ();

    fn extend(self, e: Self::Extension) -> Self {
        e
    }

    fn cf_name() -> &'static str {
        "()"
    }
}

impl DBKey for usize {}

impl DBKey for String {}
