use std::collections::BinaryHeap;
use std::cmp::Reverse;


#[cfg(test)]
mod top_k_tests {
    use super::*;

    #[test]
    fn test_top_k_basic() {
        let nums = vec![4, 1, 7, 3, 9, 2, 5];
        let res = vec![5,7,9];
        let k = 3;
        let mut top_k = BinaryHeap::new();
        nums.into_iter().for_each(|num|{
            if top_k.len() < k {
                top_k.push(Reverse(num));
            } else if top_k.peek().unwrap().0 < num {
                top_k.pop();
                top_k.push(Reverse(num));
            }
        });
        for i in 0..k{
            println!("{:?}",top_k.peek().unwrap().0);
            assert_eq!( top_k.pop().unwrap().0, res[i]);
        }
        
    }

}