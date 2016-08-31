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
    /// Adds a range to those that are known about by this object
    ///
    pub fn add_range(&mut self, range: &SymbolRange<Symbol>) {
        let existing = self.ranges.binary_search_by(|test_range| { SymbolMap::order_symbols(&range.lowest, &test_range.lowest) });

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
        let existing = self.ranges.binary_search_by(|test_range| { SymbolMap::order_symbols(&range.lowest, &test_range.lowest) });

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
}
