use crate::offers::Offer;
use std::collections::BTreeSet;
use std::iter::Iterator;
use std::ops::Bound::*;

const F64_TO_U64_FACTOR: f64 = 10_000.0;
fn f64_to_u64(price: f64) -> u64 {
  (price * F64_TO_U64_FACTOR) as u64
}
fn f64_to_i64(price: f64) -> i64 {
  (price * F64_TO_U64_FACTOR) as i64
}

#[derive(Eq, PartialOrd, Ord, Clone)]
struct EngineOffer {
  price: Option<i64>,
  key: u64,
  amount: u64,
}
impl PartialEq for EngineOffer {
  fn eq(&self, other: &Self) -> bool {
    self.key == other.key
  }
}

impl EngineOffer {
  fn from_offer_data(offer: &Offer, excedent: u64) -> Self {
    EngineOffer {
      price: offer.val.price.and_then(|v| Some(f64_to_i64(v))),
      amount: excedent,
      key: u64::from_be_bytes(offer.key),
    }
  }

  fn range_limit(price: f64, is_buy_offer: bool) -> Self {
    debug_assert!(price > 0.0);

    let price = if is_buy_offer {
      // If it's a buy offer, we should search from lower to higher magnitude
      f64_to_i64(price)
    } else {
      // If it's a sell offer, we should search from higher to lower magnitude
      -f64_to_i64(price)
    };
    EngineOffer {
      price: Some(price),
      key: std::u64::MAX,
      amount: std::u64::MAX,
    }
  }
}

enum MatchResult {
  Complete,
  Partial(EngineOffer, u64),
  PartialSame(u64),
}

type Offers = BTreeSet<EngineOffer>;
pub struct Engine {
  sell_offers: Offers,
  buy_offers: Offers,
  matches: Vec<EngineOffer>,
}

impl Engine {
  pub fn process_offer(&mut self, offer: Offer) {
    let is_buy_offer = offer.is_buy_offer();

    let range = if let Some(price) = offer.val.price {
      (
        Unbounded,
        Included(EngineOffer::range_limit(price, is_buy_offer)),
      )
    } else {
      (Unbounded, Unbounded)
    };

    let (same_offers, opposite_offers) = if is_buy_offer {
      (&mut self.buy_offers, &mut self.sell_offers)
    } else {
      (&mut self.sell_offers, &mut self.buy_offers)
    };

    let match_result = match_offer(&mut self.matches, &offer, opposite_offers.range(range));

    for v in self.matches.iter() {
      opposite_offers.remove(v);
    }
    let bb = match &match_result {
      MatchResult::Partial(partial, excedent) => {
        let mut new_offer = partial.clone();
        new_offer.amount -= excedent;
        opposite_offers.replace(new_offer);
      }
      MatchResult::PartialSame(excedent) => {
        same_offers.insert(EngineOffer::from_offer_data(&offer, *excedent));
      }
      _ => {}
    };

    let completed: Vec<_> = self.matches.drain(..self.matches.len()).collect();
  }
}

fn match_offer<'a, T>(matches: &mut Vec<EngineOffer>, offer: &Offer, range: T) -> MatchResult
where
  T: Iterator<Item = &'a EngineOffer>,
{
  let mut excedent = f64_to_u64(offer.val.amount.abs());
  for o in range {
    if o.amount > excedent {
      return MatchResult::Partial(o.clone(), excedent);
    } else if o.amount == excedent {
      matches.push(o.clone());
      return MatchResult::Complete;
    } else {
      excedent -= o.amount;
      matches.push(o.clone());
    }
  }
  MatchResult::PartialSame(excedent)
}

// impl MatchResult {
//     fn get_completed(&self) -> &Vec<EngineOffer> {
//         match self {
//             MatchResult::Complete(completed) => completed,
//             MatchResult::Partial(completed, _partial) => completed,
//             MatchResult::PartialSame(completed, _excedente) => completed,
//         }
//     }
// }
// fn lower(price: f64) -> Self {
//     let price = f64_to_u64(price);
//     EngineOffer {
//         price,
//         key: 0,
//         amount: 0,
//     }
// }

// fn upper(price: f64) -> Self {
//     let price = f64_to_u64(price);
//     EngineOffer {
//         price,
//         key: std::u64::MAX,
//         amount: std::u64::MAX,
//     }
// }
// fn get_offers(&mut self, is_buy_offer: bool) -> (&mut Offers, &mut Offers) {
//     if is_buy_offer {
//         (&mut self.buy_offers, &mut self.sell_offers)
//     } else {
//         (&mut self.sell_offers, &mut self.buy_offers)
//     }
// }
// let match_result = if is_buy_offer {
//     self.match_offer(&offer, opposite_offers.range(range))
// } else {
//     self.match_offer(&offer, opposite_offers.range(range).rev())
// };
