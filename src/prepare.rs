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
//! # Prepare
//!
//! If a pattern needs to be matched against many different things, it can be prepared in order to generate an object that can
//! be used many times without the delay of generating a DFA.
//!
//! ```
//! # use ndfa::*;
//! let match_abcs      = "abc".prepare_to_match();
//! let match_many_abcs = "abc".repeat_forever(1).prepare_to_match();
//! 
//! if matches("abcabc", match_many_abcs).is_some() { /* ... */ }
//! # assert!(matches("abc", match_many_abcs).is_some());
//! ```
//!

use super::countable::*;
use super::symbol_range::*;
use super::state_machine::*;
use super::regular_pattern::*;
use super::symbol_range_dfa::*;
use super::dfa_compiler::*;

///
/// Can be applied to patterns and other objects in order to match them
///
pub trait PrepareToMatch<As> where As: Sized {
    fn prepare_to_match(self) -> As;
}

impl<InputSymbol: Clone+PartialOrd+Countable, OutputSymbol: Ord+Clone> PrepareToMatch<SymbolRangeDfa<InputSymbol, OutputSymbol>> 
for Box<StateMachine<SymbolRange<InputSymbol>, OutputSymbol>> {
    #[inline]
    fn prepare_to_match(self) -> SymbolRangeDfa<InputSymbol, OutputSymbol> {
        let builder       = SymbolRangeDfaBuilder::new();
        let state_machine = DfaCompiler::build(self, builder);

        state_machine
    }
}

impl<InputSymbol: Clone+PartialOrd+Countable+'static> PrepareToMatch<SymbolRangeDfa<InputSymbol, bool>> 
for Pattern<InputSymbol> {
    #[inline]
    fn prepare_to_match(self) -> SymbolRangeDfa<InputSymbol, bool> {
        let ndfa = self.to_ndfa(true);

        ndfa.prepare_to_match()
    }
}

impl<'a, InputSymbol: Clone+PartialOrd+Countable+'static> PrepareToMatch<SymbolRangeDfa<InputSymbol, bool>> 
for &'a ToPattern<InputSymbol> {
    #[inline]
    fn prepare_to_match(self) -> SymbolRangeDfa<InputSymbol, bool> {
        let pattern = self.to_pattern();

        pattern.prepare_to_match()
    }
}

impl<InputSymbol: Clone+PartialOrd+Countable, OutputSymbol> PrepareToMatch<SymbolRangeDfa<InputSymbol, OutputSymbol>> for SymbolRangeDfa<InputSymbol, OutputSymbol> {
    #[inline]
    fn prepare_to_match(self) -> SymbolRangeDfa<InputSymbol, OutputSymbol> {
        self
    }
}

impl<'a> PrepareToMatch<SymbolRangeDfa<char, bool>> 
for &'a str {
    #[inline]
    fn prepare_to_match(self) -> SymbolRangeDfa<char, bool> {
        let pattern = self.to_pattern();

        pattern.prepare_to_match()
    }
}
