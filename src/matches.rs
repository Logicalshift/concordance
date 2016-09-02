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

use super::symbol_range_dfa::*;
use super::symbol_reader::*;
use super::pattern_matcher::*;
use super::prepare::*;
use super::countable::*;

///
/// Runs a pattern matcher against a stream, and returns the number of characters matching if it accepted the stream
///
fn matches_symbol_range<'a, InputSymbol: PartialOrd, OutputSymbol: 'a, Matcher>(dfa: &'a Matcher, symbol_reader: &mut SymbolReader<InputSymbol>) -> Option<usize>
where Matcher: PatternMatcher<'a, InputSymbol, OutputSymbol> {
    // Run until there are no more states
    let mut current_state = dfa.start();

    while let More(this_state) = current_state {
        let next_state = 
            if let Some(next_char) = symbol_reader.next_symbol() {
                this_state.next(next_char)
            } else {
                this_state.finish()
            };

        current_state = next_state;
    }

    if let Accept(count, _) = current_state {
        Some(count)
    } else {
        None
    }
}

///
/// Trait implemented by types that can be matched against a pattern
///
pub trait Matches<Pattern> {
    ///
    /// Matches a pattern against a source, and returns the number of characters matching if it accepted the stream
    ///
    fn matches(src: Self, pattern: Pattern) -> Option<usize>
        where Self: Sized, Pattern: Sized;
}

impl<'a, InputSymbol: PartialOrd, OutputSymbol: 'a> Matches<&'a SymbolRangeDfa<InputSymbol, OutputSymbol>> for &'a mut SymbolReader<InputSymbol> {
    fn matches(src: &'a mut SymbolReader<InputSymbol>, pattern: &'a SymbolRangeDfa<InputSymbol, OutputSymbol>) -> Option<usize> {
        matches_symbol_range(pattern, src)
    }
}

impl<'a, OutputSymbol: 'a> Matches<&'a SymbolRangeDfa<char, OutputSymbol>> for &'a str {
    fn matches(src: &'a str, pattern: &'a SymbolRangeDfa<char, OutputSymbol>) -> Option<usize> {
        matches_symbol_range(pattern, &mut src.read_symbols())
    }
}
