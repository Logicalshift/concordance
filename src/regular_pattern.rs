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
//! This provides a data structure, `Pattern<Symbol>` representing a matching pattern in a regular language. This is a 
//! more generalised form of a regular expression as it supports symbol types other than simple strings. A pattern using 
//! the `char` symbol has identical expressive power to a regular expression.
//!
//! Regular patterns can be created from strings, arrays or vectors using `into_pattern`, and can use any cloneable type
//! to represent a symbol:
//!
//! ```
//! # use concordance::*;
//! let match_abc = "abc".into_pattern();
//! let match_123 = [1,2,3].into_pattern();
//! ```
//!
//! These patterns are quite boring (they just match exactly the string that's passed in). To create more interesting
//! patterns, there are a series of functions that will create patterns with repetitions or other constructs in them:
//!
//! ```
//! # use concordance::*;
//! let stuff_or_nonsense = literal("stuff").or("nonsense");
//! let any_amount_of_stuff = literal("stuff").repeat_forever(1);
//! let went_to_market = literal("piggies").repeat(0..5);
//! ```
//!
//! For convenience, these methods will work on any type that can be converted into a pattern. Every regular expression
//! can be converted into a regular pattern, but these patterns are also for symbol types other than characters, and do
//! not require any quoting in order to be used (and are hence much easier to use with dynamic values).
//!
//! ```
//! # use concordance::*;
//! let some_counting = literal(vec![1, 2, 3]).repeat_forever(1);
//! ```
//!

use std::iter::FromIterator;
use std::ops::Range;

use super::state_machine::*;
use super::symbol_range::*;
use super::ndfa::*;
use super::countable::*;

///
/// A Pattern represents a matching pattern in a regular language
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Pattern<Symbol: Clone> {
    ///
    /// Matches nothing
    ///
    Epsilon,

    ///
    /// Matches a specific literal phrase
    ///
    Match(Vec<Symbol>),

    ///
    /// Matches a range of symbols
    ///
    /// Note that this is inclusive, so both the start and end symbols will be matched as well as any in between. Inclusive ranges allow
    /// the entire range of symbols to be matched (unlike exclusive ranges, which have to exclude at least one symbol by definition)
    ///
    MatchRange(Symbol, Symbol),

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

impl<Symbol: Clone+Ord+Countable> Pattern<Symbol> {
    ///
    /// Compiles this pattern onto a state machine, returning the accepting symbol
    ///
    pub fn compile<OutputSymbol>(&self, state_machine: &mut MutableStateMachine<SymbolRange<Symbol>, OutputSymbol>, start_state: StateId) -> StateId {
        match self {
            &Epsilon => {
                start_state
            },

            &Match(ref symbols) => {
                // Match each symbol in turn
                let mut current_state = start_state;

                for sym in symbols {
                    let next_state = state_machine.count_states();
                    state_machine.add_transition(current_state, SymbolRange::new(sym.clone(), sym.clone()), next_state);
                    current_state = next_state;
                }

                current_state
            },

            &MatchRange(ref first, ref last) => {
                let next_state = state_machine.count_states();
                state_machine.add_transition(start_state, SymbolRange::new(first.clone(), last.clone()), next_state);
                next_state
            },

            &RepeatInfinite(ref count, ref pattern) => {
                // Create a target state
                let target_state = state_machine.count_states();
                state_machine.create_state(target_state);

                let mut repeat_state = start_state;

                for repeat in 0..(count+2) {
                    // The last state can also be the target state
                    if repeat >= *count {
                        state_machine.join_states(repeat_state, target_state)
                    }

                    // Compile this iteration through the repetition
                    let initial_state = repeat_state;
                    repeat_state = pattern.compile(state_machine, repeat_state);

                    // The last state needs to repeat, so turn it into a loop
                    if repeat == *count+1 {
                        state_machine.join_states(repeat_state, initial_state);
                    }
                }

                target_state
            },

            &Repeat(ref range, ref pattern) => {
                // Create a target state
                let target_state = state_machine.count_states();
                state_machine.create_state(target_state);

                let mut repeat_state = start_state;

                for repeat in 0..(range.end) {
                    // If we've repeated at least range.start times, then we can finish the loop at this point
                    if repeat >= range.start {
                        state_machine.join_states(repeat_state, target_state)
                    }

                    // Compile this iteration through the repetition
                    repeat_state = pattern.compile(state_machine, repeat_state);
                }

                target_state
            },

            &MatchAll(ref patterns) => {
                // Match each pattern in turn
                let mut current_state = start_state;

                for pattern in patterns {
                    let next_state = pattern.compile(state_machine, current_state);
                    current_state = next_state;
                }

                current_state
            },

            &MatchAny(ref patterns) => {
                // Everything is compiled starting at a particular state, and everything ends on a particular state
                let target_state = state_machine.count_states();
                state_machine.create_state(target_state);

                for pattern in patterns {
                    let final_state = pattern.compile(state_machine, start_state);
                    state_machine.join_states(final_state, target_state);
                }

                target_state
            }
        }
    }
}

impl<Symbol: Clone+Ord+Countable+'static> ToNdfa<SymbolRange<Symbol>> for Pattern<Symbol> {
    fn to_ndfa<OutputSymbol: 'static>(&self, output: OutputSymbol) -> Box<StateMachine<SymbolRange<Symbol>, OutputSymbol>> {
        let mut result  = Ndfa::new();
        let end_state   = self.compile(&mut result, 0);

        result.set_output_symbol(end_state, output);
        result.fix_overlapping_ranges();

        Box::new(result)
    }
}

impl<Symbol: Clone+Ord+Countable+'static> ToNdfa<SymbolRange<Symbol>> for ToPattern<Symbol> {
    fn to_ndfa<OutputSymbol: 'static>(&self, output: OutputSymbol) -> Box<StateMachine<SymbolRange<Symbol>, OutputSymbol>> {
        self.to_pattern().to_ndfa(output)
    }
}

impl ToNdfa<SymbolRange<char>> for str {
    fn to_ndfa<OutputSymbol>(&self, output: OutputSymbol) -> Box<StateMachine<SymbolRange<char>, OutputSymbol>> {
        self.to_pattern().to_ndfa(output)
    }
}

impl<Symbol: Clone+Ord+Countable+'static> ToNdfa<SymbolRange<Symbol>> for [Symbol] {
    fn to_ndfa<OutputSymbol>(&self, output: OutputSymbol) -> Box<StateMachine<SymbolRange<Symbol>, OutputSymbol>> {
        self.to_pattern().to_ndfa(output)
    }
}

pub use Pattern::*;

///
/// Implemented by things that can be converted into a pattern
///
pub trait ToPattern<Symbol: Clone> {
    ///
    /// Converts a particular object into a pattern that will match it
    ///
    fn to_pattern(&self) -> Pattern<Symbol>;
}

///
/// Implemented by things that can be converted into a pattern
///
pub trait IntoPattern<Symbol: Clone> {
    ///
    /// Converts a particular object into a pattern that will match it
    ///
    fn into_pattern(self) -> Pattern<Symbol>;
}

impl<Symbol: Clone> IntoPattern<Symbol> for Pattern<Symbol> {
    #[inline]
    fn into_pattern(self) -> Pattern<Symbol> {
        self
    }
}

impl<Symbol: Clone> IntoPattern<Symbol> for Box<Pattern<Symbol>> {
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
pub trait PatternTransformer<Symbol: Clone> {
    /// Repeats the current pattern forever
    fn repeat_forever(self, min_count: u32) -> Pattern<Symbol>;

    /// Repeats the current pattern for a certain number of iterations
    fn repeat(self, count: Range<u32>) -> Pattern<Symbol>;
}

///
/// Creates a value that is matched literally in a pattern
///
#[inline]
pub fn literal<Symbol: Clone, PatternType: IntoPattern<Symbol>>(item: PatternType) -> Pattern<Symbol> {
    item.into_pattern()
}

///
/// Implemented by things that combine patterns together to create new patterns
///
pub trait PatternCombiner<Symbol: Clone, SecondPattern: IntoPattern<Symbol>> {
    /// Appends a pattern to this one
    fn append(self, pattern: SecondPattern) -> Pattern<Symbol>;

    /// Matches either this pattern or the specified pattern
    fn or(self, pattern: SecondPattern) -> Pattern<Symbol>;
}

impl<Symbol: Clone, PatternType: IntoPattern<Symbol>> PatternTransformer<Symbol> for PatternType {
    fn repeat_forever(self, min_count: u32) -> Pattern<Symbol> {
        RepeatInfinite(min_count, Box::new(self.into_pattern()))
    }

    fn repeat(self, count: Range<u32>) -> Pattern<Symbol> {
        Repeat(count, Box::new(self.into_pattern()))
    }
}

impl<Symbol: Clone, PatternType: IntoPattern<Symbol>, SecondPatternType: IntoPattern<Symbol>> PatternCombiner<Symbol, SecondPatternType> for PatternType {
    fn append(self, pattern: SecondPatternType) -> Pattern<Symbol> {
        // Get the two patterns to combine
        let first_pattern   = self.into_pattern();
        let second_pattern  = pattern.into_pattern();

        // Combination rules depend on what the patterns are
        match (first_pattern, second_pattern) {
            // Combining 'Match(x)' and 'Match(y)' should produce 'Match(xy)'
            (Match(first_string), Match(second_string)) => Match(first_string.into_iter().chain(second_string.into_iter()).collect()),

            // Combining 'MatchAll(x)' and 'MatchAll(y)' should produce 'MatchAll(xy)'
            (MatchAll(first_string), MatchAll(second_string)) => MatchAll(first_string.into_iter().chain(second_string.into_iter()).collect()),

            // Combining 'MatchAll(x)' and 'y' should produce 'MatchAll(xy)'
            (MatchAll(first_string), second) => {
                let mut result = first_string.clone();
                result.push(second);
                MatchAll(result)
            },

            // Combining 'x' and 'MatchAll(y)' should produce 'MatchAll(xy)'
            (first, MatchAll(second_string)) => {
                let mut result: Vec<Pattern<Symbol>> = vec![first];
                for p in second_string {
                    result.push(p.clone());
                }
                MatchAll(result)
            },

            // Everything else is just MatchAll(xy)
            (first, second) => MatchAll(vec![first, second])
        }
    }

    fn or(self, pattern: SecondPatternType) -> Pattern<Symbol> {
        // Get the two patterns to combine
        let first_pattern   = self.into_pattern();
        let second_pattern  = pattern.into_pattern();

        // Combination rules depend on what the patterns are
        match (first_pattern, second_pattern) {
            // Combining 'MatchAny(x)' and 'MatchAny(y)' should produce 'MatchAny(xy)'
            (MatchAny(first_string), MatchAny(second_string)) => MatchAny(first_string.into_iter().chain(second_string.into_iter()).collect()),

            // Everything else is just MatchAny(xy)
            (first, second) => MatchAny(vec![first, second])
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::state_machine::*;

    #[test]
    fn can_convert_vec_to_pattern() {
        let pattern = literal(&vec![0, 1, 2]);

        assert!(pattern == Match(vec![0, 1, 2]));
    }

    /*
    #[test]
    fn can_convert_array_to_pattern() {
        let pattern = literal([0, 1, 2]);

        assert!(pattern == Match(vec![0, 1, 2]));
    }
    */

    #[test]
    fn can_convert_string_to_pattern() {
        let pattern = literal("abc");

        assert!(pattern == Match(vec!['a', 'b', 'c']));
    }

    #[test]
    fn can_repeat_pattern() {
        let pattern = literal("abc").repeat(1..2);

        assert!(pattern == Repeat(1..2, Box::new(Match(vec!['a', 'b', 'c']))));
    }

    #[test]
    fn can_repeat_pattern_forever() {
        let pattern = literal("abc").repeat_forever(0);

        assert!(pattern == RepeatInfinite(0, Box::new(Match(vec!['a', 'b', 'c']))));
    }

    #[test]
    fn can_append_pattern_combine_matches() {
        let pattern = literal("abc").append("def");

        assert!(pattern == Match(vec!['a', 'b', 'c', 'd', 'e', 'f']));
    }

    #[test]
    fn can_append_pattern_combine_matchalls_left() {
        let pattern = MatchAll(vec!["abc".to_pattern()]).append("def");

        assert!(pattern == MatchAll(vec![Match(vec!['a', 'b', 'c']), Match(vec!['d', 'e', 'f'])]));
    }

    #[test]
    fn can_append_pattern_combine_matchalls_right() {
        let pattern = "abc".append(MatchAll(vec!["def".to_pattern()]));

        assert!(pattern == MatchAll(vec![Match(vec!['a', 'b', 'c']), Match(vec!['d', 'e', 'f'])]));
    }

    #[test]
    fn can_append_pattern_combine_matchalls_both() {
        let pattern = MatchAll(vec!["abc".to_pattern()]).append(MatchAll(vec!["def".to_pattern()]));

        assert!(pattern == MatchAll(vec![Match(vec!['a', 'b', 'c']), Match(vec!['d', 'e', 'f'])]));
    }

    #[test]
    fn can_append_pattern_combine_not_matchall_or_match() {
        let pattern: Pattern<char> = MatchAny(vec![Epsilon]).append(MatchAny(vec![Epsilon]));

        assert!(pattern == MatchAll(vec![MatchAny(vec![Epsilon]), MatchAny(vec![Epsilon])]));
    }

    #[test]
    fn can_or_pattern() {
        let pattern = "abc".or("def");

        assert!(pattern == MatchAny(vec![Match(vec!['a', 'b', 'c']), Match(vec!['d', 'e', 'f'])]));
    }

    #[test]
    fn can_or_pattern_combine_matchanys() {
        let pattern = MatchAny(vec!["abc".to_pattern()]).or(MatchAny(vec!["def".to_pattern()]));

        assert!(pattern == MatchAny(vec![Match(vec!['a', 'b', 'c']), Match(vec!['d', 'e', 'f'])]));
    }

    #[test]
    fn can_build_ndfa() {
        let pattern = "abc".or("xyz").repeat_forever(0);
        let ndfa = pattern.to_ndfa("success");

        assert!(ndfa.count_states() > 1);
    }

    #[test]
    fn can_build_ndfa_from_patternable() {
        let ndfa_str = "abc".to_ndfa("success");
        assert!(ndfa_str.count_states() > 1);

        let ndfa_array = [1, 2, 3].to_ndfa("success");
        assert!(ndfa_array.count_states() > 1);

        let vec: Vec<u32> = vec![1, 2, 3];
        let ndfa_vec = vec.to_ndfa("success");
        assert!(ndfa_vec.count_states() > 1);
    }
}
