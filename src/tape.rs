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
//! A tape implements `SymbolReader` so can be treated just like a normal symbol stream. However, it also adds two new methods.
//! `rewind` will go back a number of symbols (provided the tape contains that many symbols). `cut` removes any symbols prior
//! to the current one from the tape, making it impossible to rewind beyond the current point.
//!

use super::symbol_reader::*;

///
/// Rewindable symbol reader
///
pub struct Tape<Symbol: Sized, SourceReader: SymbolReader<Symbol>+Sized> {
    /// Symbol reader where items not in the buffer are read from
    read_from: SourceReader,

    /// Circular buffer where stored symbols are kept
    buffer: Vec<Option<Symbol>>,

    /// Next symbol to read from the buffer
    read_index: usize,

    /// Position of the last symbol in the buffer
    last_symbol_index: usize,

    /// Position of the first symbol in the buffer
    first_symbol_index: usize,

    /// True if the target reader has returned None
    end_of_reader: bool
}

impl<Symbol: Clone+Sized, SourceReader: SymbolReader<Symbol>> Tape<Symbol, SourceReader> {
    ///
    /// Creates a new tape from a symbol reader
    ///
    pub fn new(source: SourceReader) -> Tape<Symbol, SourceReader> {
        Tape { read_from: source, buffer: vec![None, None, None, None], read_index: 0, last_symbol_index: 0, first_symbol_index: 0, end_of_reader: false }
    }

    ///
    /// Resizes the buffer so that it can store at least one more symbol
    ///
    pub fn resize(&mut self) {
        let new_size        = self.buffer.len() * 2;
        let mut new_buffer  = Vec::with_capacity(new_size);

        // Copy all the occupied symbols from the old buffer
        let mut copy_index     = self.first_symbol_index;
        let mut new_read_index = 0;
        while copy_index != self.last_symbol_index {
            new_buffer.push(self.buffer[copy_index].clone());

            copy_index += 1;
            if copy_index >= self.buffer.len() { copy_index = 0; }

            if copy_index == self.read_index { new_read_index = new_buffer.len() }
        }

        // The current length is where the first 'new' symbol will go
        let new_last_symbol_index = new_buffer.len();

        // The rest of the buffer is filled with 'Nones'
        for _ in new_last_symbol_index..new_size {
            new_buffer.push(None);
        }

        // Replace the buffer
        self.buffer             = new_buffer;
        self.first_symbol_index = 0;
        self.last_symbol_index  = new_last_symbol_index;
        self.read_index         = new_read_index;
    }

    ///
    /// Trims the buffer so that it's not possible to rewind past the current read position
    ///
    pub fn cut(&mut self) {
        self.first_symbol_index = self.read_index;
    }

    ///
    /// Returns the number of symbols in the buffer
    ///
    #[inline]
    pub fn buffer_size(&mut self) -> usize {
        if self.first_symbol_index > self.last_symbol_index {
            (self.last_symbol_index + self.buffer.len()) - self.first_symbol_index
        } else {
            self.last_symbol_index - self.first_symbol_index
        }
    }

    ///
    /// Moves backwards by num_symbols
    ///
    pub fn rewind(&mut self, num_symbols: usize) {
        // Do nothing if we're over the maximum length of the buffer
        if num_symbols > self.buffer_size() {
            panic!("Can't rewind beyond the start of the tape");
        }

        // Move backwards num_symbols
        let mut new_read_index = (self.read_index+self.buffer.len()) - num_symbols;
        if new_read_index >= self.buffer.len() { new_read_index -= self.buffer.len(); }

        self.read_index = new_read_index;
    }
}

impl<Symbol: Clone+Sized, Reader: SymbolReader<Symbol>+Sized> SymbolReader<Symbol> for Tape<Symbol, Reader> {
    fn next_symbol(&mut self) -> Option<Symbol> {
        if self.read_index == self.last_symbol_index {
            // If the source reader has ended, there's nothing more to read
            if self.end_of_reader {
                return None;
            }

            // At end of buffer: need to fill it some more
            let maybe_symbol = self.read_from.next_symbol();
            match maybe_symbol {
                None => {
                    // Mark this stream as having finished (we won't read from the source stream any more)
                    self.end_of_reader = true;
                    return None;
                },

                _ => {
                    // Resize the buffer if needed
                    if (self.last_symbol_index+1)%self.buffer.len() == self.first_symbol_index {
                        self.resize();
                    }

                    // Store the next symbol (we'll immediately read it from the buffer later on)
                    self.buffer[self.last_symbol_index] = maybe_symbol;

                    self.last_symbol_index += 1;
                    if self.last_symbol_index >= self.buffer.len() { self.last_symbol_index = 0; }
                }
            }
        }

        // Result is whatever is in the buffer at the read position
        let result = self.buffer[self.read_index].clone();

        self.read_index += 1;
        if self.read_index >= self.buffer.len() { self.read_index = 0; }

        result
    }
}

#[cfg(test)]
mod test {
    use super::super::*;

    #[test]
    fn can_read_from_tape() {
        let source_vec    = vec![1,2,3,4,5,6];
        let source_stream = source_vec.read_symbols();
        let mut tape      = Tape::new(source_stream);

        assert!(tape.next_symbol() == Some(1));
        assert!(tape.next_symbol() == Some(2));
        assert!(tape.next_symbol() == Some(3));
        assert!(tape.next_symbol() == Some(4));
        assert!(tape.next_symbol() == Some(5));
        assert!(tape.next_symbol() == Some(6));
        assert!(tape.next_symbol() == None);
    }

    #[test]
    fn can_rewind_tape() {
        let source_vec    = vec![1,2,3,4,5,6];
        let source_stream = source_vec.read_symbols();
        let mut tape      = Tape::new(source_stream);

        assert!(tape.next_symbol() == Some(1));
        assert!(tape.next_symbol() == Some(2));
        assert!(tape.next_symbol() == Some(3));
        tape.rewind(3);

        assert!(tape.next_symbol() == Some(1));
        assert!(tape.next_symbol() == Some(2));
        assert!(tape.next_symbol() == Some(3));
        assert!(tape.next_symbol() == Some(4));
        assert!(tape.next_symbol() == Some(5));
        assert!(tape.next_symbol() == Some(6));
        assert!(tape.next_symbol() == None);

        tape.rewind(4);
        assert!(tape.next_symbol() == Some(3));
        assert!(tape.next_symbol() == Some(4));
        assert!(tape.next_symbol() == Some(5));
        assert!(tape.next_symbol() == Some(6));
        assert!(tape.next_symbol() == None);
    }

    #[test]
    fn can_cut_tape() {
        let source_vec    = vec![1,2,3,4,5,6,7,8,9];
        let source_stream = source_vec.read_symbols();
        let mut tape      = Tape::new(source_stream);

        assert!(tape.next_symbol() == Some(1));
        assert!(tape.next_symbol() == Some(2));
        assert!(tape.next_symbol() == Some(3));
        tape.cut();

        assert!(tape.next_symbol() == Some(4));
        assert!(tape.next_symbol() == Some(5));
        assert!(tape.next_symbol() == Some(6));
        tape.rewind(3);

        assert!(tape.next_symbol() == Some(4));
        assert!(tape.next_symbol() == Some(5));
        assert!(tape.next_symbol() == Some(6));
        assert!(tape.next_symbol() == Some(7));
        assert!(tape.next_symbol() == Some(8));
        assert!(tape.next_symbol() == Some(9));
        assert!(tape.next_symbol() == None);

        tape.rewind(3);
        tape.cut();
        assert!(tape.next_symbol() == Some(7));
        assert!(tape.next_symbol() == Some(8));
        assert!(tape.next_symbol() == Some(9));
        assert!(tape.next_symbol() == None);

        tape.rewind(2);
        assert!(tape.next_symbol() == Some(8));
        assert!(tape.next_symbol() == Some(9));
        assert!(tape.next_symbol() == None);
    }
}
