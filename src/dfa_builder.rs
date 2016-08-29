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
//! # DFA builder
//!
//! The DFA builder trait is implemented by classes that initialise DFAs. 
//!

use super::state_machine::*;
use super::matcher::*;

///
/// Class that can build a particular type of DFA
///
pub trait DfaBuilder<InputSymbol, OutputSymbol, DfaType: Matcher<InputSymbol, OutputSymbol>> {
    ///
    /// Starts the next state for this DFA
    ///
    /// When this is first called, the DFA will enter state 0, then state 1, etc. If this hasn't been called yet then the DFA is not
    /// in a valid state and the other calls cannot be made.
    ///
    fn start_state(&mut self);

    ///
    /// Adds a transition to the current state
    ///
    /// Any input symbol can appear exactly once in a state, and must not overlap any other input symbol. Transitions must be in input
    /// symbol order.
    ///
    fn transition(&mut self, symbol: InputSymbol, target_state: StateId);

    ///
    /// Sets the current state as an accepting state and sets the output symbol that will be produced if this is the longest match
    ///
    fn accept(&mut self, symbol: OutputSymbol);

    ///
    /// Finishes building the DFA and returns the matcher for the pattern it represents
    ///
    fn build(self) -> DfaType;
}
