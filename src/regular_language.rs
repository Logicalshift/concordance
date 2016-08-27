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

use std::slice::*;

///
/// A phrase iterator can be used to return the symbols in a phrase one at a time
///
pub trait PhraseIterator<Symbol> {
    fn next_symbol(&mut self) -> Option<&Symbol>;
}

///
/// Phrases are sequences of symbols, matched in order
///
pub trait Phrase<Symbol> {
    type PhraseIterator: PhraseIterator<Symbol>;

    ///
    /// Retrieves an iterator that can be used to read the symbols for this phrase
    ///
    fn get_symbols(self) -> Self::PhraseIterator;
}

impl<'a, Symbol> Phrase<Symbol> for &'a Vec<Symbol> {
    type PhraseIterator = Iter<'a, Symbol>;

    #[inline]
    fn get_symbols(self) -> Self::PhraseIterator {
        self.iter()
    }
}

impl<'a, Symbol> PhraseIterator<Symbol> for Iter<'a, Symbol> {
    #[inline]
    fn next_symbol(&mut self) -> Option<&Symbol> {
        self.next()
    }
}

impl<'a, Symbol> Phrase<Symbol> for &'a [Symbol] {
    type PhraseIterator = Iter<'a, Symbol>;

    #[inline]
    fn get_symbols(self) -> Self::PhraseIterator {
        self.iter()
    }
}
