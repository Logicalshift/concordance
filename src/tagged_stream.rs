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
//! # Tagged stream
//!
//! A tagged stream recursively applies tags to regions of an input stream. A stream consisting of a single tag forms a tree,
//! so this stream type is useful for representing the parsed represention of an input.
//!
//! The simplest form of tagged stream is one that's just a set of untagged symbols. This can be created using the `TaggedStream::from_reader`
//! method.
//!
//! ```
//! # use concordance::*;
//! #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
//! enum SomeTag {
//!     Identifier
//! };
//!
//! let simple_tagged : TaggedStream<char, SomeTag> = TaggedStream::from_reader(&mut "Hello, world".read_symbols());
//! ```
//!

use std::slice::Iter;
use std::ops::Index;

use super::symbol_reader::*;

///
/// Represents a symbol in a tagged stream.
///
/// A symbol can either be a sequence of untagged base symbols, or it can be a tag (which itself can contain any number of tagged symbol)
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TagSymbol<Base: Clone, Tag: Clone> {
    Untagged(Base),
    Tagged(Tag, TaggedStream<Base, Tag>)
}

///
/// Represents a stream of tagged symbols
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TaggedStream<Base: Clone, Tag: Clone> {
    /// The data in this stream
    data: Vec<TagSymbol<Base, Tag>>
}

impl<Base: Clone, Tag: Clone> TaggedStream<Base, Tag> {
    ///
    /// Creates a basic tagged stream from a source of the base symbol
    ///
    pub fn from_reader(reader: &mut SymbolReader<Base>) -> TaggedStream<Base, Tag> {
        // Read from the reader into an array
        let mut symbols = vec![];

        while let Some(next_symbol) = reader.next_symbol() {
            symbols.push(TagSymbol::Untagged(next_symbol));
        }

        // Generate a simple tagged stream from the result
        TaggedStream { data: symbols }
    }

    ///
    /// The number of symbols in this stream
    ///
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<Base: Clone, Tag: Clone> Index<usize> for TaggedStream<Base, Tag> {
    type Output = TagSymbol<Base, Tag>;

    fn index(&self, index: usize) -> &TagSymbol<Base, Tag> {
        &self.data[index]
    }
}

impl<'a, Base: Clone, Tag: Clone> SymbolSource<'a, TagSymbol<Base, Tag>> for &'a TaggedStream<Base, Tag> {
    type SymbolReader = Iter<'a, TagSymbol<Base, Tag>>;

    /// Returns a new object that can read the symbols from this one
    fn read_symbols(self) -> Self::SymbolReader {
        self.data.read_symbols()
    }
}
