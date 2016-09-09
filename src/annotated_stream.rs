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
//! A token tree represents the results of applying the tokenizer to an input string. It can output its contents as a series of tokens,
//! which makes it suitable for repeatedly performing pattern matches. Matching against an already tokenized string produces a tree of
//! matches.
//!

use std::ops::Range;

///
/// An annotated stream represents an original stream, with ranges tagged with tokens. This can be used to map between a
/// tokenized stream and the original characters.
///
pub struct AnnotatedStream<InputType, TokenType> {
    /// The original stream that was tokenized
    original: Vec<InputType>,

    /// The tokenized version and where in the original they appear
    tokenized: Vec<(TokenType, Range<usize>)>
}
