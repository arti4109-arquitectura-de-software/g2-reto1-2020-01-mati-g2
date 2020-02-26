#![feature(binary_heap_drain_sorted)]
#![feature(binary_heap_into_iter_sorted)]
#![feature(test)]

pub mod engine;
pub mod matches;
pub mod offers;
pub mod typed_tree;
mod utils;

#[cfg(test)]
mod tests {
    use crate::offers::OfferEventKeyed;
    use itchy::{self, AddOrder};
    use rand_distr::{Bernoulli, Distribution, Normal};
    use std::collections::{BinaryHeap, HashMap, HashSet};
    extern crate test;
    use crate::{
        engine::{engine_keyedheap::KeyedBinaryHeapEngine, Engine, Matches},
        matches::MatchPersistor,
        offers::{Offer, OfferEvent, OfferKey, OfferValue, Security, Side},
    };
    use test::Bencher;

    #[derive(Debug)]
    struct Stats {
        adds: Vec<itchy::AddOrder>,
        deletes: u32,
    }
    // #[bench]
    // fn bench_18std2(b: &mut Bencher) {
    //     b.iter(|| {
    //         let offers = generate_offers();
    //         let (_sender_offer, receiver_offer) = crossbeam_channel::unbounded::<Offer>();
    //         let (sender_matches, _receiver_matches) = crossbeam_channel::unbounded::<Matches>();

    //         let mut engine = Engine::<BinaryHeapEngine>::new(receiver_offer, sender_matches);
    //         for o in offers.into_iter() {
    //             engine.process_offer(o);
    //         }
    //     })
    // }

    // #[bench]
    // fn bench_keyed(b: &mut Bencher) {
    //     b.iter(|| {
    //         let offers = generate_offers();
    //         let (_sender_offer, receiver_offer) = crossbeam_channel::unbounded::<Offer>();
    //         let (sender_matches, _receiver_matches) = crossbeam_channel::unbounded::<Matches>();

    //         let mut engine = Engine::<KeyedBinaryHeapEngine>::new(receiver_offer, sender_matches);
    //         let mut k:u64 = 1;
    //         for o in offers.into_iter() {
    //             match o {
    //                 OfferEvent::Add(v) => {
    //                     k += 1;
    //                     engine.process_offer(Offer {
    //                         key: OfferKey::from(k),
    //                         value: v,
    //                     });
    //                 }
    //                 OfferEvent::Delete(_) => {
    //                     engine.delete_offer(&OfferKey::from(k));
    //                     k -= 1;
    //                 }
    //             }
    //         }
    //     })
    // }

    fn generate_offers_raw() -> Vec<Offer> {
        let normal = Normal::new(200.0, 200.0 * 0.20).unwrap();
        let bernoulli = Bernoulli::new(0.5).unwrap();

        let normal_p = Normal::new(200.0, 200.0 * 0.01).unwrap();
        let bernoulli_p = Bernoulli::new(0.1).unwrap();

        let mut rn = rand::thread_rng();

        let offers: Vec<Offer> = (0..1000)
            .map(|i| {
                let side = if bernoulli.sample(&mut rn) {
                    Side::Buy
                } else {
                    Side::Sell
                };

                let price = if bernoulli_p.sample(&mut rn) {
                    Some(normal_p.sample(&mut rn) as u64)
                } else {
                    None
                };

                Offer {
                    key: OfferKey::from(i),
                    value: OfferValue {
                        side,
                        security: Security::BTC,
                        amount: normal.sample(&mut rn) as u64,
                        price,
                    },
                }
            })
            .collect();
        offers
    }

    #[test]
    fn combined() {
        let matches_db = sled::open(format!("database-matches.sled")).unwrap();

        let durations: Vec<std::time::Duration> = (1..10)
            .map(|_| {
                let (sender_offer, receiver_offer) =
                    crossbeam_channel::unbounded::<OfferEventKeyed>();
                let (sender_matches, receiver_matches) = crossbeam_channel::unbounded::<Matches>();

                let mut engine =
                    Engine::<KeyedBinaryHeapEngine>::new(receiver_offer, sender_matches);
                let mut persistor = MatchPersistor::new(receiver_matches, matches_db.clone());

                let offers = generate_offers();
                let persistor_id = std::thread::spawn(move || {
                    persistor.start();
                    println!("DONE pers");
                });
                let offers_len = offers.len();
                let eng_id = std::thread::spawn(move || {
                    if let Err(e) = std::panic::catch_unwind(move || {
                        engine.start(offers_len);
                    }){
                        println!("Err eng {:?}", e);
                    }
                    println!("DONE eng");
                });

                let t1 = std::time::Instant::now();
                for o in offers.into_iter() {
                    sender_offer.send(o).unwrap();
                }
                println!("DONE");
                eng_id.join().unwrap();
                persistor_id.join().unwrap();
                let dur = std::time::Instant::now().duration_since(t1);

                matches_db.clear().unwrap();
                dur
            })
            .collect();
        println!("{:?}", durations);
    }

    fn generate_offers() -> Vec<OfferEventKeyed> {
        let normal = Normal::new(200.0, 200.0 * 0.20).unwrap();
        let bernoulli = Bernoulli::new(0.5).unwrap();
        let bernoulli_2 = Bernoulli::new(0.5).unwrap();

        let normal_p = Normal::new(200.0, 200.0 * 0.01).unwrap();
        let bernoulli_p = Bernoulli::new(0.1).unwrap();

        let mut rn = rand::thread_rng();

        let offers: Vec<OfferEventKeyed> = (0..1000)
            .map(|i| {
                if bernoulli_2.sample(&mut rn) {
                    let side = if bernoulli.sample(&mut rn) {
                        Side::Buy
                    } else {
                        Side::Sell
                    };

                    let price = if bernoulli_p.sample(&mut rn) {
                        Some(normal_p.sample(&mut rn) as u64)
                    } else {
                        None
                    };
                    OfferEventKeyed::Add(
                        OfferKey::from(i),
                        OfferValue {
                            side,
                            security: Security::BTC,
                            amount: normal.sample(&mut rn) as u64,
                            price,
                        },
                    )
                } else {
                    OfferEventKeyed::Delete(OfferKey::from(0), OfferKey::from(0))
                }
            })
            .collect();
        offers
    }

    // fn generate_offers() -> Vec<OfferEvent> {
    //     let normal = Normal::new(200.0, 200.0 * 0.20).unwrap();
    //     let bernoulli = Bernoulli::new(0.5).unwrap();
    //     let bernoulli_2 = Bernoulli::new(0.5).unwrap();

    //     let normal_p = Normal::new(200.0, 200.0 * 0.01).unwrap();
    //     let bernoulli_p = Bernoulli::new(0.1).unwrap();

    //     let mut rn = rand::thread_rng();

    //     let offers: Vec<OfferEvent> = (0..1000)
    //         .map(|_i| {
    //             if bernoulli_2.sample(&mut rn) {
    //                 let side = if bernoulli.sample(&mut rn) {
    //                     Side::Buy
    //                 } else {
    //                     Side::Sell
    //                 };

    //                 let price = if bernoulli_p.sample(&mut rn) {
    //                     Some(normal_p.sample(&mut rn) as u64)
    //                 } else {
    //                     None
    //                 };

    //                 OfferEvent::Add(OfferValue {
    //                     side,
    //                     security: Security::BTC,
    //                     amount: normal.sample(&mut rn) as u64,
    //                     price,
    //                 })
    //             } else {
    //                 OfferEvent::Delete(OfferKey::from(0))
    //             }
    //         })
    //         .collect();
    //     offers
    // }

    // fn generate_offers_raw() -> Vec<Offer> {
    //     let normal = Normal::new(200.0, 200.0 * 0.20).unwrap();
    //     let bernoulli = Bernoulli::new(0.5).unwrap();

    //     let normal_p = Normal::new(200.0, 200.0 * 0.01).unwrap();
    //     let bernoulli_p = Bernoulli::new(0.1).unwrap();

    //     let mut rn = rand::thread_rng();

    //     let offers: Vec<Offer> = (0..1000)
    //         .map(|i| {
    //             let side = if bernoulli.sample(&mut rn) {
    //                 Side::Buy
    //             } else {
    //                 Side::Sell
    //             };

    //             let price = if bernoulli_p.sample(&mut rn) {
    //                 Some(normal_p.sample(&mut rn) as u64)
    //             } else {
    //                 None
    //             };

    //             Offer {
    //                 key: OfferKey::from(i),
    //                 value: OfferValue {
    //                     side,
    //                     security: Security::BTC,
    //                     amount: normal.sample(&mut rn) as u64,
    //                     price,
    //                 },
    //             }
    //         })
    //         .collect();
    //     offers
    // }

    // #[bench]
    // fn bench_18std(b: &mut Bencher) {
    //     let normal_p = Normal::new(200.0, 100000.0).unwrap();
    //     let mut rn = rand::thread_rng();

    //     b.iter(|| {
    //         let matches_db =
    //             sled::open(format!("database-matches{}.sled", normal_p.sample(&mut rn))).unwrap();
    //         let offers: Vec<Offer> = generate_offers_raw();

    //         let (_sender_matches, receiver_matches) = crossbeam_channel::unbounded::<Matches>();
    //         let persistor = MatchPersistor::new(receiver_matches, matches_db.clone());
    //         for o in offers {
    //             persistor.persistir(o.into())
    //         }
    //         matches_db.flush().unwrap();
    //         matches_db.clear().unwrap();
    //     });
    // }

    #[test]
    fn itch50_parser3() {
        let stream = itchy::MessageStream::from_file(
            r#"C:\Users\jmanu\Downloads\20200130.BX_ITCH_50\20200130.BX_ITCH_50"#,
        )
        .unwrap();
        let mut map = HashMap::<String, Stats>::default();
        let mut map_stock = HashMap::<u64, String>::default();

        for msg in stream.filter(|v| {
            if let Ok(v) = v.as_ref() {
                v.tag == 65 || v.tag == 85 || v.tag == 68
            } else {
                false
            }
        }) {
            let msg = msg.unwrap();
            match msg.body {
                itchy::Body::AddOrder(o) => {
                    map_stock.insert(o.reference, String::from(o.stock.as_str()));
                    let entry = map.entry(String::from(o.stock.as_str()));
                    entry
                        .and_modify(|v| {
                            v.adds.push(o);
                        })
                        .or_insert(Stats {
                            adds: Vec::new(),
                            deletes: 0,
                        });
                }
                itchy::Body::DeleteOrder { reference } => {
                    let o = map_stock.get(&reference);
                    if let Some(o) = o {
                        let o = map.get_mut(o).unwrap();
                        o.deletes += 1;
                    }
                }
                itchy::Body::ReplaceOrder(_) => {}
                _ => unreachable!(),
            }
        }
        println!("{:?}", map.len());
        let mut stds = Vec::new();
        let mut stds_prices = Vec::new();
        for (k, v) in map.into_iter().filter(|(_, v)| v.adds.len() > 10) {
            let prices: Vec<f64> = v
                .adds
                .iter()
                .map::<f64, _>(|a: &AddOrder| {
                    let v: u32 = unsafe { std::mem::transmute(a.price) };
                    f64::from(v) / 10_000.0
                })
                .collect();

            let (min, mean, max, std) = min_max_std(&k, &prices);

            stds_prices.push(std / mean);

            let (min, mean, max, std) =
                min_max_std(&k, &v.adds.iter().map(|v| v.shares as f64).collect());
            stds.push(std / mean);
        }
        min_max_std("AMOUNTS: ", &stds);
        min_max_std("PRICES: ", &stds_prices);
    }

    fn min_max_std(k: &str, arr: &Vec<f64>) -> (f64, f64, f64, f64) {
        let size = arr.len() as f64;
        let mean = arr.iter().sum::<f64>() / size;
        let (min, max, mut std) = arr.iter().fold(
            (std::f64::MAX, std::f64::MIN, 0 as f64),
            |(min, max, std), v| {
                (
                    if min < *v { min } else { *v },
                    if max > *v { max } else { *v },
                    std + (*v as f64 - mean).powi(2),
                )
            },
        );
        std /= size;
        std = std.powf(0.5);
        println!(
            "{}: Prices  len {} -- min {} -- mean {} -- max {} -- std {}",
            k,
            arr.len(),
            min,
            mean,
            max,
            std
        );
        (min, mean, max, std)
    }

    #[test]
    fn transmute_test() {
        let p = 67_899_000;
        let val = itchy::Price4::from(p);
        let val2: u32 = unsafe { std::mem::transmute(val) };
        assert_eq!(val2, p);
    }

    #[test]
    fn b_heap() {
        let mut heap = BinaryHeap::<u8>::new();
        heap.extend(vec![1, 2, 3, 4]);
        for v in heap.drain_sorted() {
            if v > 2 {
                break;
            }
        }
        assert_eq!((vec![] as Vec<u8>), heap.into_vec());
    }

    #[test]
    fn vec_drain() {
        let mut v = vec![1, 2, 3, 4];
        v.drain(0..1);
        assert_eq!(vec![2, 3, 4], v);
    }

    #[test]
    fn itch50_parser1() {
        let stream = itchy::MessageStream::from_file(
            r#"C:\Users\jmanu\Downloads\20200130.BX_ITCH_50\20200130.BX_ITCH_50"#,
        )
        .unwrap();
        let mut counter: u32 = 0;

        let mut map = std::collections::HashMap::<u8, u32>::default();
        // let tags: [u8; 6] = [88, 83, 86, 68, 65, 85];
        // let mut seen: HashSet<u8> = tags.iter().map(|tag| *tag).collect();

        // 51_645
        // {88(OrderCancelled): 1_311, 83(SystemEvent): 2,   86(MwcbDeclineLevel): 1,
        //  68(DeleteOrder):   24_675, 65(AddOrder): 25_128, 85(ReplaceOrder): 528}
        // 32_969
        // 72 (TradingAction): 8906, 82(StockDirectory): 8915, 89 (RegShoRestriction): 8906,
        // 76 (ParticipantPosition): 6144, 69 (OrderExecuted): 72, 80 (NonCrossTrade): 26
        // 67 (OrderExecutedWithPrice), 75 (IpoQuotingPeriod), 73 (Imbalance),
        // 81 (CrossTrade), 74 (LULDAuctionCollar):

        // {65(add): 184_735_355, 68(delete): 180_285_101, 85(replace): 36_777_372,
        //  69(executed): 8_415_610, 88(cancelled): 4_990_972,
        //  73(Imbalance): 4_025_192, 80(NonCrossTrade): 1_779_727, 70(AddOrder): 1_875_350,
        //  76: 216_802, 67: 139_474,  82: 8_916,
        //  89: 9_068, 81: 17_835, 72: 8_921,
        //  75: 2, 83: 6, 74: 5, 86: 1,}

        let tags: [u8; 6] = [69, 80, 72, 82, 76, 89];
        let seen: HashSet<u8> = tags.iter().map(|tag| *tag).collect();

        for msg in stream.filter(|v| {
            if let Ok(v) = v.as_ref() {
                // v.tag != 69
                //     && v.tag != 80
                //     && v.tag != 72
                //     && v.tag != 82
                //     && v.tag != 89
                //     && v.tag != 76
                seen.contains(&v.tag)
            } else {
                false
            }
        }) {
            let msg = msg.unwrap();

            let entry = map.entry(msg.tag);
            entry.and_modify(|v| *v += 1).or_insert(1);
            counter += 1;

            // if seen.len() == 0 {
            //     break;
            // }
            // if seen.remove(&msg.tag) {
            //     println!("{:?}", msg);
            // }
        }
        println!("{}", counter);
        println!("{:?}", map);
    }
    #[test]
    fn itch50_parser2() {
        let stream = itchy::MessageStream::from_file(
            r#"C:\Users\jmanu\Downloads\01302020.NASDAQ_ITCH50\01302020.NASDAQ_ITCH50"#,
        )
        .unwrap();
        let mut map = std::collections::HashMap::<u8, u32>::default();
        let mut counter = 0;
        for msg in stream.filter(|v| v.is_ok()).take(1) {
            let msg = msg.unwrap();
            let entry = map.entry(msg.tag);
            entry.and_modify(|v| *v += 1).or_insert_with(|| {
                println!("{:?}", msg);
                1
            });
            counter += 1;
        }
        println!("{:?}", counter);
        println!("{:?}", map);
    }

    use bincode;
    #[test]
    fn bincode_endianess() {
        println!("{:?}", bincode::serialize(&(4 as u64)).unwrap());
        let mut conf = bincode::config();
        println!("{:?}", conf.big_endian().serialize(&(4 as u64)).unwrap());
        println!("{:?}", bincode::serialize(&(4 as u64)).unwrap());
    }
}
