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
use std::ops::Range;

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

    ///
    /// Replaces a range in this stream with a tag
    ///
    pub fn tag(&mut self, tag: Tag, range: Range<usize>) {
        // Create a tag to replace the range
        let replaced_symbols    = self.data[range.clone()].to_vec();
        let tag_symbol          = TagSymbol::Tagged(tag, TaggedStream { data: replaced_symbols });

        // Draining seems to be for reading a range but does double duty for deleting a range?
        // I don't think rust has a way to replace a range in a vector, or at least not one that's easy to find in the docs.
        self.data.drain(range.clone());

        // Draining then inserting is inefficient compared to flat out replacing items :-/ This may be possible but the vec docs aren't very easy to read
        self.data.insert(range.start, tag_symbol);
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

#[cfg(test)]
mod test {
    use super::super::*;

    #[test]
    fn can_tag_range() {
        #[derive(Clone, PartialEq, Eq, Copy)]
        enum Tags {
            Hello,
            World
        }

        let mut tagged: TaggedStream<char, Tags> = TaggedStream::from_reader(&mut "HelloWorld".read_symbols());

        tagged.tag(Tags::Hello, 0..5);
        tagged.tag(Tags::World, 1..6);

        assert!(tagged.len() == 2);

        if let TagSymbol::Tagged(ref tag, ref stream) = tagged[0] {
            assert!(*tag == Tags::Hello);
            assert!(stream.len() == 5);
            assert!(stream[0] == TagSymbol::Untagged('H'));
            assert!(stream[1] == TagSymbol::Untagged('e'));
            assert!(stream[2] == TagSymbol::Untagged('l'));
            assert!(stream[3] == TagSymbol::Untagged('l'));
            assert!(stream[4] == TagSymbol::Untagged('o'));
        } else {
            assert!(false);
        }

        if let TagSymbol::Tagged(ref tag, ref stream) = tagged[1] {
            assert!(*tag == Tags::World);
            assert!(stream.len() == 5);
            assert!(stream[0] == TagSymbol::Untagged('W'));
            assert!(stream[1] == TagSymbol::Untagged('o'));
            assert!(stream[2] == TagSymbol::Untagged('r'));
            assert!(stream[3] == TagSymbol::Untagged('l'));
            assert!(stream[4] == TagSymbol::Untagged('d'));
        } else {
            assert!(false);
        }
    }
}
