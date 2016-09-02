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
//! These functions deal with establishing how much of a particular stream (or string) matches a particular pattern. The
//! most important function is `matches`, which takes a stream and a pattern and returns the number of characters from the
//! stream that matches the pattern. Use it like this:
//!
//! ```
//! # use ndfa::*;
//! # assert!(matches("abcabc", "abc".repeat_forever(1)) == Some(6));
//! # assert!(matches("abcabcabc", "abc".repeat_forever(1)).is_some());
//! # assert!(matches("abc", "abc").is_some());
//! # assert!(matches("def", "abc".repeat_forever(1)).is_none());
//! if matches("abcabc", "abc".repeat_forever(1)) == Some(6) { /* ... */ }
//! ```
//!
//! To determine if a string exactly matches a pattern, compare to the string length like this:
//!
//! ```
//! # use ndfa::*;
//! let input_string = "abcabc";
//! let pattern      = "abc".repeat_forever(1);
//!
//! if matches(input_string, pattern) == Some(input_string.len()) { }
//! # assert!(matches(input_string, "abc".repeat_forever(1)) == Some(input_string.len()));
//! ```
//!

use super::symbol_range_dfa::*;
use super::symbol_reader::*;
use super::pattern_matcher::*;
use super::prepare::*;

///
/// Runs a DFA against a symbol stream and returns its final state
///
/// This takes a DFA match action as an initial state (such as that returned by `SymbolRangeDfa::start()`) and runs it until it accepts
/// or rejects the pattern passed to it.
///
/// This call is useful for cases where there is more than one output symbol (as the output symbol that was matched can be retrieved)
/// or for working with pattern matchers other than the default one.
///
/// ```
/// # use ndfa::*;
/// let input_string = "abcabc";
/// let pattern      = "abc".repeat_forever(1);
/// let matcher      = pattern.prepare_to_match();
///
/// let match_result = match_pattern(matcher.start(), &mut input_string.read_symbols()); // == Accept(6, &true)
/// # assert!(match match_result { Accept(count, val) => count == 6 && val == &true, _ => false });
/// ```
///
pub fn match_pattern<'a, InputSymbol: Ord, OutputSymbol, State>(start_state: MatchAction<'a, OutputSymbol, State>, symbol_reader: &mut SymbolReader<InputSymbol>) -> MatchAction<'a, OutputSymbol, State>
where State: MatchingState<'a, InputSymbol, OutputSymbol> {
    let mut current_state = start_state;

    while let More(this_state) = current_state {
        let next_state = 
            if let Some(next_char) = symbol_reader.next_symbol() {
                this_state.next(next_char)
            } else {
                this_state.finish()
            };

        current_state = next_state;
    }

    current_state
}

///
/// Runs a pattern matcher against a stream, and returns the number of characters matching if it accepted the stream
///
fn matches_symbol_range<InputSymbol: Ord, OutputSymbol: 'static>(dfa: &SymbolRangeDfa<InputSymbol, OutputSymbol>, symbol_reader: &mut SymbolReader<InputSymbol>) -> Option<usize> {
    // Run the DFA
    let final_state = match_pattern(dfa.start(), symbol_reader);

    if let Accept(count, _) = final_state {
        Some(count)
    } else {
        None
    }
}

///
/// Matches a source stream against a pattern
///
/// This is the basic pattern matcher. It matches against the left-hand side of the source, and if there is a string of any
/// length that can match the passed in pattern it will return the length of that string. Pattern matchers should be greedy,
/// so this will return the length of the longest string that can match the given pattern.
///
/// ```
/// # use ndfa::*;
/// matches("abc", "abc");                      // Returns Some(3)
/// matches("abcabc", "abc");                   // Also returns Some(3) as 'abc' matches the pattern
/// matches("abcabc", "abc".repeat_forever(0)); // Returns Some(6)
/// matches("ab", "abc");                       // Doesn't match: returns None
/// ```
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
/// Matches a source stream against a prepared pattern
///
/// If it's necessary to match a pattern against a lot of different things, then preparing it by calling `pattern.prepare_to_match()`
/// will increase the performance of the matcher for every match after the first one. This call is otherwise identical to `matches`.
///
/// ```
/// # use ndfa::*;
/// let prepared = "abc".repeat_forever(1).prepare_to_match();
///
/// matches_prepared("abcabc", &prepared);      // == Some(6));
/// matches_prepared("abc", &prepared);         // == Some(3));
/// matches_prepared("abcabcabc", &prepared);   // == Some(9));
/// ```
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
    fn match_prepared() {
        let prepared = "abc".repeat_forever(1).prepare_to_match();

        assert!(matches_prepared("abcabc", &prepared) == Some(6));
        assert!(matches_prepared("abc", &prepared) == Some(3));
        assert!(matches_prepared("abcabcabc", &prepared) == Some(9));
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
