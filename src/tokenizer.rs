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
//! A TokenMatcher is a pattern matcher that is intended to turn a stream of symbols into another stream of symbols based on the patterns
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

#[cfg(test)]
mod test {
    use super::super::*;

    #[test]
    fn can_match_TokenMatcher_like_any_other_pattern() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            AllAs,
            AllBs
        }

        let mut TokenMatcher = TokenMatcher::new();
        TokenMatcher.add_pattern("a".repeat_forever(1), TestToken::AllAs);
        TokenMatcher.add_pattern("b".repeat_forever(1), TestToken::AllBs);

        assert!(matches("aaaa", &TokenMatcher) == Some(4));
        assert!(matches("bbbbb", &TokenMatcher) == Some(5));
        assert!(matches("abbb", &TokenMatcher) == Some(1));
        assert!(matches("bbaaa", &TokenMatcher) == Some(2));
    }

    #[test]
    fn can_distinguish_simple_tokens() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            AllAs,
            AllBs
        }

        let mut TokenMatcher = TokenMatcher::new();
        TokenMatcher.add_pattern("a".repeat_forever(1), TestToken::AllAs);
        TokenMatcher.add_pattern("b".repeat_forever(1), TestToken::AllBs);

        let matcher = TokenMatcher.prepare_to_match();

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

        let mut TokenMatcher = TokenMatcher::new();
        TokenMatcher.add_pattern("a".repeat_forever(1).append("b"), TestToken::Aaab);
        TokenMatcher.add_pattern("a".append("b".repeat_forever(1)), TestToken::Abbb);

        let matcher = TokenMatcher.prepare_to_match();

        assert!(match_pattern(matcher.start(), &mut "aaab".read_symbols()).is_accepted(&TestToken::Aaab));
        assert!(match_pattern(matcher.start(), &mut "ab".read_symbols()).is_accepted(&TestToken::Abbb));
        assert!(match_pattern(matcher.start(), &mut "abbbb".read_symbols()).is_accepted(&TestToken::Abbb));
    }
}
