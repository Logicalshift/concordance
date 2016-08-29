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
//! # Matcher
//!
//! The Matcher trait is implemented by objects that can match patterns against the left-hand side of a stream. It's a fairly
//! baseline implementation: it's up to the caller to implement things like rewinding in order to perform tokenisation. That is,
//! Matchers are greedy and may (indeed, are likely to) consume more characters than the longest match while trying to find
//! a longer one.
//!

///
/// Matcher that can read an input stream of type `Symbol` and find the longest matching pattern, which it will identify using
/// `OutputSymbol`
///
pub trait PatternMatcher<InputSymbol, OutputSymbol> {
}
