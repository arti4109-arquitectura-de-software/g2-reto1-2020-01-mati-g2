use crate::{derive_offer_ord, engine::offer_ord::OfferOrdSigned};
use crate::{
    engine::{EngineDataStruct, MatchResult},
    offers::{Offer, Side},
};
use std::collections::BinaryHeap;

#[derive(Eq, PartialOrd, Clone, Debug)]
struct EngineOfferBH {
    price: Option<i64>,
    key: [u8; 8],
    amount: u64,
}
derive_offer_ord!(OfferOrdSigned, EngineOfferBH, cmp_max);

type BinaryHeapEngine = BinaryHeap<EngineOfferBH>;

// impl BinaryHeapEngine {
//     fn delete_key(&mut self) {
//         self.re
//     }
// }
impl EngineDataStruct for BinaryHeapEngine {
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }

    fn match_offer(
        &mut self,
        matches: &mut Vec<Offer>,
        offer: Offer,
        other: &mut Self,
    ) -> MatchResult {
        // let mut excedent = f64_to_u64(offer.value.amount.abs());
        let mut excedent = offer.value.amount;
        let opposite_side = offer.opposite_side();

        if let Some(price) = offer.value.price {
            // let price = match offer.value.side {
            //     Side::Buy => f64_to_i64(price),
            //     Side::Sell => -f64_to_i64(price),
            // };
            let price = match offer.value.side {
                Side::Buy => price as i64,
                Side::Sell => -(price as i64),
            };

            while let Some(o) = self.peek() {
                if let Some(p) = o.price {
                    if price > p {
                        break;
                    }
                }

                if o.amount > excedent {
                    let new_offer = o.into_offer(opposite_side, offer.value.security);
                    self.peek_mut().unwrap().amount -= excedent;

                    return MatchResult::Partial {
                        offer: new_offer,
                        to_substract: excedent,
                    };
                }

                let o = self.pop().unwrap();
                matches.push(o.into_offer(opposite_side, offer.value.security));
                if o.amount == excedent {
                    return MatchResult::Complete;
                } else {
                    excedent -= o.amount;
                }
            }
        } else {
            while let Some(o) = self.peek() {
                if o.amount > excedent {
                    let new_offer = o.into_offer(opposite_side, offer.value.security);
                    self.peek_mut().unwrap().amount -= excedent;

                    return MatchResult::Partial {
                        offer: new_offer,
                        to_substract: excedent,
                    };
                }

                let o = self.pop().unwrap();
                matches.push(o.into_offer(opposite_side, offer.value.security));
                if o.amount == excedent {
                    matches.push(offer.into());
                    return MatchResult::Complete;
                } else {
                    excedent -= o.amount;
                }
            }
        }

        let new_offer = EngineOfferBH {
            price: EngineOfferBH::price_from_offer(&offer),
            amount: excedent,
            key: *offer.key.as_ref(),
        };
        other.push(new_offer);

        if offer.value.amount == excedent {
            MatchResult::None
        } else {
            let to_substract = offer.value.amount - excedent;
            MatchResult::Partial {
                offer,
                to_substract,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        engine::{Engine, Matches},
        offers::{Offer, OfferValue, Security, Side},
    };

    #[test]
    fn engine_test() {
        let (_sender_offer, receiver_offer) = crossbeam_channel::unbounded::<Offer>();
        let (sender_matches, _receiver_matches) = crossbeam_channel::unbounded::<Matches>();
        let mut engine = Engine::<BinaryHeapEngine>::new(receiver_offer, sender_matches);
        let offer = Offer {
            key: u64::to_be_bytes(0).into(),
            value: OfferValue {
                side: Side::Buy,
                security: Security::BTC,
                amount: 10,
                price: None,
            },
        };
        engine.process_offer(offer);
        let offer = Offer {
            key: u64::to_be_bytes(1).into(),
            value: OfferValue {
                side: Side::Buy,
                security: Security::BTC,
                amount: 5,
                price: Some(32),
            },
        };
        let matches = engine.process_offer(offer);
        println!("{:?}", matches);

        let offer = Offer {
            key: u64::to_be_bytes(2).into(),
            value: OfferValue {
                side: Side::Sell,
                security: Security::BTC,
                amount: 8,
                price: None,
            },
        };
        let matches = engine.process_offer(offer);
        println!("{:?}", matches);

        let offer = Offer {
            key: u64::to_be_bytes(3).into(),
            value: OfferValue {
                side: Side::Sell,
                security: Security::BTC,
                amount: 6,
                price: Some(33),
            },
        };
        let matches = engine.process_offer(offer);
        println!("{:?}", matches);
    }
}

// impl OfferOrdSigned for EngineOfferBH {
//     fn key(&self) -> u64 {
//         self.key
//     }
//     fn price(&self) -> Option<i64> {
//         self.price
//     }
//     fn amount(&self) -> u64 {
//         self.amount
//     }
// }
// impl cmp::PartialEq for EngineOfferBH {
//     fn eq(&self, other: &Self) -> bool {
//         self.key == other.key
//     }
// }
// impl cmp::Ord for EngineOfferBH {
//     fn cmp(&self, other: &Self) -> cmp::Ordering {
//         self.cmp_max(other)
//     }
// }

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
