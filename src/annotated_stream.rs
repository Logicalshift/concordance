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
    }
}
