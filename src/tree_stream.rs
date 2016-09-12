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
//! A tree stream is the result of annotating an annotated stream. This forms a tree structure with the original input
//! stream at the leaves.
//!

use super::countable::*;
use super::annotated_stream::*;

///
/// Represents a stream that has been decomposed into a tree
///
#[derive(Clone)]
pub struct TreeStream<InputSymbol, TokenType> {
    /// How the original input stream maps onto tokens
    tokens: AnnotatedStream<InputSymbol, TokenType>,

    /// The hierarchy of annotations. As a tree, the 'root' is considered to be the last entry in this list (it's built from the bottom up)
    annotations: Vec<AnnotatedStream<TokenType, TokenType>>
}

impl<InputSymbol: Clone+Ord+Countable, TokenType: Clone+Ord+'static> TreeStream<InputSymbol, TokenType> {
    ///
    /// Creates a new tree stream from a tokenized stream
    ///
    pub fn new_with_tokens(tokens: AnnotatedStream<InputSymbol, TokenType>) -> Self {
        TreeStream { tokens: tokens, annotations: vec![] }
    }
}
