//
//   Copyright 2016 Andrew Hunter
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.
//

use std::cmp::Ordering;

use super::symbol_range::*;
use super::countable::*;

///
/// A symbol map maps from one set of symbol ranges to another
///
pub struct SymbolMap<Symbol: PartialOrd+Clone+Countable> {
    // Ranges in this symbol map
    ranges: Vec<SymbolRange<Symbol>>,
}

impl<Symbol: PartialOrd+Clone+Countable> SymbolMap<Symbol> {
    ///
    /// Creates a new symbol map
    ///
    pub fn new() -> SymbolMap<Symbol> {
        SymbolMap { ranges: vec![] }
    }

    ///
    /// Orders two symbols (makes unordered symbols equal)
    ///
    #[inline]
    fn order_symbols(a: &Symbol, b: &Symbol) -> Ordering {
        if let Some(order) = a.partial_cmp(b) {
            order
        } else {
            Ordering::Equal
        }
    }

    ///
    /// Orders two symbol ranges
    ///
    #[inline]
    fn order_ranges(a: &SymbolRange<Symbol>, b: &SymbolRange<Symbol>) -> Ordering {
        let ordering = SymbolMap::order_symbols(&a.lowest, &b.lowest);

        if ordering == Ordering::Equal {
            SymbolMap::order_symbols(&b.highest, &a.highest)
        } else {
            ordering
        }
    }

    ///
    /// Adds a range to those that are known about by this object
    ///
    pub fn add_range(&mut self, range: &SymbolRange<Symbol>) {
        let existing = self.ranges.binary_search_by(|test_range| { SymbolMap::order_ranges(test_range, range) });

        // Insert the range if it is not already in the map
        if let Err(insertion_pos) = existing {
            self.ranges.insert(insertion_pos, range.clone());
        }
    }

    ///
    /// Finds the ranges in this map that overlap the target ranges
    ///
    pub fn find_overlapping_ranges(&self, range: &SymbolRange<Symbol>) -> Vec<&SymbolRange<Symbol>> {
        let mut result = vec![];

        // Find the first range that matches (or the insertion position, which should be the first range wit a lowest value higher than the target range)
        let existing = self.ranges.binary_search_by(|test_range| { SymbolMap::order_ranges(test_range, range) });

        // Start returning values from here
        let mut pos = match existing {
            Ok(found_position) => found_position,
            Err(insert_position) => insert_position
        };

        // Move backwards if the previous position overlaps this one
        if pos > 0 && self.ranges[pos-1].highest >= range.lowest {
            pos -= 1;
        }

        // TODO: can we construct a set of ranges such that one is missed here? Think maybe we can
        while pos < self.ranges.len() && self.ranges[pos].lowest <= range.highest {
            result.push(&self.ranges[pos]);
            pos += 1;
        }

        result
    }

    ///
    /// Creates a non-overlapping range from an overlapping one
    ///
    pub fn to_non_overlapping_map(&self) -> SymbolMap<Symbol> {
        // Stack, popping the lowest ordered ranges first
        let mut to_process = self.ranges.clone();
        to_process.reverse();

        let mut result = vec![];

        while let Some(might_overlap) = to_process.pop() {
            if let Some(overlap_with) = to_process.pop() {
                // Stack has two ranges on top. They might overlap
                if !might_overlap.overlaps(&overlap_with) {
                    // Doesn't overlap: can just push might_overlap and continue
                    result.push(might_overlap);
                    to_process.push(overlap_with);
                } else {
                    // Got an overlap
                    if might_overlap == overlap_with {
                        // Ranges are the same, just discard one
                        to_process.push(overlap_with);
                    } else if might_overlap.lowest == overlap_with.lowest {
                        // Ranges start at the same location. We need to divide them in case more than two ranges are overlapping
                        let (smaller_range, larger_range) = (overlap_with, might_overlap);      // Because of the sort order

                        // Chop out the smaller range from the larger range, then insert into the stack in order
                        let larger_range_without_smaller_range = SymbolRange::new(smaller_range.highest.next(), larger_range.highest.clone());

                        to_process.push(smaller_range);

                        if let Err(insertion_pos) = to_process.binary_search_by(|test_range| { SymbolMap::order_ranges(&larger_range_without_smaller_range, test_range) }) {
                            to_process.insert(insertion_pos, larger_range_without_smaller_range);
                        }
                    } else {
                        // There's a range from the lowest of the first range to the lowest of the second ranges
                        result.push(SymbolRange::new(might_overlap.lowest.clone(), overlap_with.lowest.prev()));

                        // Chop out the bit we just pushed from might_overlap and push back both ranges
                        to_process.push(overlap_with.clone());
                        to_process.push(SymbolRange::new(overlap_with.lowest, might_overlap.highest));
                    }
                }
            } else {
                // Last range should never overlap
                result.push(might_overlap);
            }
        }

        // Ranges should already be sorted as we worked from left to right
        SymbolMap { ranges: result }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::symbol_range::*;

    #[test]
    fn can_lookup_overlapping_ranges() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 4));
        map.add_range(&SymbolRange::new(2, 5));
        map.add_range(&SymbolRange::new(3, 6));

        let bottom = map.find_overlapping_ranges(&SymbolRange::new(0, 1));
        let all    = map.find_overlapping_ranges(&SymbolRange::new(1, 3));
        let top    = map.find_overlapping_ranges(&SymbolRange::new(6, 6));

        assert!(bottom == vec![&SymbolRange::new(0, 4)]);
        assert!(all == vec![&SymbolRange::new(0, 4), &SymbolRange::new(2, 5), &SymbolRange::new(3, 6)]);
        assert!(top == vec![&SymbolRange::new(3, 6)]);
    }

    #[test]
    fn can_lookup_mid_range() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 4));
        map.add_range(&SymbolRange::new(5, 10));
        map.add_range(&SymbolRange::new(11, 15));

        let mid = map.find_overlapping_ranges(&SymbolRange::new(1, 3));

        assert!(mid == vec![&SymbolRange::new(0, 4)]);
    }

    #[test]
    fn works_with_duplicate_lower_values() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 2));
        map.add_range(&SymbolRange::new(0, 3));

        let all    = map.find_overlapping_ranges(&SymbolRange::new(0, 1));

        assert!(all == vec![&SymbolRange::new(0, 2), &SymbolRange::new(0, 3)]);
    }

    #[test]
    fn can_get_non_overlapping_map() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 4));
        map.add_range(&SymbolRange::new(2, 5));
        map.add_range(&SymbolRange::new(3, 6));

        let non_overlapping = map.to_non_overlapping_map();

        let all = non_overlapping.find_overlapping_ranges(&SymbolRange::new(0, 6));

        assert!(all == vec![&SymbolRange::new(0, 1), &SymbolRange::new(2, 2), &SymbolRange::new(3, 4), &SymbolRange::new(5, 5), &SymbolRange::new(6, 6)]);
    }

    #[test]
    fn can_get_non_overlapping_map_with_single_symbols() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 5));
        map.add_range(&SymbolRange::new(2, 2));
        map.add_range(&SymbolRange::new(3, 6));

        let non_overlapping = map.to_non_overlapping_map();

        let all = non_overlapping.find_overlapping_ranges(&SymbolRange::new(0, 6));

        assert!(all == vec![&SymbolRange::new(0, 1), &SymbolRange::new(2, 2), &SymbolRange::new(3, 5), &SymbolRange::new(6, 6)]);
    }

    #[test]
    fn generate_correctly_for_single_symbol() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 0));

        let non_overlapping = map.to_non_overlapping_map();

        let all = non_overlapping.find_overlapping_ranges(&SymbolRange::new(0, 10));

        assert!(all == vec![&SymbolRange::new(0, 0)]);
    }

    #[test]
    fn generate_correctly_for_two_single_symbols() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 0));
        map.add_range(&SymbolRange::new(1, 1));

        let non_overlapping = map.to_non_overlapping_map();

        let all = non_overlapping.find_overlapping_ranges(&SymbolRange::new(0, 10));

        assert!(all == vec![&SymbolRange::new(0, 0), &SymbolRange::new(1, 1)]);
    }

    #[test]
    fn generate_correctly_for_non_overlapping_ranges() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 1));
        map.add_range(&SymbolRange::new(2, 4));

        let non_overlapping = map.to_non_overlapping_map();

        let all = non_overlapping.find_overlapping_ranges(&SymbolRange::new(0, 10));

        assert!(all == vec![&SymbolRange::new(0, 1), &SymbolRange::new(2, 4)]);
    }

    #[test]
    fn generate_correctly_for_single_overlap() {
        let mut map = SymbolMap::new();

        // Here the symbol '5' is in both ranges, so we should generate it as a seperate range in the non-overlapping version
        map.add_range(&SymbolRange::new(0, 5));
        map.add_range(&SymbolRange::new(5, 10));

        let non_overlapping = map.to_non_overlapping_map();

        let all = non_overlapping.find_overlapping_ranges(&SymbolRange::new(0, 10));
        println!("{:?}", all);

        assert!(all == vec![&SymbolRange::new(0, 4), &SymbolRange::new(5,5), &SymbolRange::new(6, 10)]);
    }

    #[test]
    fn can_get_non_overlapping_map_with_single_symbols_at_start() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 0));
        map.add_range(&SymbolRange::new(0, 1));

        let non_overlapping = map.to_non_overlapping_map();

        let all = non_overlapping.find_overlapping_ranges(&SymbolRange::new(0, 6));

        assert!(all == vec![&SymbolRange::new(0, 0), &SymbolRange::new(1, 1)]);
    }

    #[test]
    fn can_get_non_overlapping_map_with_single_symbols_at_start_and_gap() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 0));
        map.add_range(&SymbolRange::new(0, 1));
        map.add_range(&SymbolRange::new(3, 6));

        let non_overlapping = map.to_non_overlapping_map();

        let all = non_overlapping.find_overlapping_ranges(&SymbolRange::new(0, 6));

        assert!(all == vec![&SymbolRange::new(0, 0), &SymbolRange::new(1, 1), &SymbolRange::new(3, 6)]);
    }
}
