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
//! The DFA compiler converts NDFAs into DFAs, using a DFA builder. Usually the `SymbolRangeDfaBuilder` would be used but the
//! compiler can be used with any object implementing the `DfaBuilder` trait.
//!
//! The NDFA should not have any overlapping symbols, which is to say symbols that are not equal and yet could match the same
//! input symbol. If the builder finds that two NDFA states have identical output symbols, then the builder will pick the symbol
//! that compares as being lower as the final output symbol.
//!
//! Any NDFA can be converted into a DFA: if the NDFA can move to two states as the result of a particular input symbol, the DFA
//! just needs a single new state representing both those possible states. In this way, the NDFA can be converted into a form where
//! every input symbol only maps to a single transition.
//!
//! Normally the compiler isn't used directly: it's called by implementations of `PrepareToMatch`. It's useful for manually building
//! DFAs for other purposes or for creating new types of state machine that aren't already in this library.
//!
//! ```
//! # use concordance::*;
//! let ndfa     = "abc".into_pattern().to_ndfa("Success");
//! let builder  = SymbolRangeDfaBuilder::new();
//!
//! let state_machine = DfaCompiler::build(ndfa, builder);
//! ```
//!

use std::marker::PhantomData;
use std::collections::HashSet;
use std::collections::HashMap;
use std::iter::*;

use super::dfa_builder::*;
use super::state_machine::*;

///
/// Builds a deterministic finite automaton from a NDFA
///
pub struct DfaCompiler<InputSymbol: Ord+Clone, OutputSymbol, DfaType, Ndfa: StateMachine<InputSymbol, OutputSymbol>, Builder: DfaBuilder<InputSymbol, OutputSymbol, DfaType>> {
    /// State machine that is to be compiled
    ndfa: Ndfa,

    /// Builder where the state machine should be generated
    builder: Builder,

    // Phantom data to poke Rust's type system (it's too dumb to see that InputSymbol is used in both Ndfa and Builder there via the type constraint)
    #[allow(dead_code)]
    phantom: (PhantomData<InputSymbol>, PhantomData<OutputSymbol>, PhantomData<DfaType>)
}

/// Represents a state in the DFA (one or more states from the source)
#[derive(Eq, PartialEq, Hash, Clone)]
struct DfaState {
    /// List of the states from the NDFA that make up this DFA state (will be ordered if `dedupe` is called)
    source_states: Vec<StateId>
}

impl DfaState {
    /// Creates a DFA state from a set of source states
    fn create(source_states: Vec<StateId>) -> DfaState {
        let mut set = HashSet::new();
        for state in source_states {
            set.insert(state);
        }

        DfaState { source_states: Vec::from_iter(set.into_iter()) }
    }

    /// Removes any duplicate states from source_states
    fn dedupe(&mut self) {
        self.source_states.sort();
        self.source_states.dedup();
    }
}

/// Represents the transitions for a source state
struct DfaTransitions<InputSymbol, OutputSymbol: Ord> {
    /// ID of this state in the resulting DFA
    state_id: StateId,

    /// Transitions for this state (will be unique if `merge_states` is called)
    transitions: Vec<(InputSymbol, DfaState)>,

    /// The output symbols for this state (empty if this is not an accepting state)
    output: Vec<OutputSymbol>
}

impl<InputSymbol: Ord+Clone, OutputSymbol: Ord> DfaTransitions<InputSymbol, OutputSymbol> {
    /// Goes through the transitions in the object and merges the states of any with the same symbol
    fn merge_states(&mut self) {
        if self.transitions.len() > 0 {
            // Order the transitions so that if two transitions have the same input symbol, they are neighbours
            self.transitions.sort_by(|a, b| {
                let &(ref symbol_a, _) = a;
                let &(ref symbol_b, _) = b;

                symbol_a.cmp(symbol_b)
            });

            // For any transition that has a duplicate state, combine it with the previous state
            let mut new_transitions = vec![];
            new_transitions.push(self.transitions[0].clone());

            for transit_idx in 1..self.transitions.len() {
                let len          = new_transitions.len();
                let this_transit = &mut self.transitions[transit_idx];

                if new_transitions[len-1].0 == this_transit.0 {
                    // Same symbol: expand this state
                    new_transitions[len-1].1.source_states.append(&mut this_transit.1.source_states);
                } else {
                    // Different symbol: add new state
                    new_transitions.push(this_transit.clone());
                }
            }

            // Ensure all of the states are sorted and de-duplicated
            for transit in &mut new_transitions {
                transit.1.dedupe();
            }

            // Update the transitions
            self.transitions = new_transitions;
        }
    }

    ///
    /// Finds the output symbol that corresponds to this state
    ///
    /// Rule is that if there is more than one output symbol then the symbol whose value is ordered lowest is the output for this state
    ///
    fn output_symbol(&mut self) -> Option<&OutputSymbol> {
        if self.output.len() > 0 {
            self.output.sort();
            Some(&self.output[0])
        } else {
            None
        }
    }
}

impl<InputSymbol: Ord+Clone, OutputSymbol: Ord+Clone, DfaType, Ndfa: StateMachine<InputSymbol, OutputSymbol>, Builder: DfaBuilder<InputSymbol, OutputSymbol, DfaType>> 
    DfaCompiler<InputSymbol, OutputSymbol, DfaType, Ndfa, Builder> {
    ///
    /// Builds a DFA using an NDFA and a builder
    ///
    pub fn build(ndfa: Ndfa, builder: Builder) -> DfaType {
        let compiler = DfaCompiler::new(ndfa, builder);
        compiler.compile()
    }

    ///
    /// Creates a new DFA compiler using a particular builder and NDFA
    ///
    pub fn new(ndfa: Ndfa, builder: Builder) -> Self {
        DfaCompiler { ndfa: ndfa, builder: builder, phantom: (PhantomData, PhantomData, PhantomData) }
    }

    ///
    /// Compiles the NDFA into a DFA
    ///
    pub fn compile(self) -> DfaType {
        // We assume that input symbols are non-overlapping, which is not automatically the case for symbol ranges
        // You can call Ndfa.fix_overlapping_ranges() to remove any overlapping ranges from an NDFA

        // Work out the state mapping for each input symbol
        let mut states       = vec![];
        let mut known_states = HashMap::new();
        let mut to_process   = vec![];

        // All state machines have state 0 as their starting state
        let state_zero = DfaState::create(vec![0]);

        known_states.insert(state_zero.clone(), 0);
        to_process.push(state_zero);

        while let Some(state) = to_process.pop() {
            // Create a new transitions object for this state
            let mut transitions = vec![];
            let mut output      = vec![];

            for source_state in &state.source_states {
                let source_transitions = self.ndfa.get_transitions_for_state(*source_state);

                for (symbol, state) in source_transitions {
                    transitions.push((symbol, DfaState::create(vec![state])));
                }

                if let Some(source_output) = self.ndfa.output_symbol_for_state(*source_state) {
                    output.push(source_output.clone());
                }
            }

            // Merge it so that we only have one transition per symbol
            let mut dfa_transitions = DfaTransitions { state_id: states.len() as StateId, transitions: transitions, output: output };
            dfa_transitions.merge_states();

            // Process any generated states that are not already in the DFA
            for &(_, ref maybe_new_state) in &dfa_transitions.transitions {
                if !known_states.contains_key(maybe_new_state) {
                    to_process.push(maybe_new_state.clone());
                }
            }

            // Store the new state
            known_states.insert(state.clone(), dfa_transitions.state_id);
            states.push(dfa_transitions);
        }

        // Build the DFA
        let mut builder = self.builder;

        for mut dfa_state in states {
            builder.start_state();

            if let Some(output_symbol) = dfa_state.output_symbol() {
                builder.accept(output_symbol.clone());
            }

            for (symbol, target_state) in dfa_state.transitions {
                builder.transition(symbol, known_states[&target_state]);
            }
        }

        // Generate the final DFA
        builder.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::regular_pattern::*;
    use super::super::state_machine::*;
    use super::super::symbol_range_dfa::*;
    use super::super::pattern_matcher::*;
    use super::super::symbol_reader::*;

    #[test]
    fn can_create_compiler() {
        let ndfa     = "abc".into_pattern().to_ndfa("success");
        let builder  = SymbolRangeDfaBuilder::new();

        DfaCompiler::new(ndfa, builder);
    }

    #[test]
    fn can_build_dfa() {
        // Generate a state machine from the "abc" pattern
        let ndfa     = "abc".into_pattern().to_ndfa("Success");
        let builder  = SymbolRangeDfaBuilder::new();

        let state_machine = DfaCompiler::build(ndfa, builder);

        // Read back 'abc' manually
        let mut state = state_machine.start();
        let mut input = "abc".read_symbols();

        while let More(this_state) = state {
            let next_state = 
                if let Some(next_char) = input.next_symbol() {
                    this_state.next(next_char)
                } else {
                    this_state.finish()
                };

            state = next_state;
        }

        // Check for acceptance
        if let Accept(count, output) = state {
            assert!(count == 3);
            assert!(output == &"Success");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn can_build_dfa_with_overlapping_range() {
        // Generate a state machine from the "abc" pattern
        let ndfa     = (MatchRange('a', 'c').append("a")).or(MatchRange('b', 'd').append("b")).to_ndfa("Success");
        let builder  = SymbolRangeDfaBuilder::new();

        let state_machine = DfaCompiler::build(ndfa, builder);

        // Read back 'ba' manually
        let mut state = state_machine.start();
        let mut input = "ba".read_symbols();

        while let More(this_state) = state {
            let next_state = 
                if let Some(next_char) = input.next_symbol() {
                    this_state.next(next_char)
                } else {
                    this_state.finish()
                };

            state = next_state;
        }

        // Check for acceptance
        if let Accept(count, output) = state {
            assert!(count == 2);
            assert!(output == &"Success");
        } else {
            assert!(false);
        }

        // Read back 'bb' manually
        state = state_machine.start();
        input = "bb".read_symbols();

        while let More(this_state) = state {
            let next_state = 
                if let Some(next_char) = input.next_symbol() {
                    this_state.next(next_char)
                } else {
                    this_state.finish()
                };

            state = next_state;
        }

        // Check for acceptance
        if let Accept(count, output) = state {
            assert!(count == 2);
            assert!(output == &"Success");
        } else {
            assert!(false);
        }
    }
}
