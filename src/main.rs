use matching_engine::{offers::*, typed_tree::*};
use sled;
// use std::sync::atomic::AtomicU64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let t: sled::Db = sled::open("database.sled")?;
    // let atomic = AtomicU64::new(0);
    
    for v in t.iter() {
        println!("{:?}", v.unwrap());
    }

    if false {
        let kv = KeyVal {
            key: OfferKey::from(1),
            val: OfferEvent::Add(OfferValue {
                side: Side::Buy,
                security: Security::BTC,
                price: Some(289),
                amount: 23,
            }),
        };

        t.insert_typed(&kv.key, kv.val)?;
        let val = t.get_typed(&kv.key);
        println!("{:?}", val?.unwrap());
    }

    if false {
        let offer_val = OfferEvent::Add(OfferValue {
            side: Side::Sell,
            security: Security::COP,
            price: Some(1203),
            amount: 2000,
        });

        let (key, _): (OfferKey, Option<_>) = t.insert_monotonic(offer_val)?;
        println!("{:?}", key);
        println!("{:?}", u64::from(key));
    }

    if false {
        println!("{}", std::str::from_utf8(&t.get("key")?.unwrap())?);
        println!("{}", t.len());
        t.insert("key", "value")?;
        let v = t.get("key")?.unwrap();
        assert_eq!(v, "value");
        println!("{}", std::str::from_utf8(&v)?);
    }

    Ok(())
}

// where
//   T: Sized + TryFrom<sled::IVec> + Into<sled::IVec>,
//   <T as std::convert::TryFrom<sled::IVec>>::Error: Into<sled::Error>,

// trait SledSerde {
//   fn serialize(&self) -> bincode::Result<Vec<u8>>;
//   fn deserialize<'a, T>(bytes: &'a [u8]) -> bincode::Result<Self>;
// }

// impl<'a> SledSerde for OfferData  {
//   fn serialize(&self) -> Vec<u8>{
//     bincode::serialize(self).unwrap()
//   }

//   fn deserialize(bytes: &'a [u8]) -> bincode::Result<Self>{
//     bincode::deserialize(bytes)
//   }
// }

//const OFFER_SIZE: usize = ::std::mem::size_of::<OfferData>();
//
// unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
//   ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
// }
//use std::convert::TryFrom;
// impl From<sled::IVec> for OfferData {
//   //type Error = sled::Error;

//   fn from(data: sled::IVec) -> Self {
//     let slice = data.as_ref();
//     if slice.len() == OFFER_SIZE {
//       let ptr = slice.as_ptr() as *const [u8; OFFER_SIZE];
//       unsafe { std::mem::transmute::<[u8; OFFER_SIZE], OfferData>(*ptr) }
//     } else {
//       panic!();
//       //Result::Err(sled::Error::Unsupported("".into()))
//     }
//   }
// }
