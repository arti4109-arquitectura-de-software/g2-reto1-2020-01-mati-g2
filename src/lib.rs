#![feature(binary_heap_drain_sorted)]
#![feature(binary_heap_into_iter_sorted)]

pub mod engine;
pub mod offers;
pub mod typed_tree;
pub mod matches;
mod utils;

#[cfg(test)]
mod tests {
    use itchy;
    use std::collections::{BinaryHeap, HashSet};

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
