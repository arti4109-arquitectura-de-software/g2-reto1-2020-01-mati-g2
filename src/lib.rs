#![feature(binary_heap_drain_sorted)]
#![feature(binary_heap_into_iter_sorted)]

mod offers;
mod typed_tree;
 
/// Returns the mantissa, exponent and sign as integers.
fn integer_decode(float: f64) -> (u64, i16, i8) {
    let bits: u64 = unsafe { ::std::mem::transmute(float) };
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };
    // Exponent bias + mantissa shift
    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

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
}
