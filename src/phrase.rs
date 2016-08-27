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
use std::iter::FromIterator;

///
/// A phrase iterator can be used to return the symbols in a phrase one at a time
///
pub trait PhraseIterator<'a, Symbol> {
    fn next_symbol(&mut self) -> Option<&'a Symbol>;
}

///
/// Phrases are sequences of symbols, matched in order
///
pub trait Phrase<'a, Symbol> {
    type PhraseIterator: PhraseIterator<'a, Symbol>;

    ///
    /// Retrieves an iterator that can be used to read the symbols for this phrase
    ///
    fn get_symbols(&'a self) -> Self::PhraseIterator;
}

impl<'a, Symbol: 'a> Phrase<'a, Symbol> for Vec<Symbol> {
    type PhraseIterator = Iter<'a, Symbol>;

    #[inline]
    fn get_symbols(&'a self) -> Self::PhraseIterator {
        self.iter()
    }
}

impl<'a, Symbol> PhraseIterator<'a, Symbol> for Iter<'a, Symbol> {
    #[inline]
    fn next_symbol(&mut self) -> Option<&'a Symbol> {
        self.next()
    }
}

/*
impl<'a, Symbol> Phrase<Symbol> for &'a [Symbol] {
    type PhraseIterator = Iter<'a, Symbol>;

    #[inline]
    fn get_symbols(self) -> Self::PhraseIterator {
        self.iter()
    }
}

impl<'a> Phrase<char> for &'a str {
    type PhraseIterator = StringPhraseIterator;

    #[inline]
    fn get_symbols(self) -> Self::PhraseIterator {
        StringPhraseIterator { index: 0, string: Vec::from_iter(self.chars()) }
    }
}

///
/// Phrase iterator that goes over a string
///
pub struct StringPhraseIterator {
    /// Where we've reached in the string
    index: usize,

    /// The string that this iterator will cover
    string: Vec<char>
}

impl PhraseIterator<char> for StringPhraseIterator {
    fn next_symbol(&mut self) -> Option<&char> {
        if self.index >= self.string.len() {
            None
        } else {
            let result = Some(&self.string[self.index]);
            self.index += 1;

            result
        }
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_iterate_vector_phrase() {
        let some_phrase     = vec![1, 2, 3];
        let mut iterator    = some_phrase.get_symbols();

        assert!(Some(&1) == iterator.next_symbol());
        assert!(Some(&2) == iterator.next_symbol());
        assert!(Some(&3) == iterator.next_symbol());
        assert!(iterator.next_symbol() == None);
    }

    /*
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

        assert!(iterator.next_symbol() == Some(&'A'));
        assert!(iterator.next_symbol() == Some(&'B'));
        assert!(iterator.next_symbol() == Some(&'C'));
        assert!(iterator.next_symbol() == None);
    }
    */
}
