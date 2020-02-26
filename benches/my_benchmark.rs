use criterion::{criterion_group, criterion_main, Criterion};
use matching_engine::offers::OfferEventKeyed;
use matching_engine::{
    engine::{engine_keyedheap::KeyedBinaryHeapEngine, Engine, Matches},
    matches::MatchPersistor,
    offers::{Offer, OfferEvent, OfferKey, OfferValue, Security, Side},
};

use rand_distr::{Bernoulli, Distribution, Normal};

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

fn bench_keyed() {
    let offers = generate_offers();
    let (_sender_offer, receiver_offer) = crossbeam_channel::unbounded::<OfferEventKeyed>();
    let (sender_matches, _receiver_matches) = crossbeam_channel::unbounded::<Matches>();

    let mut engine = Engine::<KeyedBinaryHeapEngine>::new(receiver_offer, sender_matches);
    let mut k: u64 = 1;
    for o in offers.into_iter() {
        match o {
            OfferEventKeyed::Add(_, v) => {
                k += 1;
                engine.process_offer(Offer {
                    key: OfferKey::from(k),
                    value: v,
                });
            }
            OfferEventKeyed::Delete(_, _) => {
                engine.delete_offer(&OfferKey::from(k));
                k -= 1;
            }
        }
    }
}

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

fn combined() {
    let matches_db = sled::open(format!("database-matches.sled")).unwrap();

    let durations: Vec<std::time::Duration> = (1..10)
        .map(|_| {
            let (sender_offer, receiver_offer) = crossbeam_channel::unbounded::<OfferEventKeyed>();
            let (sender_matches, receiver_matches) = crossbeam_channel::unbounded::<Matches>();

            let mut engine = Engine::<KeyedBinaryHeapEngine>::new(receiver_offer, sender_matches);
            let mut persistor = MatchPersistor::new(receiver_matches, matches_db.clone());

            let offers = generate_offers();
            let persistor_id = std::thread::spawn(move || {
                persistor.start();
            });
            let offers_len = offers.len();
            let eng_id = std::thread::spawn(move || {
                engine.start(offers_len);
            });

            let t1 = std::time::Instant::now();
            for o in offers.into_iter() {
                sender_offer.send(o).unwrap();
            }
            eng_id.join().unwrap();
            persistor_id.join().unwrap();
            let dur = std::time::Instant::now().duration_since(t1);

            matches_db.clear().unwrap();
            dur
        })
        .collect();
    println!("{:?}", durations);
}

fn bench_18std() {
    let normal_p = Normal::new(200.0, 100000.0).unwrap();
    let mut rn = rand::thread_rng();

    let matches_db =
        sled::open(format!("database-matches{}.sled", normal_p.sample(&mut rn))).unwrap();
    let offers: Vec<Offer> = generate_offers_raw();

    let (_sender_matches, receiver_matches) = crossbeam_channel::unbounded::<Matches>();
    let persistor = MatchPersistor::new(receiver_matches, matches_db.clone());
    for o in offers {
        persistor.persistir(o.into())
    }
    matches_db.flush().unwrap();
    matches_db.clear().unwrap();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("bench_18std test init", |b| b.iter(|| bench_18std()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
