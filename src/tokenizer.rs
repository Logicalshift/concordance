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
//! A tokenizer is a pattern matcher that is intended to turn a stream of symbols into another stream of symbols based on the patterns
//! that are matched. Every pattern can produce a different output symbol. If two input strings can ndfa in two different output
//! symbols, then the output symbol that is ordered lower is the one that's produced (ie, if the output symbols are numbers, then '0' will
//! be produced instead of '1' in the event of a clash)
//!

use super::countable::*;
use super::symbol_range::*;
use super::regular_pattern::*;
use super::state_machine::*;
use super::ndfa::*;
use super::prepare::*;
use super::symbol_range_dfa::*;
use super::symbol_reader::*;
use super::pattern_matcher::*;
use super::matches::*;
use super::tape::*;

///
/// Used for generating tokenizing pattern matchers
///
pub struct TokenMatcher<InputSymbol: Clone+Ord+Countable, OutputSymbol: Clone+Ord> {
    patterns: Vec<(Pattern<InputSymbol>, OutputSymbol)>
}

impl<InputSymbol: Clone+Ord+Countable+'static, OutputSymbol: Clone+Ord+'static> TokenMatcher<InputSymbol, OutputSymbol> {
    ///
    /// Creates a new TokenMatcher
    ///
    pub fn new() -> TokenMatcher<InputSymbol, OutputSymbol> {
        TokenMatcher { patterns: vec![] }
    }

    ///
    /// Adds a new pattern that will generate the specified output symbol
    ///
    pub fn add_pattern<TPattern: ToPattern<InputSymbol>>(&mut self, pattern: TPattern, output: OutputSymbol) {
        self.patterns.push((pattern.to_pattern(), output));
    }

    ///
    /// Compiles an NDFA from this TokenMatcher
    ///
    pub fn to_ndfa(&self) -> Box<StateMachine<SymbolRange<InputSymbol>, OutputSymbol>> {
        let mut ndfa = Ndfa::new();

        for &(ref pattern, ref output) in &self.patterns {
            // Compile each pattern starting at state 0
            let end_state = pattern.compile(&mut ndfa, 0);

            // Set the output for this pattern
            ndfa.set_output_symbol(end_state, output.clone());
        }

        // Clear out any overlapping ranges so we can build a valid DFA
        ndfa.fix_overlapping_ranges();

        Box::new(ndfa)
    }
}

impl<'a, InputSymbol: Clone+Ord+Countable+'static, OutputSymbol: Clone+Ord+'static> PrepareToMatch<SymbolRangeDfa<InputSymbol, OutputSymbol>> 
for &'a TokenMatcher<InputSymbol, OutputSymbol> {
    #[inline]
    fn prepare_to_match(self) -> SymbolRangeDfa<InputSymbol, OutputSymbol> {
        let ndfa = self.to_ndfa();

        ndfa.prepare_to_match()
    }
}

///
/// A tokenizer is a type of symbol stream that uses a pattern matcher to convert a symbol stream into a stream of tokens
///
pub struct Tokenizer<InputSymbol: Clone+Ord+Countable, OutputSymbol: Clone+Ord, Reader: SymbolReader<InputSymbol>> {
    /// The pattern matcher for this tokenizer
    dfa: SymbolRangeDfa<InputSymbol, OutputSymbol>,

    /// Tape of input symbols that will be used to generate the result
    tape: Tape<InputSymbol, Reader>,
}

impl<InputSymbol: Clone+Ord+Countable, OutputSymbol: Clone+Ord, Reader: SymbolReader<InputSymbol>> Tokenizer<InputSymbol, OutputSymbol, Reader> {
    ///
    /// Creates a new tokenizer from a pattern (usually a TokenMatcher)
    ///
    pub fn new<Prepare: PrepareToMatch<SymbolRangeDfa<InputSymbol, OutputSymbol>>>(source: Reader, pattern: Prepare) -> Tokenizer<InputSymbol, OutputSymbol, Reader> {
        Tokenizer { dfa: pattern.prepare_to_match(), tape: Tape::new(source) }
    }

    ///
    /// Creates a new tokenizer from a prepared pattern
    ///
    pub fn new_prepared(source: Reader, pattern: SymbolRangeDfa<InputSymbol, OutputSymbol>) -> Tokenizer<InputSymbol, OutputSymbol, Reader> {
        Tokenizer { dfa: pattern, tape: Tape::new(source) }
    }

    ///
    /// Returns the current position in the source (the position after the last matched symbol)
    ///
    pub fn get_source_position(&self) -> usize {
        self.tape.get_source_position()
    }

    ///
    /// Skips an input symbol (returning the symbol that was skipped)
    ///
    pub fn skip_input(&mut self) -> Option<InputSymbol> {
        self.tape.next_symbol()
    }

    ///
    /// True if we've reached the end of the source reader
    ///
    /// If `next_symbol` returns `None` and `at_end_of_reader` is false, then the input stream does not contain a symbol matching the DFA
    ///
    pub fn at_end_of_reader(&self) -> bool {
        self.tape.at_end_of_reader()
    }
}

impl<InputSymbol: Clone+Ord+Countable, OutputSymbol: Clone+Ord+'static, Reader: SymbolReader<InputSymbol>> SymbolReader<OutputSymbol> for Tokenizer<InputSymbol, OutputSymbol, Reader> {
    fn next_symbol(&mut self) -> Option<OutputSymbol> {
        // Start of the next symbol
        let start_pos = self.tape.get_source_position();

        // Match against it
        let match_result = match_pattern(self.dfa.start(), &mut self.tape);

        let end_pos = self.tape.get_source_position();
        match match_result {
            Accept(length, outputsymbol) => {
                if length > 0 {
                    // Rewind the tape to after the accepted symbol
                    self.tape.rewind(end_pos-start_pos - length);

                    // Won't try to match anything before this position
                    self.tape.cut();

                    // Result is the oputput symbol
                    Some(outputsymbol.clone())
                } else {
                    // Zero-length match
                    // If we accepted matches of length 0 we'd get an infinite stream when we hit a symbol that doesn't match
                    self.tape.rewind(end_pos-start_pos);

                    // Return no match
                    None
                }
            },

            Reject => {
                // Rewind back to the start position
                self.tape.rewind(end_pos-start_pos);

                // No match
                None
            },

            _ => {
                panic!("Unexpected output state of ");
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;

    #[test]
    fn can_match_tokens_like_any_other_pattern() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            AllAs,
            AllBs
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern("a".repeat_forever(1), TestToken::AllAs);
        token_matcher.add_pattern("b".repeat_forever(1), TestToken::AllBs);

        assert!(matches("aaaa", &token_matcher) == Some(4));
        assert!(matches("bbbbb", &token_matcher) == Some(5));
        assert!(matches("abbb", &token_matcher) == Some(1));
        assert!(matches("bbaaa", &token_matcher) == Some(2));
    }

    #[test]
    fn can_distinguish_simple_tokens() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            AllAs,
            AllBs
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern("a".repeat_forever(1), TestToken::AllAs);
        token_matcher.add_pattern("b".repeat_forever(1), TestToken::AllBs);

        let matcher = token_matcher.prepare_to_match();

        assert!(match_pattern(matcher.start(), &mut "aaaaa".read_symbols()).is_accepted(&TestToken::AllAs));
        assert!(match_pattern(matcher.start(), &mut "bbbb".read_symbols()).is_accepted(&TestToken::AllBs));
    }

    #[test]
    fn clashes_producer_lower_tokens() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            Abbb,
            Aaab
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern("a".repeat_forever(1).append("b"), TestToken::Aaab);
        token_matcher.add_pattern("a".append("b".repeat_forever(1)), TestToken::Abbb);

        let matcher = token_matcher.prepare_to_match();

        assert!(match_pattern(matcher.start(), &mut "aaab".read_symbols()).is_accepted(&TestToken::Aaab));
        assert!(match_pattern(matcher.start(), &mut "ab".read_symbols()).is_accepted(&TestToken::Abbb));
        assert!(match_pattern(matcher.start(), &mut "abbbb".read_symbols()).is_accepted(&TestToken::Abbb));
    }

    #[test]
    fn can_match_number_stream() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            Digit,
            Whitespace
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(1), TestToken::Digit);
        token_matcher.add_pattern(" ".repeat_forever(1), TestToken::Whitespace);

        let mut tokenizer = Tokenizer::new("12 390  32 ".read_symbols(), &token_matcher);

        assert!(tokenizer.next_symbol() == Some(TestToken::Digit));
        assert!(tokenizer.get_source_position() == 2);
        assert!(!tokenizer.at_end_of_reader());
        assert!(tokenizer.next_symbol() == Some(TestToken::Whitespace));
        assert!(tokenizer.get_source_position() == 3);
        assert!(tokenizer.next_symbol() == Some(TestToken::Digit));
        assert!(tokenizer.get_source_position() == 6);
        assert!(tokenizer.next_symbol() == Some(TestToken::Whitespace));
        assert!(tokenizer.get_source_position() == 8);
        assert!(tokenizer.next_symbol() == Some(TestToken::Digit));
        assert!(tokenizer.get_source_position() == 10);
        assert!(!tokenizer.at_end_of_reader());
        assert!(tokenizer.next_symbol() == Some(TestToken::Whitespace));
        assert!(tokenizer.at_end_of_reader());
        assert!(tokenizer.next_symbol() == None);
    }

    #[test]
    fn can_skip_bad_symbols() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            Digit,
            Whitespace
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(1), TestToken::Digit);
        token_matcher.add_pattern(" ".repeat_forever(1), TestToken::Whitespace);

        let mut tokenizer = Tokenizer::new("12 ab 12".read_symbols(), &token_matcher);

        assert!(tokenizer.next_symbol() == Some(TestToken::Digit));
        assert!(tokenizer.next_symbol() == Some(TestToken::Whitespace));
        assert!(tokenizer.next_symbol() == None);
        assert!(!tokenizer.at_end_of_reader());
        assert!(tokenizer.skip_input() == Some('a'));
        assert!(tokenizer.next_symbol() == None);
        assert!(tokenizer.skip_input() == Some('b'));
        assert!(!tokenizer.at_end_of_reader());
        assert!(tokenizer.next_symbol() == Some(TestToken::Whitespace));
        assert!(tokenizer.next_symbol() == Some(TestToken::Digit));
        assert!(tokenizer.next_symbol() == None);
        assert!(tokenizer.at_end_of_reader());
    }

    #[test]
    fn wont_match_zero_length() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            Digit,
            Whitespace
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(0), TestToken::Digit);
        token_matcher.add_pattern(" ".repeat_forever(0), TestToken::Whitespace);

        let mut tokenizer = Tokenizer::new("12 ab 12".read_symbols(), &token_matcher);

        assert!(tokenizer.next_symbol() == Some(TestToken::Digit));
        assert!(tokenizer.next_symbol() == Some(TestToken::Whitespace));
        assert!(tokenizer.next_symbol() == None);
        assert!(!tokenizer.at_end_of_reader());
        assert!(tokenizer.skip_input() == Some('a'));
        assert!(tokenizer.next_symbol() == None);
        assert!(!tokenizer.at_end_of_reader());
        assert!(tokenizer.skip_input() == Some('b'));
        assert!(tokenizer.next_symbol() == Some(TestToken::Whitespace));
        assert!(tokenizer.next_symbol() == Some(TestToken::Digit));
        assert!(tokenizer.next_symbol() == None);
        assert!(tokenizer.at_end_of_reader());
    }
}
