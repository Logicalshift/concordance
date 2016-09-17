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
//! stream at the leaves. This can be used to represent the result of parsing an input.
//!

use super::countable::*;
use super::symbol_reader::*;
use super::annotated_stream::*;
use super::symbol_range_dfa::*;

///
/// Represents a stream that has been decomposed into a tree
///
#[derive(Clone)]
pub struct TreeStream<InputSymbol, TokenType> {
    /// The original input for this stream
    input: Vec<InputSymbol>,

    /// How the original input stream maps onto tokens
    tokens: AnnotatedStream<TokenType>,

    /// The hierarchy of annotations. As a tree, the 'root' is considered to be the last entry in this list (it's built from the bottom up)
    annotations: Vec<AnnotatedStream<TokenType>>
}

impl<InputSymbol: Clone+Ord+Countable, TokenType: Clone+Ord+Countable+'static> TreeStream<InputSymbol, TokenType> {
    ///
    /// Creates a new tree stream from a tokenized stream
    ///
    pub fn new_with_tokens(input: Vec<InputSymbol>, tokens: AnnotatedStream<TokenType>) -> Self {
        TreeStream { input: input, tokens: tokens, annotations: vec![] }
    }

    ///
    /// Reads the input that made up this tree stream
    ///
    pub fn read_input<'a>(&'a self) -> Box<SymbolReader<InputSymbol>+'a> {
        Box::new(self.input.read_symbols())
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
            Box::new(VecReader::from_vec(tokens))
        }
    }

    ///
    /// Reads a particular level of the tree as a stream
    ///
    /// `0` is the top-most level of the tree, and `depth()-1` represent the bottom-most level (the level containing the tokens)
    ///
    pub fn read_level<'a>(&'a self, depth: usize) -> Box<SymbolReader<TokenType>+'a> {
        Box::new(self.read_level_tokens(depth).map_symbols(|token| token.output))
    }

    ///
    /// Performs pattern matching on the 'top level', adding another level to the tree. Returns the number of symbols in the new stream
    ///
    /// By matching a series of tokens against a regular pattern, we can further reduce a tokenized string, and build up a tree
    /// representation of the string's contents. 'Holes' that do not match the pattern allow the tree to have variable depth:
    ///
    pub fn tokenize_top_level(&mut self, dfa: &SymbolRangeDfa<TokenType, TokenType>) -> usize {
        // Read the tokens at the top level
        let next_level  = AnnotatedStream::from_tokenizer(dfa, self.read_level(0));
        let num_matched = next_level.output_len();

        self.annotations.push(next_level);

        num_matched
    }
}

///
/// Represents a node in a tree stream
///
pub struct TreeNode<'a, InputSymbol: 'a, OutputSymbol: 'a> {
    /// The tree that this node is a part of
    tree: &'a TreeStream<InputSymbol, OutputSymbol>,

    /// The distance from the bottom of the tree that this node represents (0 = tokens, 1+ = a leven in the annotations)
    level: usize,

    /// The token representing the 'root' of this tree
    token: Token<OutputSymbol>
}

impl<'a, InputSymbol: 'a, OutputSymbol: Clone+'a> TreeNode<'a, InputSymbol, OutputSymbol> {
    ///
    /// Retrieves the output symbol represented by this tree node
    ///
    pub fn get_symbol(&self) -> OutputSymbol {
        self.token.output.clone()
    }

    ///
    /// Retrieves the child nodes of this node (the empty set if this is a leaf node)
    ///
    pub fn get_children(&self) -> Vec<TreeNode<'a, InputSymbol, OutputSymbol>> {
        if self.level == 0 {
            // Level 0 is at the bottom
            vec![]
        } else {
            unimplemented!()
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;

    #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
    enum TestToken {
        Digit,
        Identifier,
        Operator,
        Whitespace,

        Expression
    }

    impl Countable for TestToken {
        fn next(&self) -> Self {
            match *self {
                TestToken::Digit        => TestToken::Identifier,
                TestToken::Identifier   => TestToken::Operator,
                TestToken::Operator     => TestToken::Whitespace,
                TestToken::Whitespace   => TestToken::Expression,
                TestToken::Expression   => TestToken::Digit
            }
        }

        fn prev(&self) -> Self {
            match *self {
                TestToken::Digit        => TestToken::Expression,
                TestToken::Identifier   => TestToken::Digit,
                TestToken::Operator     => TestToken::Identifier,
                TestToken::Whitespace   => TestToken::Operator,
                TestToken::Expression   => TestToken::Whitespace
            }
        }
    }

    fn create_simple_tokenizer() -> SymbolRangeDfa<char, TestToken> {
        let mut token_matcher = TokenMatcher::new();

        token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(1), TestToken::Digit);
        token_matcher.add_pattern(MatchRange('a', 'z').repeat_forever(1), TestToken::Identifier);
        token_matcher.add_pattern("+".into_pattern(), TestToken::Operator);
        token_matcher.add_pattern(" ".repeat_forever(1), TestToken::Whitespace);

        token_matcher.prepare_to_match()
    }

    fn create_expression_parser() -> SymbolRangeDfa<TestToken, TestToken> {
        let mut token_matcher = TokenMatcher::new();

        token_matcher.add_pattern(Match(vec![TestToken::Digit]), TestToken::Expression);
        token_matcher.add_pattern(Match(vec![TestToken::Identifier]), TestToken::Expression);
        token_matcher.add_pattern(Match(vec![TestToken::Expression, TestToken::Operator, TestToken::Expression]), TestToken::Expression);

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

    #[test]
    pub fn can_reduce_expression() {
        let tokenizer          = create_simple_tokenizer();
        let parser             = create_expression_parser(); 
        let tokenized_stream   = AnnotatedStream::from_tokenizer(&tokenizer, "a+1".read_symbols());
        let mut tokenized_tree = TreeStream::new_with_tokens(tokenized_stream);

        let num_expressions_1  = tokenized_tree.tokenize_top_level(&parser);
        let num_expressions_2  = tokenized_tree.tokenize_top_level(&parser);

        assert!(tokenized_tree.read_level(2).to_vec() == vec![TestToken::Identifier, TestToken::Operator, TestToken::Digit]);
        assert!(tokenized_tree.read_level(1).to_vec() == vec![TestToken::Expression, TestToken::Operator, TestToken::Expression]);
        assert!(tokenized_tree.read_level(0).to_vec() == vec![TestToken::Expression]);
        assert!(num_expressions_1 == 2);            // 'Expression Operator Expression'
        assert!(num_expressions_2 == 1);            // 'Expression'
    }

    #[test]
    pub fn can_reduce_longer_expression() {
        let tokenizer          = create_simple_tokenizer();
        let parser             = create_expression_parser(); 
        let tokenized_stream   = AnnotatedStream::from_tokenizer(&tokenizer, "a+1+b".read_symbols());
        let mut tokenized_tree = TreeStream::new_with_tokens(tokenized_stream);

        let num_expressions_1  = tokenized_tree.tokenize_top_level(&parser);
        let num_expressions_2  = tokenized_tree.tokenize_top_level(&parser);
        let num_expressions_3  = tokenized_tree.tokenize_top_level(&parser);

        assert!(tokenized_tree.read_level(3).to_vec() == vec![TestToken::Identifier, TestToken::Operator, TestToken::Digit, TestToken::Operator, TestToken::Identifier]);
        assert!(tokenized_tree.read_level(2).to_vec() == vec![TestToken::Expression, TestToken::Operator, TestToken::Expression, TestToken::Operator, TestToken::Expression]);
        assert!(tokenized_tree.read_level(1).to_vec() == vec![TestToken::Expression, TestToken::Operator, TestToken::Expression]);
        assert!(tokenized_tree.read_level(0).to_vec() == vec![TestToken::Expression]);
        assert!(num_expressions_1 == 3);            // 'Expression Operator Expression Operator Expression'
        assert!(num_expressions_2 == 1);            // 'Expression Operator Expression'
        assert!(num_expressions_3 == 1);            // 'Expression'
    }
}
