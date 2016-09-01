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
            SymbolMap::order_symbols(&a.highest, &b.highest)
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
        // Create a vector of all the range symbols
        let mut lowest:  Vec<Symbol> = self.ranges.iter().map(|r| r.lowest.clone()).collect();
        let mut highest: Vec<Symbol> = self.ranges.iter().map(|r| r.highest.clone()).collect();

        // Dedupe the lowest and highest symbols seperately
        lowest.sort_by(SymbolMap::order_symbols);
        lowest.dedup();

        highest.sort_by(SymbolMap::order_symbols);
        highest.dedup();

        // Combine into the set of ranges (each pair representing a range in the result)
        let mut range_symbols: Vec<Symbol> = lowest.into_iter().chain(highest.into_iter()).collect();
        range_symbols.sort_by(SymbolMap::order_symbols);

        // Generate a new symbol map with these ranges
        let mut result = vec![];
        for index in 0..range_symbols.len()-2 {
            result.push(SymbolRange::new(range_symbols[index].clone(), range_symbols[index+1].prev()));
        }

        if range_symbols.len() > 1 {
            // Last item is always inclusive
            result.push(SymbolRange::new(range_symbols[range_symbols.len()-2].clone(), range_symbols[range_symbols.len()-1].clone()));
        }

        // We already sorted everything, so bypass the usual 'add' method (which sorts as it goes)
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

        assert!(all == vec![&SymbolRange::new(0, 1), &SymbolRange::new(2, 2), &SymbolRange::new(3, 3), &SymbolRange::new(4, 4), &SymbolRange::new(5, 6)]);
    }

    #[test]
    fn can_get_non_overlapping_map_with_single_symbols() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 5));
        map.add_range(&SymbolRange::new(2, 2));
        map.add_range(&SymbolRange::new(3, 6));

        let non_overlapping = map.to_non_overlapping_map();

        let all = non_overlapping.find_overlapping_ranges(&SymbolRange::new(0, 6));

        assert!(all == vec![&SymbolRange::new(0, 1), &SymbolRange::new(2, 2), &SymbolRange::new(3, 4), &SymbolRange::new(5, 6)]);
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

        assert!(all == vec![&SymbolRange::new(0, 0), &SymbolRange::new(1, 1), &SymbolRange::new(2, 2), &SymbolRange::new(3, 6)]);
    }
}
