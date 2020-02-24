use crate::{
    bincode_des, bincode_ser, derive_key_of, derive_monotonic_key, derive_simple_struct,
    typed_tree::KeyOf,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Deserialize, Serialize, Debug)]
pub struct Offer {
    pub key: OfferKey,
    pub value: OfferValue,
}

impl Offer {
    pub fn opposite_side(&self) -> Side {
        match self.value.side {
            Side::Sell => Side::Buy,
            Side::Buy => Side::Sell,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OfferKey([u8; 8]);
derive_monotonic_key!(OfferKey);

#[derive(Deserialize, Serialize, Debug)]
pub enum OfferEvent {
    Delete(OfferKey),
    Add(OfferValue),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OfferValue {
    pub security: Security,
    pub side: Side,
    pub amount: u64,
    pub price: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum Security {
    BTC,
    USD,
    COP,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Sell,
    Buy,
}

derive_key_of!(OfferKey, OfferEvent, "OfferEvent", 0);

// impl std::convert::AsRef<[u8]> for OfferKey {
//     fn as_ref(&self) -> &[u8] {
//         self.0.as_ref()
//     }
// }
// impl From<u64> for OfferKey {
//     fn from(v: u64) -> Self {
//         OfferKey(u64::to_be_bytes(v))
//     }
// }
// impl From<OfferKey> for u64 {
//     fn from(v: OfferKey) -> Self {
//         u64::from_be_bytes(v.0)
//     }
// }

// impl std::convert::AsRef<[u8; 8]> for OfferKey {
//     fn as_ref(&self) -> &[u8; 8] {
//         &self.0
//     }
// }
// impl From<[u8; 8]> for OfferKey {
//     fn from(v: [u8; 8]) -> Self {
//         OfferKey(v)
//     }
// }
// impl From<OfferKey> for [u8; 8] {
//     fn from(v: OfferKey) -> Self {
//         v.0
//     }
// }

// impl KeyOf for OfferKey {
//     const NAME: &'static str = "Offer";
//     const PREFIX: u8 = 0;
//     type T = OfferValue;
// }

// impl From<OfferValue> for sled::IVec {
//     fn from(data: OfferValue) -> Self {
//         // unsafe { any_as_u8_slice(&data) }
//         sled::IVec::from(bincode_ser!(&data).unwrap())
//     }
// }

// impl<'a> TryFrom<sled::IVec> for OfferValue {
//     type Error = sled::Error;

//     fn try_from(data: sled::IVec) -> Result<OfferValue, sled::Error> {
//         bincode_des!(data.as_ref())
//             .map_err(|_| sled::Error::Unsupported("Error Deserializing".into()))
//     }
// }
