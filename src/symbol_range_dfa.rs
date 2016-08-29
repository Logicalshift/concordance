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
//! # RangeDfa
//!
//! A DFA that matches transitions against symbol ranges.
//!

use std::marker::PhantomData;

use super::dfa_builder::*;
use super::pattern_matcher::*;
use super::symbol_range::*;
use super::state_machine::*;

///
/// DFA that decides on transitions based on non-overlapping, sorted lists of input symbols
///
pub struct SymbolRangeDfa<InputSymbol: PartialOrd, OutputSymbol> {
    //
    // Indexes of where each state starts in the transition table (it ends at the start of the next state)
    //
    states: Vec<usize>,

    //
    // The transitions making up this DFA
    //
    transitions: Vec<(SymbolRange<InputSymbol>, StateId)>,

    //
    // The accepting symbol for each state
    //
    accept: Vec<Option<OutputSymbol>>
}

///
/// DFA builder that creates RangeDfas
///
pub struct SymbolRangeDfaBuilder<InputSymbol: PartialOrd, OutputSymbol> {
    states: Vec<usize>,
    transitions: Vec<(SymbolRange<InputSymbol>, StateId)>,
    accept: Vec<Option<OutputSymbol>>
}

impl<InputSymbol: PartialOrd, OutputSymbol> SymbolRangeDfaBuilder<InputSymbol, OutputSymbol> {
    pub fn new() -> SymbolRangeDfaBuilder<InputSymbol, OutputSymbol> {
        SymbolRangeDfaBuilder { states: vec![], transitions: vec![], accept: vec![] }
    }
}

impl<InputSymbol: PartialOrd, OutputSymbol> DfaBuilder<SymbolRange<InputSymbol>, OutputSymbol, SymbolRangeDfa<InputSymbol, OutputSymbol>> for SymbolRangeDfaBuilder<InputSymbol, OutputSymbol> {
    fn start_state(&mut self) {
        // Begin the next state
        self.states.push(self.transitions.len());
        self.accept.push(None);
    }

    fn transition(&mut self, symbol: SymbolRange<InputSymbol>, target_state: StateId) {
        self.transitions.push((symbol, target_state));
    }

    fn accept(&mut self, symbol: OutputSymbol) {
        self.accept.pop();
        self.accept.push(Some(symbol));
    }

    fn build(self) -> SymbolRangeDfa<InputSymbol, OutputSymbol> {
        // Turn into a RangeDfa
        let mut result = SymbolRangeDfa { states: self.states, transitions: self.transitions, accept: self.accept };

        // 'Cap' the last state so we don't need to special-case it later 
        // ie, we can always find the index of the last symbol by looking at the next state and don't need to handle the final state differently
        result.states.push(result.transitions.len());

        result
    }
}

///
/// A state of a symbol range state machine
///
pub struct SymbolRangeState<InputSymbol: PartialOrd, OutputSymbol: Sized> {
    // The current state of the state machine
    state: StateId,

    // The number of symbols that have been processed so far
    count: usize,

    // If something other than none, the most recent accepting state
    accept: Option<(usize, OutputSymbol)>,

    // Stop Rust whining about there being no fields of this type, this is used by the function definitions
    input_symbol: PhantomData<InputSymbol>
}

impl<InputSymbol: PartialOrd, OutputSymbol: Sized> PatternMatcher<InputSymbol, OutputSymbol> for SymbolRangeDfa<InputSymbol, OutputSymbol> {
    type State = SymbolRangeState<InputSymbol, OutputSymbol>;

    fn start(&self) -> Self::State {
        SymbolRangeState { state: 0, count: 0, accept: None, input_symbol: PhantomData }
    }
}

impl<InputSymbol: PartialOrd, OutputSymbol: Sized> MatchingState<InputSymbol, OutputSymbol> for SymbolRangeState<InputSymbol, OutputSymbol> {
    fn next(self, symbol: InputSymbol) -> MatchAction<OutputSymbol, Self> {
        unimplemented!()
    }

    fn finish(self) -> MatchAction<OutputSymbol, Self> {
        if let Some(accept_state) = self.accept {
            let (length, symbol) = accept_state;
            Accept(length, symbol)
        } else {
            Reject
        }
    }
}
