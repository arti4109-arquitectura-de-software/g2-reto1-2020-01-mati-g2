use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Deserialize, Serialize)]
pub struct KeyVal<K>
where
    K: AsRef<[u8]> + KeyOf,
    <<K as KeyOf>::T as std::convert::TryFrom<sled::IVec>>::Error: Into<sled::Error>,
{
    pub key: K,
    pub val: <K as KeyOf>::T,
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
    sled::IVec: From<<K as KeyOf>::T>,
{
    fn get_typed(&self, key: &K) -> sled::Result<Option<<K as KeyOf>::T>>;
    fn insert_typed(&self, key: &K, value: <K as KeyOf>::T) -> sled::Result<Option<sled::IVec>>;
}

pub trait MonotonicTypedTree<K>: TypedTree<K>
where
    K: AsRef<[u8]> + KeyOf + From<u64>,
    <<K as KeyOf>::T as std::convert::TryFrom<sled::IVec>>::Error: Into<sled::Error>,
    <K as KeyOf>::T: TryFrom<sled::IVec>,
    sled::IVec: From<<K as KeyOf>::T>,
{
    fn insert_monotonic(&self, value: <K as KeyOf>::T) -> sled::Result<(K, Option<sled::IVec>)>;
    fn get_max_key(&mut self) -> sled::Result<Arc<AtomicU64>>;
    fn insert_monotonic_atomic(
        &self,
        atomic: &Arc<AtomicU64>,
        value: <K as KeyOf>::T,
    ) -> sled::Result<(K, Option<sled::IVec>)> {
        let key = K::from(atomic.fetch_add(1, Ordering::SeqCst));
        self.insert_typed(&key, value).and_then(|v| Ok((key, v)))
    }
}

use std::convert::TryInto;

fn read_be_u64(input: &mut &[u8]) -> u64 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u64>());
    *input = rest;
    u64::from_be_bytes(int_bytes.try_into().unwrap())
}

impl<K> MonotonicTypedTree<K> for sled::Db
where
    K: AsRef<[u8]> + KeyOf + From<u64>,
    <<K as KeyOf>::T as std::convert::TryFrom<sled::IVec>>::Error: Into<sled::Error>,
    <K as KeyOf>::T: TryFrom<sled::IVec>,
    sled::IVec: From<<K as KeyOf>::T>,
{
    fn insert_monotonic(&self, value: <K as KeyOf>::T) -> sled::Result<(K, Option<sled::IVec>)> {
        let key = K::from(self.generate_id()?);
        self.insert(&key, value).and_then(|v| Ok((key, v)))
    }
    fn get_max_key(&mut self) -> sled::Result<Arc<AtomicU64>> {
        if let Some((k, v)) = self.pop_max()? {
            let count = {
                let mut b = k.as_ref();
                if b.len() == 8 {
                    read_be_u64(&mut b)
                } else {
                    panic!()
                }
            };
            self.insert(
                k.clone(),
                <K as KeyOf>::T::try_from(v).map_err(|e| e.into())?,
            )?;

            Ok(Arc::new(AtomicU64::new(count + 1)))
        } else {
            Ok(Arc::new(AtomicU64::new(1)))
        }
    }
}

impl<K> TypedTree<K> for sled::Db
where
    <<K as KeyOf>::T as std::convert::TryFrom<sled::IVec>>::Error: Into<sled::Error>,
    <K as KeyOf>::T: TryFrom<sled::IVec>,
    K: AsRef<[u8]> + KeyOf,
    sled::IVec: From<<K as KeyOf>::T>,
{
    fn get_typed(&self, key: &K) -> sled::Result<Option<<K as KeyOf>::T>>
    where
        K: AsRef<[u8]>,
    {
        self.get(key).and_then(|v| match v {
            Some(v) => sled::Result::Ok(Some(<K as KeyOf>::T::try_from(v).map_err(|e| e.into())?)),
            None => sled::Result::Ok(None),
        })
    }

    fn insert_typed(&self, key: &K, value: <K as KeyOf>::T) -> sled::Result<Option<sled::IVec>> {
        self.insert(key, value)
    }
}

#[macro_export]
macro_rules! derive_key_of {
    ($key: ty, $value: ty, $NAME: literal, $PREFIX: literal) => {
        impl KeyOf for $key {
            const NAME: &'static str = $NAME;
            const PREFIX: u8 = $PREFIX;
            type T = $value;
        }

        impl From<$value> for sled::IVec {
            fn from(data: $value) -> Self {
                sled::IVec::from(bincode_ser!(&data).unwrap())
            }
        }

        impl<'a> TryFrom<sled::IVec> for $value {
            type Error = sled::Error;

            fn try_from(data: sled::IVec) -> Result<$value, sled::Error> {
                bincode_des!(data.as_ref())
                    .map_err(|_| sled::Error::Unsupported("Error Deserializing".into()))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offers::{OfferEvent, OfferKey, OfferValue, Security, Side};

    #[test]
    fn atomics() {
        let t: sled::Db = sled::Config::default().temporary(true).open().unwrap();
        let atomic = Arc::new(AtomicU64::new(0));

        let val = OfferEvent::Add(OfferValue {
            amount: 23,
            price: Some(34),
            security: Security::BTC,
            side: Side::Buy,
        });
        let (k, _val_bytes): (OfferKey, Option<_>) =
            t.insert_monotonic_atomic(&atomic, val).unwrap();
        assert_eq!(u64::from(k), 0);
        assert_eq!(atomic.load(Ordering::SeqCst), 1);
    }
}
