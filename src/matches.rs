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
//! # Matches
//!
//! ```
//! # use ndfa::*;
//! # assert!(matches("abcabc", "abc".repeat_forever(1)).is_some());
//! # assert!(matches("abcabcabc", "abc".repeat_forever(1)).is_some());
//! # assert!(matches("abc", "abc").is_some());
//! # assert!(matches("def", "abc".repeat_forever(1)).is_none());
//! if matches("abcabc", "abc".repeat_forever(1)).is_some() { /* ... */ }
//! ```

use super::symbol_range_dfa::*;
use super::symbol_reader::*;
use super::pattern_matcher::*;
use super::prepare::*;

///
/// Runs a pattern matcher against a stream, and returns the number of characters matching if it accepted the stream
///
fn matches_symbol_range<InputSymbol: Ord, OutputSymbol: 'static>(dfa: &SymbolRangeDfa<InputSymbol, OutputSymbol>, symbol_reader: &mut SymbolReader<InputSymbol>) -> Option<usize> {
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
/// Matches a source stream against a pattern
///
pub fn matches<'a, Symbol, OutputSymbol, Prepare, Reader, Source>(source: Source, pattern: Prepare) -> Option<usize>
where   Prepare: PrepareToMatch<SymbolRangeDfa<Symbol, OutputSymbol>>
,       Reader: SymbolReader<Symbol>+'a
,       Source: SymbolSource<'a, Symbol, SymbolReader=Reader>
,       Symbol: Ord
,       OutputSymbol: 'static {
    let matcher    = pattern.prepare_to_match();
    let mut reader = source.read_symbols();

    matches_symbol_range(&matcher, &mut reader)
}

///
/// Matches a source stream against a pattern
///
pub fn matches_prepared<'a, Symbol, OutputSymbol, Reader, Source>(source: Source, matcher: &SymbolRangeDfa<Symbol, OutputSymbol>) -> Option<usize>
where   Reader: SymbolReader<Symbol>+'a
,       Source: SymbolSource<'a, Symbol, SymbolReader=Reader>
,       Symbol: Ord
,       OutputSymbol: 'static {
    let mut reader = source.read_symbols();

    matches_symbol_range(&matcher, &mut reader)
}

#[cfg(test)]
mod test {
    use super::super::*;

    #[test]
    fn match_multiple_repeats() {
        assert!(matches("abcabc", "abc".repeat_forever(1)).is_some());
    }

    #[test]
    fn match_single_repeat() {
        assert!(matches("abc", "abc".repeat_forever(1)).is_some());
    }

    #[test]
    fn match_with_zero_or_more() {
        assert!(matches("abc", "abc".repeat_forever(0)).is_some());
        assert!(matches("abcabc", "abc".repeat_forever(0)).is_some());
        assert!(matches("abcabcabc", "abc".repeat_forever(0)).is_some());
    }

    #[test]
    fn match_with_zero_or_more_following() {
        assert!(matches("abc", "abc".repeat_forever(0).append("def")).is_none());
        assert!(matches("abcabc", "abc".repeat_forever(0).append("def")).is_none());

        assert!(matches("abcdef", "abc".repeat_forever(0).append("def")).is_some());
        assert!(matches("abcabcdef", "abc".repeat_forever(0).append("def")).is_some());
    }

    #[test]
    fn match_limited_range() {
        assert!(matches("abc", "abc".repeat(2..4)).is_none());
        assert!(matches("abcabc", "abc".repeat(2..4)).is_some());
        assert!(matches("abcabcabc", "abc".repeat(2..4)) == Some(3*3));
        assert!(matches("abcabcabcabc", "abc".repeat(2..4)) == Some(3*3));
    }

    /* -- BROKEN
    #[test]
    fn match_zero_repeats() {
        assert!(matches("", "abc".repeat_forever(0)).is_some());
    }
    */
}
