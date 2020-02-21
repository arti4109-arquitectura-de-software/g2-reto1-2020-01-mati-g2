use crate::typed_tree::KeyOf;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub type Price = f64;

// #[derive(Deserialize, Serialize, Debug)]
// enum Offer {
//   Sell(OfferData),
//   Buy(OfferData),
// }

#[derive(Deserialize, Serialize, Debug)]
pub enum Security {
  BTC,
  USD,
  COP,
}

#[derive(Deserialize, Serialize)]
pub struct Offer {
  pub key: OfferKey,
  pub val: OfferData,
}

impl Offer {
  pub fn is_sell_offer(&self) -> bool {
    self.val.amount > 0.0
  }
  pub fn is_buy_offer(&self) -> bool {
    self.val.amount < 0.0
  }
}


#[derive(Deserialize, Serialize, Debug)]
pub struct OfferData {
  pub security: Security,
  pub amount: f64,
  pub price: Option<Price>,
}
pub type OfferKey = [u8; 8];

// impl std::convert::AsRef<[u8]> for OfferKey {
//   fn as_ref(&self) -> &[u8] {
//     self.0.as_ref()
//   }
// }

impl KeyOf for OfferKey {
  const NAME: &'static str = "Offer";
  const PREFIX: u8 = 0;
  type T = OfferData;
}

impl From<OfferData> for sled::IVec {
  fn from(data: OfferData) -> Self {
    // unsafe { any_as_u8_slice(&data) }
    sled::IVec::from(bincode::serialize(&data).unwrap())
  }
}

impl<'a> TryFrom<sled::IVec> for OfferData {
  type Error = sled::Error;

  fn try_from(data: sled::IVec) -> Result<OfferData, sled::Error> {
    bincode::deserialize(data.as_ref())
      .map_err(|_| sled::Error::Unsupported("Error Deserializing".into()))
  }
}
