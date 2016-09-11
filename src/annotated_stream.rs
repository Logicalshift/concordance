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

use super::countable::*;
use super::tokenizer::*;
use super::symbol_reader::*;
use super::symbol_range_dfa::*;

///
/// An annotated stream represents an original stream, with ranges tagged with tokens. This can be used to map between a
/// tokenized stream and the original characters.
///
pub struct AnnotatedStream<InputType, TokenType> {
    /// The original stream that was tokenized
    original: Vec<InputType>,

    /// The tokenized version and where in the original they appear, in order
    tokenized: Vec<(TokenType, Range<usize>)>
}

impl<InputSymbol: Clone+Ord+Countable, OutputSymbol: Clone+Ord+'static> AnnotatedStream<InputSymbol, OutputSymbol> {
    ///
    /// Given a stream and a DFA, tokenizes the stream and annotates it with the appropriate tokens
    ///
    pub fn from_tokenizer<Reader: SymbolReader<InputSymbol>>(dfa: &SymbolRangeDfa<InputSymbol, OutputSymbol>, reader: &mut Reader) -> AnnotatedStream<InputSymbol, OutputSymbol> {
        // Capture the contents of the original reader (we store them in the result)
        let     original = reader.to_vec();
        let mut tokens   = vec![];

        {
            // Create a new reader to read our captured symbols
            let token_reader  = original.read_symbols();
            let mut tokenizer = Tokenizer::new_prepared(token_reader, dfa);

            // Start tokenizing
            let mut pos: usize = 0;

            loop {
                // Tokenize the next symbol
                let next_token  = tokenizer.next_symbol();
                let final_pos   = tokenizer.get_source_position();

                if let Some(output) = next_token {
                    // Got a token
                    tokens.push((output, pos..final_pos));

                    // Next token begins after this one
                    pos = final_pos;
                } else if !tokenizer.at_end_of_reader() {
                    // Skip tokens that don't form a match (returned none + not at the end of the reader)
                    // Try again in case there are further tokens
                    pos += 1;
                    tokenizer.skip_input();
                } else {
                    // Reached the end of the input
                    break;
                }
            }
        }

        // Annotated stream is ready
        AnnotatedStream { original: original, tokenized: tokens }
    }

    ///
    /// Reads the original stream for the original input for this stream
    ///
    pub fn read_input<'a>(&'a self) -> Box<SymbolReader<InputSymbol> + 'a> {
        Box::new(self.original.read_symbols())
    }

    ///
    /// Reads the tokenized output stream (only for the symbols that were recognised)
    ///
    pub fn read_output<'a>(&'a self) -> Box<SymbolReader<OutputSymbol> + 'a> {
        let full_output  = self.tokenized.read_symbols();
        let only_symbols = full_output.map_symbols(|(token, _)| token); 

        Box::new(only_symbols)
    }

    ///
    /// Reads the annotated stream as a series of tokens
    ///
    pub fn read_tokens<'a>(&'a self) -> Box<SymbolReader<Token<'a, InputSymbol, OutputSymbol>> + 'a> {
        let full_output = self.tokenized.read_symbols();
        let with_tokens = full_output.map_symbols(move |(output, location)| {
            Token {
                location: location.clone(),
                matched:  &self.original[location],
                output:   output
            }
        });

        Box::new(with_tokens)
    }

    ///
    /// Finds the index into the tokenized list of the token corresponding to the specified position
    ///
    /// Returns Err(index_after) if there's no range corresponding to the position in this stream
    ///
    fn find_token_index(&self, position: usize) -> Result<usize, usize> {
        // Try to find the specified position: assumes the tokens are in order (which they are if we generated this stream from left to right)
        let found = self.tokenized.binary_search_by(|&(_, ref location)| location.start.cmp(&position));
        
        // We're only searching on the start position: if we don't find it exactly, then the token might be in the preceding index
        match found {
            Ok(index) => Ok(index),

            Err(index) => {
                if index == 0 {
                    Err(0)
                } else {
                    if self.tokenized[index-1].1.end > position {
                        Ok(index-1)
                    } else {
                        Err(index)
                    }
                }
            }
        }
    }

    ///
    /// Finds the token at the specified position in this stream
    ///
    pub fn find_token<'a>(&'a self, position: usize) -> Option<Token<'a, InputSymbol, OutputSymbol>> {
        let found_index = self.find_token_index(position).ok();

        // Build a token for this location
        found_index.map(move |index| {
            let (ref output, ref location) = self.tokenized[index];

            Token { 
                location: location.clone(),
                output:   output.clone(),
                matched:  &self.original[location.clone()]
            }
        })
    }
}

///
/// A token represents an individual item in an annotated stream
///
#[derive(Eq, PartialEq, Clone)]
pub struct Token<'a, InputSymbol: 'a, OutputSymbol> {
    /// Where this token appears in the output
    pub location: Range<usize>,

    /// The input symbols that were matched for this token
    pub matched: &'a [InputSymbol],

    /// The output symbol that was matched for this token
    pub output: OutputSymbol
}

#[cfg(test)]
mod test {
    pub use super::super::*;

    #[test]
    fn can_annotate_stream() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            Digit,
            Whitespace
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(0), TestToken::Digit);
        token_matcher.add_pattern(" ".repeat_forever(0), TestToken::Whitespace);

        let dfa   = token_matcher.prepare_to_match();
        let input = "12 42 13";

        let annotated = AnnotatedStream::from_tokenizer(&dfa, &mut input.read_symbols());

        let annotated_input  = annotated.read_input().to_vec();
        let annotated_output = annotated.read_output().to_vec();
        let annotated_tokens = annotated.read_tokens().to_vec();

        assert!(annotated_input == vec!['1', '2', ' ', '4', '2', ' ', '1', '3']);
        assert!(annotated_output == vec![TestToken::Digit, TestToken::Whitespace, TestToken::Digit, TestToken::Whitespace, TestToken::Digit]);

        assert!(annotated_tokens[0].location.start == 0);
        assert!(annotated_tokens[0].location.end == 2);
        assert!(annotated_tokens[0].output == TestToken::Digit);
        assert!(annotated_tokens[0].matched == &['1', '2']);
    }

    #[test]
    fn can_find_token() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            Digit,
            Whitespace
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(0), TestToken::Digit);
        token_matcher.add_pattern(" ".repeat_forever(0), TestToken::Whitespace);

        let dfa   = token_matcher.prepare_to_match();
        let input = "12 42 13";

        let annotated = AnnotatedStream::from_tokenizer(&dfa, &mut input.read_symbols());

        let fortytwo = annotated.find_token(4).expect("No token");
        let whitespace = annotated.find_token(5).expect("No token");

        assert!(fortytwo.location.start == 3);
        assert!(fortytwo.location.end == 5);
        assert!(fortytwo.output == TestToken::Digit);
        assert!(fortytwo.matched == &['4', '2']);

        assert!(whitespace.location.start == 5);
        assert!(whitespace.location.end == 6);
        assert!(whitespace.output == TestToken::Whitespace);
        assert!(whitespace.matched == &[' ']);
    }
}
