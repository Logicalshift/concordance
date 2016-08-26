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

use super::state_machine::*;

///
/// Represents a state machine with no states
///
pub struct EmptyStateMachine {
}

impl<InputSymbol, OutputSymbol> StateMachine<InputSymbol, OutputSymbol> for EmptyStateMachine {
    ///
    /// Returns the number of states in this state machine
    ///
    fn count_states(&self) -> StateId {
        1
    }

    ///
    /// Returns the transitions for a particular symbol 
    ///
    fn get_transitions_for_state(&self, _state: StateId) -> Vec<(InputSymbol, StateId)> {
        vec![]
    }

    ///
    /// If a state is an accepting state, then this returns the output symbol that should be produced if this is the longest match
    ///
    fn output_symbol_for_state(&self, _state: StateId) -> Option<OutputSymbol> {
        None
    }
}
