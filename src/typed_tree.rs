use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Deserialize, Serialize)]
pub struct KeyVal<K>
where
  K: AsRef<[u8]> + KeyOf,
  <<K as KeyOf>::T as std::convert::TryFrom<sled::IVec>>::Error: Into<sled::Error>,
{
  key: K,
  val: <K as KeyOf>::T,
}

pub trait KeyOf: AsRef<[u8]>
where
  <Self::T as std::convert::TryFrom<sled::IVec>>::Error: Into<sled::Error>,
{
  const NAME: &'static str;
  const PREFIX: u8;
  type T: ?Sized + TryFrom<sled::IVec> + Into<sled::IVec> + for<'a> Deserialize<'a> + Serialize;
}

pub trait TypedTree<K>
where
  K: AsRef<[u8]> + KeyOf,
  <<K as KeyOf>::T as std::convert::TryFrom<sled::IVec>>::Error: Into<sled::Error>,
  <K as KeyOf>::T: TryFrom<sled::IVec>,
{
  fn get_typed(&self, key: K) -> sled::Result<Option<<K as KeyOf>::T>>;
  fn insert_typed(&self, kv: KeyVal<K>) -> sled::Result<Option<sled::IVec>>;
}

impl<K> TypedTree<K> for sled::Db
where
  <<K as KeyOf>::T as std::convert::TryFrom<sled::IVec>>::Error: Into<sled::Error>,
  <K as KeyOf>::T: TryFrom<sled::IVec>,
  K: AsRef<[u8]> + KeyOf,
  sled::IVec: From<<K as KeyOf>::T>,
{
  fn get_typed(&self, key: K) -> sled::Result<Option<<K as KeyOf>::T>>
  where
    K: AsRef<[u8]>,
  {
    self.get(key).and_then(|v| match v {
      Some(v) => sled::Result::Ok(Some(<K as KeyOf>::T::try_from(v).map_err(|e| e.into())?)),
      None => sled::Result::Ok(None),
    })
  }

  fn insert_typed(&self, kv: KeyVal<K>) -> sled::Result<Option<sled::IVec>> {
    self.insert(kv.key, kv.val)
  }
}
