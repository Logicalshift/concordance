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
//! # DFA compiler
//!
//! The DFA compiler converts NDFAs into DFAs, using a DFA builder.
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
struct DfaTransitions<InputSymbol> {
    transitions: Vec<(InputSymbol, DfaState)>
}

impl<InputSymbol: Ord+Clone> DfaTransitions<InputSymbol> {
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
}

impl<InputSymbol: Ord+Clone, OutputSymbol, DfaType, Ndfa: StateMachine<InputSymbol, OutputSymbol>, Builder: DfaBuilder<InputSymbol, OutputSymbol, DfaType>> 
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
        // TODO: Convert overlapping symbol ranges to non-overlapping symbol ranges
        // This is only needed for input symbol types that implement SymbolRange

        // Work out the state mapping for each input symbol
        let mut states: HashMap<DfaState, DfaTransitions<InputSymbol>> = HashMap::new();
        let mut to_process  = vec![];

        // All state machines have state 0 as their starting state
        let state_zero = DfaState::create(vec![0]);

        states.insert(state_zero.clone(), DfaTransitions { transitions: vec![] });
        to_process.push(state_zero);

        // Assign final IDs to the states

        // TODO: Build the DFA

        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::regular_pattern::*;
    use super::super::state_machine::*;
    use super::super::symbol_range_dfa::*;

    #[test]
    fn can_create_compiler() {
        let ndfa     = "abc".into_pattern().to_ndfa("success");
        let builder  = SymbolRangeDfaBuilder::new();

        DfaCompiler::new(ndfa, builder);
    }

    #[test]
    fn can_build_dfa() {
        let ndfa     = "abc".into_pattern().to_ndfa("success");
        let builder  = SymbolRangeDfaBuilder::new();

        DfaCompiler::build(ndfa, builder);
    }
}
