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
//! # NDFA
//!
//! The acronym NDFA stands for 'Non-Deterministic Finite Automaton'. An NDFA is a state machine where each state can have
//! transitions for more than one target state - this allows it to be in more than one state at once, which is the property
//! that makes it non-deterministic.
//!
//! NDFAs can match patterns conforming to a regular language - that is, they match regular expressions. However, their
//! non-deterministic nature means that they are inconvenient to evaluate. Fortunately, for every NDFA there can be found
//! a DFA - a *Deterministic* Finite Automaton - which can be used to rapidly match strings of symbols against the expression.
//!
//! Every implementation of the `StateMachine` trait represents an NDFA. This particular implementation is useful because it
//! also implements `MutableStateMachine` so can be used as a means to build state machines that can be used to match patterns.
//!
//! An NDFA is created by calling the constructor:
//!
//! ```
//! # use ndfa::*;
//! let mut ndfa: Ndfa<u32, u32> = Ndfa::new();
//! ```
//!

use super::state_machine::*;
use std::collections::HashMap;

///
/// Represents a non-deterministic finite-state automata
///
#[derive(Clone)]
pub struct Ndfa<InputSymbol, OutputSymbol> where InputSymbol : Clone {
    /// Highest known state ID
    max_state: StateId,

    /// Transitions for each state
    transitions: Vec<Vec<(InputSymbol, StateId)>>,

    /// Output symbols for each state
    output_symbols: HashMap<StateId, OutputSymbol>
}

impl<InputSymbol : Clone, OutputSymbol> Ndfa<InputSymbol, OutputSymbol> {
    pub fn new() -> Ndfa<InputSymbol, OutputSymbol> {
        Ndfa { max_state: 0, transitions: vec![], output_symbols: HashMap::new() }
    }
}

impl<InputSymbol : Clone, OutputSymbol> StateMachine<InputSymbol, OutputSymbol> for Ndfa<InputSymbol, OutputSymbol> {
    fn count_states(&self) -> StateId {
        self.max_state + 1
    }

    fn get_transitions_for_state(&self, state: StateId) -> Vec<(InputSymbol, StateId)> {
        if (state as usize) >= self.transitions.len() {
            vec![]
        } else {
            self.transitions[state as usize].clone()
        }
    }

    fn output_symbol_for_state(&self, state: StateId) -> Option<&OutputSymbol> {
        self.output_symbols.get(&state)
    }
}

impl<InputSymbol : Clone, OutputSymbol> MutableStateMachine<InputSymbol, OutputSymbol> for Ndfa<InputSymbol, OutputSymbol> {
    fn add_transition(&mut self, state: StateId, for_symbol: InputSymbol, new_state: StateId) {
        // Make sure that max_state reflects the highest state added by the user
        if new_state > self.max_state {
            self.max_state = new_state;
        }

        if state > self.max_state {
            self.max_state = state;
        }

        // Expand the transitions vector so that the new state has somewhere to go
        while self.transitions.len() <= state as usize {
            self.transitions.push(vec![]);
        }

        self.transitions[state as usize].push((for_symbol, new_state));
    }

    fn set_output_symbol(&mut self, state: StateId, new_output_symbol: OutputSymbol) {
        // Output symbols update the max state too
        if state > self.max_state {
            self.max_state = state;
        }

        self.output_symbols.insert(state, new_output_symbol);
    }

    fn join_states(&mut self, first_state: StateId, second_state: StateId) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::super::state_machine::*;
    use super::*;

    #[test]
    fn there_is_initially_one_state() {
        let ndfa: Ndfa<u32, u32> = Ndfa::new();

        assert!(ndfa.count_states() == 1);
    }

    #[test]
    fn adding_transition_updates_max_state_to_target() {
        let mut ndfa: Ndfa<u32, u32> = Ndfa::new();

        ndfa.add_transition(0, 42, 1);
        assert!(ndfa.count_states() == 2);
    }

    #[test]
    fn adding_output_symbol_updates_max_state() {
        let mut ndfa: Ndfa<u32, u32> = Ndfa::new();

        ndfa.set_output_symbol(1, 128);
        assert!(ndfa.count_states() == 2);
    }

    #[test]
    fn adding_transition_updates_max_state_to_source() {
        let mut ndfa: Ndfa<u32, u32> = Ndfa::new();

        ndfa.add_transition(1, 42, 0);
        assert!(ndfa.count_states() == 2);
    }

    #[test]
    fn can_retrieve_transition() {
        let mut ndfa: Ndfa<u32, u32> = Ndfa::new();

        ndfa.add_transition(1, 42, 0);
        assert!(ndfa.get_transitions_for_state(1).len() == 1);
        assert!(ndfa.get_transitions_for_state(1)[0] == (42, 0));
    }

    #[test]
    fn can_add_multiple_transitions() {
        let mut ndfa: Ndfa<u32, u32> = Ndfa::new();

        ndfa.add_transition(1, 42, 0);
        ndfa.add_transition(1, 42, 2);

        assert!(ndfa.get_transitions_for_state(1).len() == 2);
        assert!(ndfa.get_transitions_for_state(1).contains(&(42, 0)));
        assert!(ndfa.get_transitions_for_state(1).contains(&(42, 2)));
    }

    #[test]
    fn output_symbol_is_none_by_default() {
        let ndfa: Ndfa<u32, u32> = Ndfa::new();

        assert!(ndfa.output_symbol_for_state(0) == None);
        assert!(ndfa.output_symbol_for_state(1) == None);
    }

    #[test]
    fn can_set_output_symbol() {
        let mut ndfa: Ndfa<u32, u32> = Ndfa::new();

        ndfa.set_output_symbol(0, 64);

        assert!(ndfa.output_symbol_for_state(0) == Some(&64));
        assert!(ndfa.output_symbol_for_state(1) == None);
    }
}
