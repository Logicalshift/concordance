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
//! # use concordance::*;
//! let mut ndfa: Ndfa<u32, u32> = Ndfa::new();
//! ```
//!

use std::collections::HashMap;
use std::collections::HashSet;

use super::state_machine::*;
use super::overlapping_symbols::*;
use super::countable::*;
use super::symbol_range::*;

///
/// Represents a non-deterministic finite-state automata
///
#[derive(Clone)]
pub struct Ndfa<InputSymbol, OutputSymbol> where InputSymbol : Clone {
    /// Highest known state ID
    max_state: StateId,

    /// Transitions for each state
    transitions: Vec<Vec<(InputSymbol, StateId)>>,

    /// The states that are joined with the state specified by the index
    joined_with: Vec<Vec<StateId>>,

    /// Output symbols for each state
    output_symbols: HashMap<StateId, OutputSymbol>
}

impl<InputSymbol: Clone, OutputSymbol> Ndfa<InputSymbol, OutputSymbol> {
    ///
    /// Creates a new non-deterministic finite Automaton
    ///
    /// The NDFA will initially just consist of the start state 0. The methods in `MutableStateMachine` should be called to
    /// build it into a more useful structure.
    ///
    pub fn new() -> Ndfa<InputSymbol, OutputSymbol> {
        Ndfa { max_state: 0, transitions: vec![], joined_with: vec![], output_symbols: HashMap::new() }
    }

    ///
    /// Retrieves the complete set of states whose transitions should be returned due to joining for a given state
    ///
    #[inline]
    fn get_join_closure(&self, state: StateId) -> HashSet<StateId> {
        // TODO: this is a great use-case for the BitSet collection (available in the nightlies)

        // The result is initially empty. We'll add the initial state in the first pass
        let mut result: HashSet<StateId> = HashSet::new();

        // Add in any joined states
        let mut stack = vec![];
        stack.push(state);

        while let Some(next_state) = stack.pop() {
            if !result.contains(&next_state) {
                // Add to the result
                result.insert(next_state);

                // Process any states that are joined to this one
                if (next_state as usize) < self.joined_with.len() {
                    let ref join_states = self.joined_with[next_state as usize];

                    for join_to in join_states {
                        stack.push(*join_to);
                    }
                }
            }
        }

        result
    }
}

impl<Symbol: Ord+Clone+Countable, OutputSymbol> Ndfa<SymbolRange<Symbol>, OutputSymbol> {
    ///
    /// Modifies this NDFA so that all ranges used in all transitions are unique and have no overlapping ranges
    ///
    pub fn fix_overlapping_ranges(&mut self) {
        // TODO: this forces us to fix overlapping ranges every time we generate an NDFA, rather than before use
        // We'd like to fix before use to allow for things like merged state machines

        // Gather all of the symbols in a map
        let mut symbol_map = SymbolMap::new();

        for transit in &self.transitions {
            for &(ref range, _) in transit {
                symbol_map.add_range(range);
            }
        }

        // Get a new map with no overlapping symbols
        let no_overlapping = symbol_map.to_non_overlapping_map();

        // Generate a new set of transitions based on no_overlapping
        let mut new_transitions = vec![];

        for transit in &self.transitions {
            let without_overlapping: Vec<(SymbolRange<Symbol>, StateId)> = transit.iter()
                .flat_map(|&(ref range, state)| {
                    let mut result = vec![];
                    for range in no_overlapping.find_overlapping_ranges(range) {
                        result.push((range.clone(), state));
                    }
                    result
                })
                .collect();
            new_transitions.push(without_overlapping);
        }

        self.transitions = new_transitions;
    }
}

impl<InputSymbol: Clone, OutputSymbol> StateMachine<InputSymbol, OutputSymbol> for Ndfa<InputSymbol, OutputSymbol> {
    ///
    /// Retrieves the number of states in this state machine
    ///
    fn count_states(&self) -> StateId {
        self.max_state + 1
    }

    ///
    /// Retrieves the transitions for a particular state
    ///
    fn get_transitions_for_state(&self, state: StateId) -> Vec<(InputSymbol, StateId)> {
        let empty           = vec![];

        // The transitions are all the transitions for this state, plus all the transitions in the states that are joined to it
        let joined_states   = self.get_join_closure(state);
        let merged          = joined_states.iter().flat_map(|join_state| {
            if (*join_state as usize) < self.transitions.len() {
                self.transitions[*join_state as usize].iter()
            } else {
                empty.iter()
            }
        });

        merged.map(|item| item.clone()).collect()
    }

    ///
    /// Retrieves the output symbol for a particular state
    ///
    fn output_symbol_for_state(&self, state: StateId) -> Option<&OutputSymbol> {
        let mut result = self.output_symbols.get(&state);

        match result {
            None => {
                let joined_states = self.get_join_closure(state);
                for joined in joined_states {
                    result = self.output_symbols.get(&joined);
                    match result {
                        None => {},
                        _ => { return result; }
                    }
                }

                None
            },

            _ => result
        }
    }
}

impl<InputSymbol : Clone, OutputSymbol> MutableStateMachine<InputSymbol, OutputSymbol> for Ndfa<InputSymbol, OutputSymbol> {
    ///
    /// Creates a new transition in the state machine
    ///
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

    ///
    /// Ensures that a state with the specified ID exists in this state machine
    ///
    fn create_state(&mut self, state: StateId) {
        if state > self.max_state {
            self.max_state = state;
        }
    }

    ///
    /// Sets the output symbol for a particular state
    ///
    fn set_output_symbol(&mut self, state: StateId, new_output_symbol: OutputSymbol) {
        // Output symbols update the max state too
        if state > self.max_state {
            self.max_state = state;
        }

        self.output_symbols.insert(state, new_output_symbol);
    }

    ///
    /// Joins two states so that the first state has the same transitions as the second state
    ///
    fn join_states(&mut self, first_state: StateId, second_state: StateId) {
        // Joining states updates the overall state count_states
        if first_state > self.max_state {
            self.max_state = first_state;
        }

        if second_state > self.max_state {
            self.max_state = second_state;
        }

        // Expand the join table so we can add the first state
        while self.joined_with.len() <= first_state as usize {
            self.joined_with.push(vec![]);
        }

        // Join the second state to the first state
        self.joined_with[first_state as usize].push(second_state);
    }
}

#[cfg(test)]
mod test {
    use super::super::state_machine::*;
    use super::*;

    #[test]
    fn there_is_initially_one_state() {
        let ndfa: Ndfa<u32, u32> = Ndfa::new();

        assert!(ndfa.count_states() == 1);
    }

    #[test]
    fn can_create_state_1() {
        let mut ndfa: Ndfa<u32, u32> = Ndfa::new();

        ndfa.create_state(1);

        assert!(ndfa.count_states() == 2);
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

    #[test]
    fn join_states_attaches_transitions_to_first_state() {
        let mut ndfa: Ndfa<u32, u32> = Ndfa::new();

        ndfa.add_transition(0, 42, 1);
        ndfa.add_transition(1, 43, 2);
        ndfa.join_states(0, 1);

        assert!(ndfa.get_transitions_for_state(0).contains(&(42, 1)));
        assert!(ndfa.get_transitions_for_state(0).contains(&(43, 2)));
    }

    #[test]
    fn adding_transition_after_join_updates_target() {
        let mut ndfa: Ndfa<u32, u32> = Ndfa::new();

        ndfa.join_states(0, 1);
        ndfa.add_transition(0, 42, 1);
        ndfa.add_transition(1, 43, 2);

        assert!(ndfa.get_transitions_for_state(0).contains(&(42, 1)));
        assert!(ndfa.get_transitions_for_state(0).contains(&(43, 2)));
    }

    #[test]
    fn join_updates_state_count() {
        let mut ndfa: Ndfa<u32, u32> = Ndfa::new();

        ndfa.join_states(0, 1);

        assert!(ndfa.count_states() == 2);
    }

    #[test]
    fn join_states_does_not_attach_to_second_state() {
        let mut ndfa: Ndfa<u32, u32> = Ndfa::new();

        ndfa.add_transition(0, 42, 1);
        ndfa.add_transition(1, 43, 2);
        ndfa.join_states(0, 1);

        assert!(!ndfa.get_transitions_for_state(1).contains(&(42, 1)));
        assert!(ndfa.get_transitions_for_state(1).contains(&(43, 2)));
    }

    #[test]
    fn join_states_recurses_to_further_states() {
        let mut ndfa: Ndfa<u32, u32> = Ndfa::new();

        ndfa.add_transition(0, 42, 1);
        ndfa.add_transition(1, 43, 2);
        ndfa.add_transition(2, 44, 3);
        ndfa.join_states(0, 1);
        ndfa.join_states(1, 2);

        assert!(ndfa.get_transitions_for_state(0).contains(&(42, 1)));
        assert!(ndfa.get_transitions_for_state(0).contains(&(43, 2)));
        assert!(ndfa.get_transitions_for_state(0).contains(&(44, 3)));
    }

    #[test]
    fn join_loop_attaches_to_both_states() {
        let mut ndfa: Ndfa<u32, u32> = Ndfa::new();

        ndfa.add_transition(0, 42, 1);
        ndfa.add_transition(1, 43, 2);
        ndfa.join_states(0, 1);
        ndfa.join_states(1, 0);

        assert!(ndfa.get_transitions_for_state(0).contains(&(42, 1)));
        assert!(ndfa.get_transitions_for_state(0).contains(&(43, 2)));

        assert!(ndfa.get_transitions_for_state(1).contains(&(42, 1)));
        assert!(ndfa.get_transitions_for_state(1).contains(&(43, 2)));
    }
}
