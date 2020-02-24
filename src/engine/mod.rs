pub mod engine_bheap;
pub mod engine_btree;
pub mod engine_keyedheap;
pub mod offer_ord;

use crate::offers::{Offer, OfferKey, Side};
use crossbeam_channel::{self, Receiver, Sender};
#[derive(Debug)]
pub enum MatchResult {
    Complete,
    Partial { offer: Offer, to_substract: u64 },
    None,
}

#[derive(Debug)]
pub struct Matches {
    pub result: MatchResult,
    pub completed: Vec<Offer>,
}

pub trait EngineDataStruct {
    fn match_offer(
        &mut self,
        matches: &mut Vec<Offer>,
        offer: Offer,
        other: &mut Self,
    ) -> MatchResult;
    fn delete_key(self, key: &OfferKey) -> Self;
    fn with_capacity(capacity: usize) -> Self;
}

pub struct Engine<T>
where
    T: EngineDataStruct,
{
    sell_offers: T,
    // market_sell_offers: Vec<MarketEngineOffer>,
    buy_offers: T,
    // market_buy_offers: Vec<MarketEngineOffer>,
    matches: Vec<Offer>,
    receiver: Receiver<Offer>,
    sender: Sender<Matches>,
}

impl<T> Engine<T>
where
    T: EngineDataStruct,
{
    pub fn new(receiver: Receiver<Offer>, sender: Sender<Matches>) -> Self {
        Engine {
            sell_offers: T::with_capacity(24),
            // market_sell_offers: Vec::with_capacity(24),
            buy_offers: T::with_capacity(24),
            // market_buy_offers: Vec::with_capacity(24),
            matches: Vec::with_capacity(24),
            sender,
            receiver,
        }
    }

    pub fn start(&mut self, count: usize) {
        let mut counter = 0;
        while let Ok(offer) = self.receiver.recv() {
            let matches = self.process_offer(offer);
            if let MatchResult::None = matches.result {
                continue;
            }
            self.sender.send(matches).unwrap();
            counter += 1;
            if counter % 10 == 0 {
                println!("{}", counter);
            }
            if count == counter {
                self.sender
                    .send(Matches {
                        completed: Vec::new(),
                        result: MatchResult::None,
                    })
                    .unwrap();
                return;
            }
        }
    }

    pub fn process_offer(&mut self, offer: Offer) -> Matches {
        let (same_offers, opposite_offers) = match offer.value.side {
            Side::Buy => (&mut self.buy_offers, &mut self.sell_offers),
            Side::Sell => (&mut self.sell_offers, &mut self.buy_offers),
        };

        let result = opposite_offers.match_offer(&mut self.matches, offer, same_offers);
        let completed: Vec<_> = self.matches.drain(..self.matches.len()).collect();
        Matches { completed, result }
    }
}

// #[derive(Clone, Debug)]
// pub struct EngineOffer {
//     side: Side,
//     price: Option<u64>,
//     key: u64,
//     amount: u64,
// }

// // pub struct MarketEngineOffer {
// //     key: u64,
// //     amount: u64,
// // }

// impl From<Offer> for EngineOffer {
//     fn from(offer: Offer) -> Self {
//         EngineOffer {
//             side: offer.value.side,
//             price: offer.value.price, // .and_then(|v| Some(f64_to_u64(v))),
//             amount: offer.value.amount, //f64_to_u64(offer.value.amount.abs()),
//             key: u64::from_be_bytes(*offer.key.as_ref()),
//         }
//     }
// }

// impl Eq for EngineOffer {}
// impl PartialEq for EngineOffer {
//     fn eq(&self, other: &Self) -> bool {
//         self.key == other.key
//     }
// }
// impl PartialOrd for EngineOffer {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }
// impl Ord for EngineOffer {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         match self.side {
//             Side::Buy => match self.price {
//                 Some(price) => match other.price {
//                     Some(price_other) => price
//                         .cmp(&price_other)
//                         .then_with(|| self.key.cmp(&other.key)),
//                     None => Greater,
//                 },
//                 None => match other.price {
//                     Some(_price_other) => Less,
//                     None => self.key.cmp(&other.key),
//                 },
//             },
//             Side::Sell => match self.price {
//                 Some(price) => match other.price {
//                     Some(price_other) => price_other
//                         .cmp(&price)
//                         .then_with(|| self.key.cmp(&other.key)),
//                     None => Greater,
//                 },
//                 None => match other.price {
//                     Some(_price_other) => Less,
//                     None => self.key.cmp(&other.key),
//                 },
//             },
//         }
//     }
// }
