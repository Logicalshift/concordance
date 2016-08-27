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
//! # Regular language
//!
//! This provides a data structure representing a regular language. This is a more generalised form of a regular
//! expression (it supports symbol types other than simple strings). A regular language using the `u8` symbol forms
//! a simple regular expression.
//!

use std::iter::FromIterator;
use std::ops::Range;

///
/// A Pattern represents a matching pattern in a regular language
///
#[derive(Clone, PartialEq, Eq)]
pub enum Pattern<Symbol> {
    ///
    /// Matches nothing
    ///
    Epsilon,

    ///
    /// Matches a specific literal phrase
    ///
    Match(Vec<Symbol>),

    ///
    /// Matches at least a particular number of repetitions of a pattern
    ///
    /// `RepeatInfinite(0, X)` is the equivalent of the regular expression `X*`, `RepeatInfinite(1, X)` is the equivalent of the regular expression `X+`
    ///
    RepeatInfinite(u32, Box<Pattern<Symbol>>),

    ///
    /// Matches a range of repetitions of a pattern
    ///
    Repeat(Range<u32>, Box<Pattern<Symbol>>),

    ///
    /// Matches a set of sub-patterns in order
    ///
    MatchAll(Vec<Pattern<Symbol>>),

    ///
    /// Matches any one of a set of patterns
    ///
    MatchAny(Vec<Pattern<Symbol>>)
}

pub use Pattern::*;

///
/// Implemented by things that can be converted into a pattern
///
pub trait ToPattern<Symbol> {
    ///
    /// Converts a particular object into a pattern that will match it
    ///
    fn to_pattern(&self) -> Pattern<Symbol>;
}

impl<Symbol: Clone> ToPattern<Symbol> for Pattern<Symbol> {
    fn to_pattern(&self) -> Pattern<Symbol> {
        self.clone()
    }
}

impl<Symbol: Clone> ToPattern<Symbol> for Box<Pattern<Symbol>> {
    fn to_pattern(&self) -> Pattern<Symbol> {
        (**self).clone()
    }
}

impl<Symbol: Clone> ToPattern<Symbol> for Vec<Symbol> {
    fn to_pattern(&self) -> Pattern<Symbol> {
        Match(self.clone())
    }
}

impl<Symbol: Clone> ToPattern<Symbol> for [Symbol] {
    fn to_pattern(&self) -> Pattern<Symbol> {
        Match(self.to_vec())
    }
}

impl ToPattern<char> for str {
    fn to_pattern(&self) -> Pattern<char> {
        Match(Vec::from_iter(self.chars()))
    }
}

///
/// Implemented by things that can build up patterns
///
/// Patterns are built in boxes to avoid the need for a lot of copying
///
pub trait PatternBuilder<Symbol> {
    /// Creates an empty pattern
    fn empty() -> Pattern<Symbol>;

    /// Appends a pattern to this one
    fn append(self, pattern: ToPattern<Symbol>) -> Pattern<Symbol>;

    /// Repeats the current pattern forever
    fn repeat_forever(self, min_count: u32) -> Pattern<Symbol>;

    /// Repeats the current pattern for a certain number of iterations
    fn repeat(self, count: Range<u32>) -> Pattern<Symbol>;
}

impl<Symbol> Pattern<Symbol> {

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_convert_vec_to_pattern() {
        let pattern = vec![0, 1, 2].to_pattern();

        assert!(pattern == Match(vec![0, 1, 2]));
    }

    #[test]
    fn can_convert_array_to_pattern() {
        let pattern = [0, 1, 2].to_pattern();

        assert!(pattern == Match(vec![0, 1, 2]));
    }

    #[test]
    fn can_convert_string_to_pattern() {
        let pattern = "abc".to_pattern();

        assert!(pattern == Match(vec!['a', 'b', 'c']));
    }
}