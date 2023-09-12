// This code solves the following puzzle:
// You are given 8 batteries but only 4 of them are functional. You have a toy that needs 2
// functional batteries. You have 7 tries to turn on the toy.

use std::fmt;
use std::ops::BitAnd;

// A small set for storing integers 0..=63
#[derive(Clone, Copy, Debug, PartialEq)]
struct BitSet(u64);

impl From<u64> for BitSet {
    fn from(val: u64) -> BitSet {
        BitSet(val)
    }
}

impl BitAnd for BitSet {
    type Output = BitSet;
    fn bitand(self, other: BitSet) -> Self::Output {
        BitSet(self.0 & other.0)
    }
}

impl BitSet {
    fn len(&self) -> u32 {
        self.0.count_ones()
    }
}

impl fmt::Display for BitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:b}", self.0)
    }
}

impl IntoIterator for BitSet {
    type Item = usize;
    type IntoIter = BitSetIter;
    fn into_iter(self) -> Self::IntoIter {
        BitSetIter(self.0)
    }
}

struct BitSetIter(u64);
impl Iterator for BitSetIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }
        let v = self.0.trailing_zeros();
        self.0 ^= 1 << v;
        Some(v as usize)
    }
}

// This iterator uses bit tricks to iterate over n-choose-k combinations.
struct CombinationIter {
    next_val: u64,
    n: u64,
}

impl CombinationIter {
    fn new(n: u64, k: u64) -> Self {
        debug_assert!(n >= k, "k must be smaller than n");
        debug_assert!(n <= 65, "only n up to 64 is supported");
        debug_assert!(k > 0, "only positive k is supported");

        let k_trailing_ones = (1 << k) - 1;

        Self {
            next_val: k_trailing_ones,
            n,
        }
    }
}

// This iterator uses bit tricks to iterate over n-choose-k combinations.
// The initial value of next_val is 00...01..11 (k trailing 1s). To move from one combination to
// another we identify the right-most cluster of ones and we shift the cluster's leading bit to the
// left by one and all other cluster's bits are shifted to least significant positions. For
// example:
// xxxx01110000 has cluster 111 and so next state is xxxx10000011
impl Iterator for CombinationIter {
    type Item = BitSet;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_val == 0 {
            return None;
        }

        let val = self.next_val;

        // 1. Get least significant 1-bit (last bit of cluster)
        let one_bit = val & (1 + !val);

        // 2. By adding the least significant 1-bit to current state we effectively turn all of
        //    cluster's bits from 1s to 0s, except for the leftmost bit which gets shifted to the
        //    left by one. If that bit is not within the rightmost N bits, then we ran out of
        //    combinations. All the other cluster's bits will be moved to rightmost positions in
        //    next step.
        self.next_val = match val.checked_add(one_bit) {
            // 3. x ^ val gives us the cluster of 1s with an extra 1 prepended. We shift if to the
            //    right and lose 2 1-bits because the cluster was 1-bit larger, and also because we
            //    only want to right shift all but the leftmost cluster's bit.
            Some(x) if x < (1 << self.n) => x | ((x ^ val) >> (one_bit.trailing_zeros() + 2)),
            Some(_) | None => 0,
        };

        Some(val.into())
    }
}

fn remove_impossible_universes(pair: BitSet, mut universes: Vec<BitSet>) -> Vec<BitSet> {
    let mut i = 0;
    while i < universes.len() {
        if universes[i] & pair == pair {
            // in this universe both batteries worked
            universes.swap_remove(i);
        } else {
            i += 1;
        }
    }
    universes
}

fn main() {
    let all_battery_pairs: Vec<_> = CombinationIter::new(8, 2).collect();

    // WLOG we can assume that the first battery pair is part of solution
    let all_battery_universes: Vec<_> =
        remove_impossible_universes(all_battery_pairs[0], CombinationIter::new(8, 4).collect());

    // Next we try all possible quintuples of battery pairs and assume each pair in a quintuple
    // will not turn on the toy. After that we have used up 6 tries (the quintuple and the one
    // above), so all that remains is to check if all remaining "universes" contain a battery pair that is functional in each one.
    let all_five_steps = CombinationIter::new(all_battery_pairs.len() as u64, 5);
    for five_steps in all_five_steps {
        let mut all_battery_universes = all_battery_universes.clone();
        for pair in five_steps {
            all_battery_universes =
                remove_impossible_universes(all_battery_pairs[pair], all_battery_universes);
        }

        match all_battery_universes
            .iter()
            .cloned()
            .reduce(|acc, v| acc & v)
        {
            Some(x) if x.len() >= 2 => {
                print!(
                    "Solution: {:?}",
                    all_battery_pairs[0].into_iter().collect::<Vec<_>>()
                );
                for pair in five_steps {
                    print!(
                        " {:?}",
                        all_battery_pairs[pair].into_iter().collect::<Vec<_>>()
                    );
                }
                println!(" {:?}", x.into_iter().collect::<Vec<_>>());
            }
            Some(_) | None => {}
        };
    }
}
