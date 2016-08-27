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
//! # Phrases
//!
//! A phrase is a string of symbols; it can be input to a DFA or part of an expression in a regular language.
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

impl<'a> Phrase<u8> for &'a str {
    type PhraseIterator = StringPhraseIterator<'a>;

    #[inline]
    fn get_symbols(self) -> Self::PhraseIterator {
        StringPhraseIterator { index: 0, string: self.as_bytes() }
    }
}

///
/// Phrase iterator that goes over a string
///
pub struct StringPhraseIterator<'a> {
    /// Where we've reached in the string
    index: usize,

    /// The string that this iterator will cover
    string: &'a [u8]
}

impl<'a> PhraseIterator<u8> for StringPhraseIterator<'a> {
    fn next_symbol(&mut self) -> Option<&u8> {
        if self.index >= self.string.len() {
            None
        } else {
            let result = Some(&self.string[self.index]);
            self.index += 1;

            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_iterate_vector_phrase() {
        let some_phrase     = vec![1, 2, 3];
        let mut iterator    = some_phrase.get_symbols();

        assert!(iterator.next_symbol() == Some(&1));
        assert!(iterator.next_symbol() == Some(&2));
        assert!(iterator.next_symbol() == Some(&3));
        assert!(iterator.next_symbol() == None);
    }

    #[test]
    fn can_iterate_array_phrase() {
        let some_phrase     = [1, 2, 3];
        let mut iterator    = some_phrase.get_symbols();

        assert!(iterator.next_symbol() == Some(&1));
        assert!(iterator.next_symbol() == Some(&2));
        assert!(iterator.next_symbol() == Some(&3));
        assert!(iterator.next_symbol() == None);
    }

    #[test]
    fn can_iterate_string_phrase() {
        let some_phrase     = "ABC";
        let mut iterator    = some_phrase.get_symbols();

        assert!(iterator.next_symbol() == Some(&65));
        assert!(iterator.next_symbol() == Some(&66));
        assert!(iterator.next_symbol() == Some(&67));
        assert!(iterator.next_symbol() == None);
    }
}
