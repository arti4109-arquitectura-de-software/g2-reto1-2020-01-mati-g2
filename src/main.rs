//#![feature(generic_associated_types)]

use bincode;
use sled;


// where
//   T: Sized + TryFrom<sled::IVec> + Into<sled::IVec>,
//   <T as std::convert::TryFrom<sled::IVec>>::Error: Into<sled::Error>,

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let t = sled::open("database.sled")?;
  let mut conf = bincode::config();
  conf.big_endian();
  // let kv = KeyVal {
  //   key: OfferKey(u64::to_be_bytes(0)),
  //   val: OfferData {
  //     security: Security::BTC,
  //     price: 223.0,
  //     limit: None,
  //   },
  // };

  // let k = kv.key;
  // t.insert_typed(kv);
  // let val = t.get_typed(k);
  // println!("{:?}", val?.unwrap());

  println!("{}", std::str::from_utf8(&t.get("key")?.unwrap())?);
  println!("{}", t.len());
  // t.insert("key", "value")?;
  // let v = t.get("key")?.unwrap();
  // assert_eq!(v, "value");
  // println!("{}", std::str::from_utf8(&v)?);
  Ok(())
}











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
