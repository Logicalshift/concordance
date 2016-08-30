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

use super::dfa_builder::*;
use super::state_machine::*;

///
/// Builds a deterministic finite automaton from a NDFA
///
pub struct DfaCompiler<InputSymbol: PartialOrd, OutputSymbol, DfaType, Ndfa: StateMachine<InputSymbol, OutputSymbol>, Builder: DfaBuilder<InputSymbol, OutputSymbol, DfaType>> {
    /// State machine that is to be compiled
    ndfa: Ndfa,

    /// Builder where the state machine should be generated
    builder: Builder,

    // Phantom data to poke Rust's type system (it's too dumb to see that InputSymbol is used in both Ndfa and Builder there via the type constraint)
    #[allow(dead_code)]
    phantom: (PhantomData<InputSymbol>, PhantomData<OutputSymbol>, PhantomData<DfaType>)
}

impl<InputSymbol: PartialOrd, OutputSymbol, DfaType, Ndfa: StateMachine<InputSymbol, OutputSymbol>, Builder: DfaBuilder<InputSymbol, OutputSymbol, DfaType>> 
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
