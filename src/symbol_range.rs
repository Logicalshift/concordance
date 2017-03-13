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
//! It's quite common to want to match ranges of symbols (most particularly, 'all' symbols but also things like all upper-case
//! characters). It would be inefficient to store all of the transitions represented by one of these transitions on a per-symbol
//! basis, and impossible for some symbol sets (consider a state machine working on u32s). Instead, transitions are stored as
//! symbol ranges.
//!
//! Symbol ranges are inclusive ranges (unlike Rusts built-in Range type, which is exclusive). This has the advantage that it's
//! possible to represent the entire range of symbols in a particular type: exclusive ranges have to exclude at least one symbol
//! so can never represent the entire range without having to treat it as a special case.
//!

use std::cmp::*;

///
/// Represents a range of symbols
///
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct SymbolRange<Symbol: Ord> {
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

impl<Symbol: Ord> PartialOrd for SymbolRange<Symbol> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Symbol: Ord> Ord for SymbolRange<Symbol> {
    fn cmp(&self, other: &Self) -> Ordering {
        let lower = self.lowest.cmp(&other.lowest);

        if lower == Ordering::Equal {
            self.highest.cmp(&other.highest)
        } else {
            lower
        }
    }
}

impl<Symbol: Ord> SymbolRange<Symbol> {
    ///
    /// Creates a new range covering everything between the specified two symbols
    ///
    #[inline]
    pub fn new(lowest: Symbol, highest: Symbol) -> SymbolRange<Symbol> {
        if lowest > highest {
            panic!("lowest must be <= highest when creating SymbolRanges");
        } else {
            SymbolRange { lowest: lowest, highest: highest }
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

    ///
    /// True if this range contains a symbol
    ///
    #[inline]
    pub fn includes(&self, symbol: &Symbol) -> bool {
        self.lowest <= *symbol && *symbol <= self.highest
    }
}

impl<Symbol: Ord+Clone> SymbolRange<Symbol> {
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_create_range() {
        let range = SymbolRange::new(1, 2);

        assert!(range.lowest == 1);
        assert!(range.highest == 2);
    }

    #[test]
    #[should_panic]
    fn reversing_ranges_panics() {
        SymbolRange::new(5, 1);
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

    #[test]
    fn includes_single_item() {
        let just_zero = SymbolRange::new(0,0);
        assert!(just_zero.includes(&0));        
    }

    #[test]
    fn includes_mid_item() {
        let just_zero = SymbolRange::new(1,4);
        assert!(just_zero.includes(&2));        
    }

    #[test]
    fn includes_first_item() {
        let just_zero = SymbolRange::new(1,4);
        assert!(just_zero.includes(&1));        
    }

    #[test]
    fn includes_last_item() {
        let just_zero = SymbolRange::new(1,4);
        assert!(just_zero.includes(&4));        
    }

    #[test]
    fn excludes_lower_item() {
        let just_zero = SymbolRange::new(1,4);
        assert!(!just_zero.includes(&0));        
    }
    #[test]
    fn excludes_higher_item() {
        let just_zero = SymbolRange::new(1,4);
        assert!(!just_zero.includes(&5));        
    }
}
