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

///
/// Implemented by things that can be converted into a pattern
///
pub trait IntoPattern<Symbol> {
    ///
    /// Converts a particular object into a pattern that will match it
    ///
    fn into_pattern(self) -> Pattern<Symbol>;
}

impl<Symbol> IntoPattern<Symbol> for Pattern<Symbol> {
    #[inline]
    fn into_pattern(self) -> Pattern<Symbol> {
        self
    }
}

impl<Symbol> IntoPattern<Symbol> for Box<Pattern<Symbol>> {
    #[inline]
    fn into_pattern(self) -> Pattern<Symbol> {
        *self
    }
}

impl<'a, Symbol: Clone, PatternType: ToPattern<Symbol>> IntoPattern<Symbol> for &'a PatternType {
    #[inline]
    fn into_pattern(self) -> Pattern<Symbol> {
        self.to_pattern()
    }
}

impl<'a, Symbol: Clone> IntoPattern<Symbol> for &'a [Symbol] {
    #[inline]
    fn into_pattern(self) -> Pattern<Symbol> {
        self.to_pattern()
    }
}

impl<'a> IntoPattern<char> for &'a str {
    #[inline]
    fn into_pattern(self) -> Pattern<char> {
        self.to_pattern()
    }
}

impl<Symbol: Clone> ToPattern<Symbol> for Pattern<Symbol> {
    #[inline]
    fn to_pattern(&self) -> Pattern<Symbol> {
        self.clone()
    }
}

impl<Symbol: Clone> ToPattern<Symbol> for Box<Pattern<Symbol>> {
    #[inline]
    fn to_pattern(&self) -> Pattern<Symbol> {
        (**self).clone()
    }
}

impl<Symbol: Clone> ToPattern<Symbol> for Vec<Symbol> {
    #[inline]
    fn to_pattern(&self) -> Pattern<Symbol> {
        Match(self.clone())
    }
}

impl<Symbol: Clone> ToPattern<Symbol> for [Symbol] {
    #[inline]
    fn to_pattern(&self) -> Pattern<Symbol> {
        Match(self.to_vec())
    }
}

impl ToPattern<char> for str {
    #[inline]
    fn to_pattern(&self) -> Pattern<char> {
        Match(Vec::from_iter(self.chars()))
    }
}

///
/// Implemented by things that can modify existing patterns into other forms
///
/// Pattern transformers act by altering a single pattern object into a new form
///
pub trait PatternTransformer<Symbol> {
    /// Repeats the current pattern forever
    fn repeat_forever(self, min_count: u32) -> Pattern<Symbol>;

    /// Repeats the current pattern for a certain number of iterations
    fn repeat(self, count: Range<u32>) -> Pattern<Symbol>;
}

///
/// Implemented by things that combine patterns together to create new patterns
///
pub trait PatternCreator<Symbol> {
    /// Appends a pattern to this one
    fn append(self, pattern: &ToPattern<Symbol>) -> Pattern<Symbol>;

    /// Matches either this pattern or the specified pattern
    fn or(self, pattern: &ToPattern<Symbol>) -> Pattern<Symbol>;
}

impl<Symbol> Pattern<Symbol> {

}

impl<Symbol, PatternType: IntoPattern<Symbol>> PatternTransformer<Symbol> for PatternType {
    fn repeat_forever(self, min_count: u32) -> Pattern<Symbol> {
        RepeatInfinite(min_count, Box::new(self.into_pattern()))
    }

    fn repeat(self, count: Range<u32>) -> Pattern<Symbol> {
        Repeat(count, Box::new(self.into_pattern()))
    }
}

impl<Symbol, PatternType: IntoPattern<Symbol>> PatternCreator<Symbol> for PatternType {
    fn append(self, pattern: &ToPattern<Symbol>) -> Pattern<Symbol> {
        MatchAll(vec![self.into_pattern(), pattern.to_pattern()])
    }

    fn or(self, pattern: &ToPattern<Symbol>) -> Pattern<Symbol> {
        MatchAny(vec![self.into_pattern(), pattern.to_pattern()])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_convert_vec_to_pattern() {
        let pattern = vec![0, 1, 2].into_pattern();

        assert!(pattern == Match(vec![0, 1, 2]));
    }

    #[test]
    fn can_convert_array_to_pattern() {
        let pattern = [0, 1, 2].into_pattern();

        assert!(pattern == Match(vec![0, 1, 2]));
    }

    #[test]
    fn can_convert_string_to_pattern() {
        let pattern = "abc".into_pattern();

        assert!(pattern == Match(vec!['a', 'b', 'c']));
    }
}