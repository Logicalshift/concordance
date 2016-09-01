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
//! NDFA is a library for working with deterministic and non-deterministic finite-state automata.
//!

pub use self::countable::*;
pub use self::symbol_range::*;
pub use self::symbol_reader::*;
pub use self::state_machine::*;
pub use self::pattern_matcher::*;
pub use self::empty_state_machine::*;
pub use self::ndfa::*;
pub use self::regular_pattern::*;
pub use self::dfa_builder::*;
pub use self::symbol_range_dfa::*;
pub use self::dfa_compiler::*;

pub mod countable;
pub mod symbol_range;
pub mod symbol_reader;
pub mod state_machine;
pub mod overlapping_symbols;
pub mod pattern_matcher;
pub mod empty_state_machine;
pub mod ndfa;
pub mod regular_pattern;
pub mod dfa_builder;
pub mod symbol_range_dfa;
pub mod dfa_compiler;
