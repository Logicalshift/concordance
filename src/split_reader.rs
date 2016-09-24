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
//! A split symbol reader is used when two targets want to read from the same stream.
//!

use std::cell::*;
use std::rc::*;
use std::collections::*;

use super::symbol_reader::*;

///
/// Shares a symbol reader between two targets
///
pub struct SplitSymbolReader<'a, Symbol: Clone+'a> {
    // The buffer for this reader
    buffer: Rc<RefCell<SplitSymbolReaderBuffer<'a, Symbol>>>,

    // The ID of this reader within the buffer
    reader_id: usize
}

///
/// Stores the shared data between two symbol readers
///
/// Rust really does a poor job of making it so you can share things. The issue is that you can only borrow something
/// mutably once, so we need to faff around with RefCells and so on, which add overhead solely for dealing with language
/// semantics rather than actual issues with sharing data.
///
struct SplitSymbolReaderBuffer<'a, Symbol: 'a> {
    /// The underlying symbol reader
    symbol_reader: &'a mut SymbolReader<Symbol>,

    /// Positions for the various split readers that are using this object, relative to the buffer
    positions: Vec<usize>,

    /// Buffer of symbols that are waiting to be consumed by other readers
    buffer: VecDeque<Option<Symbol>>
}

impl<'a, Symbol: Clone> SplitSymbolReaderBuffer<'a, Symbol> {
    fn new(reader: &'a mut SymbolReader<Symbol>) -> SplitSymbolReaderBuffer<'a, Symbol> {
        SplitSymbolReaderBuffer { symbol_reader: reader, positions: vec![], buffer: VecDeque::new() }
    }

    ///
    /// Clears out any buffered items that have been read by all readers
    ///
    fn clear_buffer(&mut self) {
        let lowest_pos = self.positions.iter()
            .min()
            .map(|x| *x);

        if let Some(lowest_pos) = lowest_pos {
            if lowest_pos < usize::max_value() {
                for _ in 0..lowest_pos {
                    self.buffer.pop_front();
                }

                for pos in self.positions.iter_mut() {
                    *pos -= lowest_pos;
                }
            }
        }
    }

    ///
    /// Reads a symbol for the reader with a particular ID
    ///
    fn read(&mut self, reader_id: usize) -> Option<Symbol> {
        let buf_pos = self.positions[reader_id];

        if buf_pos >= self.buffer.len() {
            // This reader is at the end of the buffer, so just read the next symbol
            let next_symbol = self.symbol_reader.next_symbol();
            self.buffer.push_back(next_symbol.clone());

            self.positions[reader_id] = buf_pos+1;

            // If this reader was at the start of the buffer, then it might need clearing (in this case, only if this is the only active buffer)
            if buf_pos == 0 {
                self.clear_buffer();
            }

            next_symbol
        } else {
            // This reader is in the middle of the buffer, so just return a buffered value
            let next_symbol = self.buffer[buf_pos].clone();

            self.positions[reader_id] = buf_pos + 1;

            // If this reader was at the end of the buffer, then it might need clearing
            if buf_pos == 0 {
                self.clear_buffer();
            }

            next_symbol
        }
    }
}

///
/// Trait that provides 'splittability' for symbol streams
///
pub trait SplittableSymbolReader<Symbol: Clone> : SymbolReader<Symbol> {
    ///
    /// Returns two symbol readers that will independently read the stream of symbols from this reader
    ///
    fn split<'a>(&'a mut self) -> (SplitSymbolReader<'a, Symbol>, SplitSymbolReader<'a, Symbol>);
}

impl<Symbol: Clone, Reader: SymbolReader<Symbol>> SplittableSymbolReader<Symbol> for Reader {
    fn split<'a>(&'a mut self) -> (SplitSymbolReader<'a, Symbol>, SplitSymbolReader<'a, Symbol>) {
        // Generate the buffer that gets shared between the readers
        let mut buffer = SplitSymbolReaderBuffer::new(self);

        buffer.positions = vec![0,0];

        let buffer_ref = Rc::new(RefCell::new(buffer));

        // The readers share the buffer but have different IDs so they can read the same stream twice
        (SplitSymbolReader { buffer: buffer_ref.clone(), reader_id: 0 }, SplitSymbolReader { buffer: buffer_ref.clone(), reader_id: 1 })
    }
}

impl<'a, Symbol: Clone+'a> SymbolReader<Symbol> for SplitSymbolReader<'a, Symbol> {
    fn next_symbol(&mut self) -> Option<Symbol> {
        (*self.buffer).borrow_mut().read(self.reader_id)
    }
}

impl<'a, Symbol: Clone+'a> Drop for SplitSymbolReader<'a, Symbol> {
    fn drop(&mut self) {
        let mut buffer_ref = (*self.buffer).borrow_mut();

        buffer_ref.positions[self.reader_id] = usize::max_value();
        buffer_ref.clear_buffer();
    }
}

#[cfg(test)]
mod test {
    use super::super::*;

    #[test]
    fn can_split_stream_and_read_both_interleaved() {
        let source = vec![1,2,3];
        let mut stream = source.read_symbols();

        let (mut first, mut second) = stream.split();

        assert!(first.next_symbol() == Some(1));
        assert!(second.next_symbol() == Some(1));
        assert!(first.next_symbol() == Some(2));
        assert!(second.next_symbol() == Some(2));
        assert!(first.next_symbol() == Some(3));
        assert!(second.next_symbol() == Some(3));
        assert!(first.next_symbol() == None);
        assert!(second.next_symbol() == None);
    }

    #[test]
    fn can_split_stream_and_read_both_sequentially() {
        let source = vec![1,2,3];
        let mut stream = source.read_symbols();

        let (mut first, mut second) = stream.split();

        assert!(first.to_vec() == vec![1,2,3]);
        assert!(second.to_vec() == vec![1,2,3]);
    }
}
