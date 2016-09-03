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
//! A DFA that matches transitions against symbol ranges.
//!

use super::dfa_builder::*;
use super::pattern_matcher::*;
use super::symbol_range::*;
use super::state_machine::*;

///
/// DFA that decides on transitions based on non-overlapping, sorted lists of input symbols
///
#[derive(Debug)]
pub struct SymbolRangeDfa<InputSymbol: Ord, OutputSymbol> {
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
pub struct SymbolRangeDfaBuilder<InputSymbol: Ord, OutputSymbol> {
    states: Vec<usize>,
    transitions: Vec<(SymbolRange<InputSymbol>, StateId)>,
    accept: Vec<Option<OutputSymbol>>
}

impl<InputSymbol: Ord, OutputSymbol> SymbolRangeDfaBuilder<InputSymbol, OutputSymbol> {
    pub fn new() -> SymbolRangeDfaBuilder<InputSymbol, OutputSymbol> {
        SymbolRangeDfaBuilder { states: vec![], transitions: vec![], accept: vec![] }
    }
}

impl<InputSymbol: Ord, OutputSymbol> DfaBuilder<SymbolRange<InputSymbol>, OutputSymbol, SymbolRangeDfa<InputSymbol, OutputSymbol>> for SymbolRangeDfaBuilder<InputSymbol, OutputSymbol> {
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

impl<InputSymbol: Ord+Clone, OutputSymbol> StateMachine<SymbolRange<InputSymbol>, OutputSymbol> for SymbolRangeDfa<InputSymbol, OutputSymbol> {
    ///
    /// Returns the number of states in this state machine
    ///
    /// Note that if state x exists then state x-1 is also expected to exist provided x > 0. This means that this returns the
    /// first unused state in this state machine.
    ///
    fn count_states(&self) -> StateId {
        (self.states.len()-1) as StateId
    }

    ///
    /// Returns the transitions for a particular symbol 
    ///
    fn get_transitions_for_state(&self, state: StateId) -> Vec<(SymbolRange<InputSymbol>, StateId)> {
        let mut result = vec![];

        let start_index = self.states[state as usize];
        let end_index   = self.states[(state+1) as usize];

        for transit_index in start_index..end_index {
            let (ref range, target_state) = self.transitions[transit_index];

            result.push((range.clone(), target_state));
        }

        result
    }

    ///
    /// If a state is an accepting state, then this returns the output symbol that should be produced if this is the longest match
    ///
    fn output_symbol_for_state(&self, state: StateId) -> Option<&OutputSymbol> {
        self.accept[state as usize].as_ref()
    }
}

///
/// A state of a symbol range state machine
///
#[derive(Clone)]
pub struct SymbolRangeState<'a, InputSymbol: Ord+'a, OutputSymbol: 'a> {
    // The current state of the state machine
    state: StateId,

    // The number of symbols that have been processed so far
    count: usize,

    // If something other than none, the most recent accepting state
    accept: Option<(usize, &'a OutputSymbol)>,

    // The state machine this is running
    state_machine: &'a SymbolRangeDfa<InputSymbol, OutputSymbol>
}

impl<InputSymbol: Ord, OutputSymbol> SymbolRangeDfa<InputSymbol, OutputSymbol> {
    ///
    /// Returns a `MatchAction` for the initial state of the DFA
    ///
    pub fn start<'a>(&'a self) -> MatchAction<'a, OutputSymbol, SymbolRangeState<'a, InputSymbol, OutputSymbol>> {
        // TODO: if state 0 is accepting, then this will erroneously not move straight to the accepting state
        if let Some(ref outputsymbol) = self.accept[0] {
            More(SymbolRangeState { state: 0, count: 0, accept: Some((0, outputsymbol)), state_machine: self })
        } else {
            More(SymbolRangeState { state: 0, count: 0, accept: None, state_machine: self })
        }
    }
}

impl<'a, InputSymbol: Ord+'a, OutputSymbol: 'a> MatchingState<'a, InputSymbol, OutputSymbol> for SymbolRangeState<'a, InputSymbol, OutputSymbol> {
    fn next(self, symbol: InputSymbol) -> MatchAction<'a, OutputSymbol, Self> {
        // The transition range is defined by the current state
        let start_transition    = self.state_machine.states[self.state as usize];
        let end_transition      = self.state_machine.states[self.state as usize+1];

        // See if there is an input symbol matching this transition
        // TODO: consider binary searching for states with large numbers of transitions? (Do these occur regularly in patterns that people use?)
        for transit in start_transition..end_transition {
            // Test this transition
            let (ref range, new_state) = self.state_machine.transitions[transit];

            if range.includes(&symbol) {
                // Found a transition to a new state: result will be `More(new state)`
                let new_count = self.count+1;

                // If the new state is an accepting state, then remember it in case we reach a rejecting state later
                let new_accept = if let Some(ref output) = self.state_machine.accept[new_state as usize] {
                    Some((new_count, output))
                } else {
                    self.accept
                };

                // Action is 'More'
                // TODO: might be an option to return Accept or Reject here if the new state has no transitions
                // (Possible performance advantage, but depends on the regex and input conditions)
                return More(SymbolRangeState { state: new_state, count: new_count, accept: new_accept, state_machine: self.state_machine });
            }
        }

        // No matches: finish the state machine
        self.finish()
    }

    fn finish(self) -> MatchAction<'a, OutputSymbol, Self> {
        if let Some(accept_state) = self.accept {
            // We found an accepting state earlier on, so return that
            let (length, symbol) = accept_state;
            Accept(length, symbol)
        } else {
            // No accepting state was found while this state machine was running
            Reject
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::dfa_builder::*;
    use super::super::symbol_range::*;
    use super::super::pattern_matcher::*;
    use super::super::state_machine::*;
    use super::*;

    #[test]
    fn can_build_state_machine() {
        let mut builder = SymbolRangeDfaBuilder::new();

        // State 0: '0', move to state 1
        builder.start_state();
        builder.transition(SymbolRange::new(0, 0), 1);

        // State 1: accept, output symbol "Success"
        builder.start_state();
        builder.accept("Success");

        // Create the state machine  
        let state_machine = builder.build();

        assert!(state_machine.count_states() == 2);
        assert!(state_machine.output_symbol_for_state(0) == None);
        assert!(state_machine.output_symbol_for_state(1) == Some(&"Success"));
        assert!(state_machine.get_transitions_for_state(0) == vec![(SymbolRange::new(0,0), 1)]);
    }

    #[test]
    fn can_accept_single_symbol() {
        let mut builder = SymbolRangeDfaBuilder::new();

        // State 0: '0', move to state 1
        builder.start_state();
        builder.transition(SymbolRange::new(0, 0), 1);

        // State 1: accept, output symbol "Success"
        builder.start_state();
        builder.accept("Success");

        // Create the state machine  
        let state_machine = builder.build();

        // Run the first state
        let mut action = state_machine.start();

        if let More(next_state) = action {
            action = next_state.next(0);
        }

        if let More(next_state) = action {
            action = next_state.next(0);

            // Should have reached an accepting state (read one character)
            if let Accept(count, symbol) = action {
                // One symbol accepted
                assert!(count == 1);

                // Output symbol correct
                assert!(symbol == &"Success");
            } else {
                // Should have accepted here (the second '0' is rejected)
                assert!(false);
            }
        } else {
            // State machine did not accept the character
            assert!(false);
        }
    }
}
