use crate::{
    bincode_des, bincode_ser, derive_key_of, derive_monotonic_key, derive_simple_struct,
    engine::{MatchResult, Matches},
    offers::{Offer, Security},
    typed_tree::*,
};
use core::sync::atomic::AtomicU64;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::sync::Arc;

// #[derive(Deserialize, Serialize, Debug)]
// pub enum MatchState {
//     Complete,
//     Partial,
// }

pub struct Match {
    pub key: MatchKey,
    pub value: MatchValue,
}

pub struct MatchKey([u8; 8]);
derive_monotonic_key!(MatchKey);

#[derive(Deserialize, Serialize, Debug)]
pub struct MatchValue {
    pub reference: [u8; 8],
    pub security: Security,
    pub price: Option<u64>,
    pub amount: u64,
}

impl From<Offer> for MatchValue {
    fn from(o: Offer) -> Self {
        MatchValue {
            amount: o.value.amount,
            price: o.value.price,
            reference: o.key.into(),
            security: o.value.security,
        }
    }
}

use crossbeam_channel::{self, Receiver};
pub struct MatchPersistor {
    receiver: Receiver<Matches>,
    db: sled::Db,
    atomic: Arc<AtomicU64>,
}

impl MatchPersistor {
    pub fn new(receiver: Receiver<Matches>, mut db: sled::Db) -> Self {
        let atomic = <sled::Db as MonotonicTypedTree<MatchKey>>::get_max_key(&mut db).unwrap();

        MatchPersistor {
            receiver,
            db,
            atomic,
        }
    }
    pub fn start(&mut self) {
        let mut counter = 0;
        while let Ok(matches) = self.receiver.recv() {
            if let MatchResult::None = matches.result {
                return;
            }
            counter += 1;
            if counter % 50 == 0 {
                println!("Persistor: {}", counter);
            }

            if let MatchResult::Partial {
                mut offer,
                to_substract,
            } = matches.result
            {
                offer.value.amount -= to_substract;
                self.db.insert_monotonic_atomic(&self.atomic, offer.into()).unwrap() as (MatchKey, Option<_>);
            }
            
            for m in matches.completed.into_iter().map(|o| o.into()) {
                self.db.insert_monotonic_atomic(&self.atomic, m).unwrap() as (MatchKey, Option<_>);
            }
        }
    }

    pub fn persistir(&self, m: MatchValue) {
        self.db.insert_monotonic_atomic(&self.atomic, m).unwrap() as (MatchKey, Option<_>);
    }
}

derive_key_of!(MatchKey, MatchValue, "Match", 1);
