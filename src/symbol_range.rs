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

//!
//! # Symbol range
//!
//! It's quite common to want to match ranges of symbols (most particularly, 'all' symbols but also things like all upper-case
//! characters). It would be inefficient to store all of the transitions represented by one of these transitions on a per-symbol
//! basis, and impossible for some symbol sets (consider a state machine working on u32s). Instead, transitions are stored as
//! symbol ranges.
//!

///
/// Trait implemented by symbol types that are countable - ie, for types where there's always a next symbol
///
pub trait Countable<Symbol> {
    ///
    /// Returns the next symbol in order (None if this is the last symbol)
    ///
    fn next(&self) -> Option<Symbol>;

    ///
    /// Returns the next symbol in order (None if this is the last symbol)
    ///
    fn prev(&self) -> Option<Symbol>;
}

///
/// Represents a range of symbols
///
/// Symbols must be ordered in order to use them with a range-based state machine.
///
#[derive(Clone, Eq, PartialEq)]
pub struct SymbolRange<Symbol: Ord+Clone> {
    ///
    /// Lowest symbol in the range
    ///
    pub lowest: Symbol,

    ///
    /// Highest symbol in the range
    ///
    /// This is inclusive, so the highest symbol is always included in the range (this makes it differ from Rust's 
    /// built-in Range struct, and is important for supporting uncountable symbols)
    ///
    pub highest: Symbol
}

impl<Symbol: Ord+Clone> SymbolRange<Symbol> {
    ///
    /// Creates a new range covering
    ///
    #[inline]
    pub fn new(lowest: Symbol, highest: Symbol) -> SymbolRange<Symbol> {
        if lowest > highest {
            SymbolRange { lowest: highest, highest: lowest }
        } else {
            SymbolRange { lowest: lowest, highest: highest }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_range() {
        let range = SymbolRange::new(1, 2);

        assert!(range.lowest == 1);
        assert!(range.highest == 2);
    }

    #[test]
    fn can_create_range_reversed() {
        let range = SymbolRange::new(5, 1);

        assert!(range.lowest == 1);
        assert!(range.highest == 5);
    }
}
