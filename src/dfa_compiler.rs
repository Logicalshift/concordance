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

use super::dfa_builder::*;
use super::state_machine::*;

///
/// Builds a deterministic finite automaton from a NDFA
///
pub struct DfaCompiler<'a, InputSymbol: PartialOrd+'a, OutputSymbol: 'a, DfaType: 'a> {
    //
    // Would like to do this without the references and weird lifetime stuff. There's a bunch of limitations about what you can do
    // when calling references to traits vs concrete types that make designing rust code very annoying. However, if you have a struct
    // declared thusly: struct Foo<Bar, Quux: Baz<Bar>> and only use a field of type Quux then it'll moan that 'Bar' is unused, which
    // is incorrect (it's used wherever Quux is used by definition), so you have to declare Foo<Bar> and use a reference to Baz<Bar>
    // which means a lot of code won't work due to the whole mess around things needing to be sized.
    //
    // This basically means we can't declare ndfa or builder here using their real types. PhantomData also doesn't work because rust
    // isn't smart enough to work out that if it knows the type of the state machine it also knows what InputSymbol and OutputSymbol
    // are.
    //

    ndfa: &'a StateMachine<InputSymbol, OutputSymbol>,
    builder: &'a mut DfaBuilder<InputSymbol, OutputSymbol, DfaType>,
}

impl<'a, InputSymbol: PartialOrd+'a, OutputSymbol: 'a, DfaType: 'a> DfaCompiler<'a, InputSymbol, OutputSymbol, DfaType> {
    pub fn new(ndfa: &'a StateMachine<InputSymbol, OutputSymbol>, builder: &'a mut DfaBuilder<InputSymbol, OutputSymbol, DfaType>) -> Self {
        DfaCompiler { ndfa: ndfa, builder: builder }
    }
}
