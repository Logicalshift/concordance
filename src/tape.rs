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
//! A tape is a symbol reader that can be rewound. 
//!
//! It can be useful for matching a pattern repeatedly against an input, as the state machine may read many extra symbols 
//! before it finds a match at an earlier position. To find another match after the first one, it's necessary to rewind
//! the stream.
//!

use super::symbol_reader::*;

///
/// Rewindable symbol reader
///
pub struct Tape<Symbol: Sized, SourceReader: SymbolReader<Symbol>> {
    /// Symbol reader where items not in the buffer are read from
    read_from: SourceReader,

    /// Circular buffer where stored symbols are kept
    buffer: Vec<Symbol>,

    /// Next symbol to read from the buffer
    read_index: usize,

    /// Position of the last symbol in the buffer
    last_symbol_index: usize,

    /// Position of the first symbol in the buffer
    first_symbol_index: usize,
}

impl<Symbol: Sized, SourceReader: SymbolReader<Symbol>> Tape<Symbol, SourceReader> {
    fn new(source: SourceReader) -> Tape<Symbol, SourceReader> {
        Tape { read_from: source, buffer: vec![], read_index: 0, last_symbol_index: 0, first_symbol_index: 0 }
    }
}