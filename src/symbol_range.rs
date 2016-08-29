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
pub trait Countable where Self: Sized {
    ///
    /// Returns the next symbol in order (None if this is the last symbol)
    ///
    fn next(&self) -> Option<Self>;

    ///
    /// Returns the next symbol in order (None if this is the last symbol)
    ///
    fn prev(&self) -> Option<Self>;
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
    /// Creates a new range covering everything between the specified two symbols
    ///
    #[inline]
    pub fn new(lowest: Symbol, highest: Symbol) -> SymbolRange<Symbol> {
        if lowest > highest {
            SymbolRange { lowest: highest, highest: lowest }
        } else {
            SymbolRange { lowest: lowest, highest: highest }
        }
    }

    ///
    /// Joins this range with another
    ///
    /// This creates a new range that covers all the symbols of both. If `overlaps()` is false for these two ranges, then
    /// the new range may cover additional symbols that are not in either range.
    ///
    pub fn join(&self, with: &SymbolRange<Symbol>) -> SymbolRange<Symbol> {
        SymbolRange { 
            lowest:  if with.lowest<self.lowest   { with.lowest.clone()  } else { self.lowest.clone()  },
            highest: if with.highest<self.highest { self.highest.clone() } else { with.highest.clone() }
        }
    }

    ///
    /// True if this range overlaps another
    ///
    #[inline]
    pub fn overlaps(&self, with: &SymbolRange<Symbol>) -> bool {
        if self.highest < with.lowest {
            false
        } else if self.lowest > with.highest {
            false
        } else {
            true
        }
    }
}

impl Countable for u8 {
    fn next(&self) -> Option<u8> {
        if *self == Self::max_value() {
            None
        } else {
            Some(self+1)
        }
    }

    fn prev(&self) -> Option<u8> {
        if *self == Self::min_value() {
            None
        } else {
            Some(self-1)
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

    #[test]
    fn overlaps_when_within() {
        assert!(SymbolRange::new(1, 4).overlaps(&SymbolRange::new(2, 3)));
    }

    #[test]
    fn overlaps_when_without() {
        assert!(SymbolRange::new(2, 3).overlaps(&SymbolRange::new(1, 4)));
    }

    #[test]
    fn overlaps_when_lower() {
        assert!(SymbolRange::new(1, 3).overlaps(&SymbolRange::new(2, 4)));
    }

    #[test]
    fn overlaps_when_higher() {
        assert!(SymbolRange::new(2, 4).overlaps(&SymbolRange::new(1, 3)));
    }

    #[test]
    fn overlaps_when_same() {
        assert!(SymbolRange::new(1, 4).overlaps(&SymbolRange::new(1, 4)));
    }

    #[test]
    fn does_not_overlap_lower() {
        assert!(!SymbolRange::new(1, 2).overlaps(&SymbolRange::new(4, 5)));
    }

    #[test]
    fn does_not_overlap_higher() {
        assert!(!SymbolRange::new(4, 5).overlaps(&SymbolRange::new(1, 2)));
    }

    #[test]
    fn join_left() {
        let joined = SymbolRange::new(1, 3).join(&SymbolRange::new(3, 4));

        assert!(joined.lowest == 1);
        assert!(joined.highest == 4);
    }

    #[test]
    fn join_right() {
        let joined = SymbolRange::new(3, 4).join(&SymbolRange::new(1, 3));

        assert!(joined.lowest == 1);
        assert!(joined.highest == 4);
    }

    #[test]
    fn join_overlap() {
        let joined = SymbolRange::new(1, 4).join(&SymbolRange::new(2, 3));

        assert!(joined.lowest == 1);
        assert!(joined.highest == 4);
    }
}
