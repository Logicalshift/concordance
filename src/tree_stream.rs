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
use super::symbol_reader::*;
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

    ///
    /// Reads the input that made up this tree stream
    ///
    pub fn read_input<'a>(&'a self) -> Box<SymbolReader<InputSymbol>+'a> {
        self.tokens.read_input()
    }
}

#[cfg(test)]
mod test {
    use super::super::*;

    #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
    enum TestToken {
        Digit,
        Identifier,
        Whitespace
    }

    fn create_simple_tokenizer() -> SymbolRangeDfa<char, TestToken> {
        let mut token_matcher = TokenMatcher::new();

        token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(1), TestToken::Digit);
        token_matcher.add_pattern(MatchRange('a', 'z').repeat_forever(1), TestToken::Identifier);
        token_matcher.add_pattern(" ".repeat_forever(1), TestToken::Whitespace);

        token_matcher.prepare_to_match()
    }

    #[test]
    pub fn can_iterate_over_base_stream() {
        let tokenizer        = create_simple_tokenizer();
        let tokenized_stream = AnnotatedStream::from_tokenizer(&tokenizer, &mut "a+1".read_symbols());
        let tokenized_tree   = TreeStream::new_with_tokens(tokenized_stream);

        assert!(tokenized_tree.read_input().to_vec() == vec!['a', '+', '1']);
    }
}
