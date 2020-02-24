use crate::{
    bincode_des, bincode_ser, derive_key_of, derive_monotonic_key, derive_simple_struct,
    offers::Security, typed_tree::KeyOf,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

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

derive_key_of!(MatchKey, MatchValue, "Match", 1);
