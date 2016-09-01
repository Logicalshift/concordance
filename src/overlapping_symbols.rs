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

///
/// A symbol map maps from one set of symbol ranges to another
///
pub struct SymbolMap<Symbol: PartialOrd+Clone> {
    // Ranges in this symbol map
    ranges: Vec<SymbolRange<Symbol>>,
}

impl<Symbol: PartialOrd+Clone> SymbolMap<Symbol> {
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
        let mut range_symbols: Vec<Symbol> = self.ranges.iter()
            .map(|r| r.lowest.clone())
            .chain(self.ranges.iter().map(|r| r.highest.clone()))
            .collect();

        // Sort and dedupe (we'll get pairs of ranges)
        range_symbols.sort_by(SymbolMap::order_symbols);
        range_symbols.dedup();

        // Generate a new symbol map with these ranges
        // They'll overlap with the last value, which should be OK if following the rules for adjacent ranges in symbol_range.rs
        let mut result = vec![];
        for index in 0..range_symbols.len()-1 {
            result.push(SymbolRange::new(range_symbols[index].clone(), range_symbols[index+1].clone()));
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

        println!("All is: {:?}", all);

        assert!(bottom == vec![&SymbolRange::new(0, 4)]);
        assert!(all == vec![&SymbolRange::new(0, 4), &SymbolRange::new(2, 5), &SymbolRange::new(3, 6)]);
        assert!(top == vec![&SymbolRange::new(6, 6)]);
    }

    #[test]
    fn can_lookup_mid_range() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 4));
        map.add_range(&SymbolRange::new(5, 10));
        map.add_range(&SymbolRange::new(11, 15));

        let bottom = map.find_overlapping_ranges(&SymbolRange::new(1, 3));

        assert!(bottom == vec![&SymbolRange::new(0, 4)]);
    }

    #[test]
    fn works_with_duplicate_lower_values() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 2));
        map.add_range(&SymbolRange::new(0, 3));

        let all    = map.find_overlapping_ranges(&SymbolRange::new(0, 1));

        println!("With duplicate lower values is: {:?}", all);

        assert!(all == vec![&SymbolRange::new(0, 2), &SymbolRange::new(0, 3)]);
    }

    #[test]
    fn obeys_adjacency_rule() {
        let mut map = SymbolMap::new();

        // By the adjacency rule for symbol ranges (in symbol_range.rs), '4' here is only in the upper range
        map.add_range(&SymbolRange::new(0, 4));
        map.add_range(&SymbolRange::new(4, 8));

        let top = map.find_overlapping_ranges(&SymbolRange::new(4, 4));

        assert!(top == vec![&SymbolRange::new(4, 8)]);
    }

    #[test]
    fn can_get_non_overlapping_map() {
        let mut map = SymbolMap::new();

        map.add_range(&SymbolRange::new(0, 4));
        map.add_range(&SymbolRange::new(2, 5));
        map.add_range(&SymbolRange::new(3, 6));

        let non_overlapping = map.to_non_overlapping_map();

        let all = non_overlapping.find_overlapping_ranges(&SymbolRange::new(0, 6));

        assert!(all == vec![&SymbolRange::new(0, 2), &SymbolRange::new(2, 3), &SymbolRange::new(3, 4), &SymbolRange::new(4, 5), &SymbolRange::new(5, 6)]);
    }
}
