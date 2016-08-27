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

use super::phrase::*;
use std::ops::Range;

///
/// A Pattern represents a matching pattern in a regular language
///
#[derive(Clone)]
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
