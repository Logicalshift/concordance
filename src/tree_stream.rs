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

impl<InputSymbol: Clone+Ord+Countable, TokenType: Clone+Ord+Countable+'static> TreeStream<InputSymbol, TokenType> {
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

    ///
    /// Works out the depth of this tree
    ///
    pub fn depth(&self) -> usize {
        self.annotations.len()+1
    }

    ///
    /// Reads a the tokens on a particular level of the tree as a stream
    ///
    /// `0` is the top-most level of the tree, and `depth()-1` represent the bottom-most level (the level containing the tokens)
    ///
    pub fn read_level_tokens<'a>(&'a self, depth: usize) -> Box<SymbolReader<Token<TokenType>>+'a> {
        if depth == self.annotations.len() {
            // Just the tokens, mapping onto the original input stream
            self.tokens.read_tokens()
        } else {
            // Build up a symbol reader from the output and input of this stream
            let level  = (self.annotations.len()-1) - depth;
            let stream = &self.annotations[level];

            let mut tokens   = vec![];
            let mut last_pos = 0..0;
            let mut reader   = stream.read_tokens();

            loop {
                if let Some(level_token) = reader.next_symbol() {
                    // Append input symbols if there is a gap
                    if last_pos.end != level_token.location.start {
                        let gap_range         = last_pos.end..level_token.location.start;
                        let from_higher_level = stream.input_for_range(gap_range.clone());

                        for pos in gap_range.clone() {
                            tokens.push(Token::new(pos..pos+1, from_higher_level[pos-gap_range.start].clone()));
                        }
                    }

                    // Update the last pos so we can detect gaps after it
                    last_pos = level_token.location.clone();

                    // Add this token
                    tokens.push(level_token);
                } else {
                    // End of stream
                    break;
                }
            }

            // Append input tokens from the end of the stream
            let end_pos  = stream.input_len();
            let end_gap  = last_pos.end..end_pos;
            let from_end = stream.input_for_range(end_gap.clone());

            for pos in end_gap.clone() {
                tokens.push(Token::new(pos..pos+1, from_end[pos-end_gap.start].clone()));
            }

            // Result is a new vector stream
            unimplemented!();
        }
    }

    ///
    /// Reads a particular level of the tree as a stream
    ///
    /// `0` is the top-most level of the tree, and `depth()-1` represent the bottom-most level (the level containing the tokens)
    ///
    pub fn read_level<'a>(&'a self, depth: usize) -> Box<SymbolReader<TokenType>+'a> {
        if depth == self.annotations.len() {
            // Just the tokens; this level is flat
            self.tokens.read_output()
        } else {
            // Need to read 'through' the tree to deal with gaps to create the whole stream at this level
            unimplemented!()
        }
    }
}

///
/// Symbol reader that reads a 'level' of a tree
///
/// If the level has gaps in it, then those are filled in using the tokens from the level above
///
struct LevelReader<'a, InputSymbol: 'a, TokenType: 'static> {
    // The source treestream that is being read
    source: &'a TreeStream<InputSymbol, TokenType>,

    // The stack of levels being processed: the readers and the last token seen from them
    stack: Vec<(Box<SymbolReader<Token<TokenType>>+'a>, Option<Token<TokenType>>)>
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

    impl Countable for TestToken {
        fn next(&self) -> Self {
            match *self {
                TestToken::Digit        => TestToken::Identifier,
                TestToken::Identifier   => TestToken::Whitespace,
                TestToken::Whitespace   => TestToken::Digit
            }
        }

        fn prev(&self) -> Self {
            match *self {
                TestToken::Digit        => TestToken::Whitespace,
                TestToken::Identifier   => TestToken::Digit,
                TestToken::Whitespace   => TestToken::Identifier
            }
        }
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
        let tokenized_stream = AnnotatedStream::from_tokenizer(&tokenizer, "a+1".read_symbols());
        let tokenized_tree   = TreeStream::new_with_tokens(tokenized_stream);

        assert!(tokenized_tree.depth() == 1);
        assert!(tokenized_tree.read_input().to_vec() == vec!['a', '+', '1']);
    }
}
