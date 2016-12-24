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
//! let simple_tagged = TaggedStream::from_reader(&mut "Hello, world".read_symbols());
//! let with_tag      = simple_tagged.with_tags(vec![(0..5, SomeTag::Identifier)].iter().cloned());
//! ```
//!

use std::slice::Iter;
use std::ops::Index;
use std::ops::Range;

use super::symbol_reader::*;
use super::tokenizer::*;
use super::symbol_range_dfa::*;

///
/// Represents a symbol in a tagged stream.
///
/// A symbol can either be a sequence of untagged base symbols, or it can be a tag (which itself can contain any number of tagged symbol)
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TagSymbol<Base: Clone+Ord, Tag: Clone+Ord> {
    /// An untagged element in this stream
    Untagged(Base),

    /// A tagged region of the stream
    Tagged(Tag, TaggedStream<Base, Tag>)
}

///
/// Represents a stream of tagged symbols
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TaggedStream<Base: Clone+Ord, Tag: Clone+Ord> {
    /// The data in this stream
    data: Vec<TagSymbol<Base, Tag>>
}

impl<Base: Ord+Clone, Tag: Ord+Clone> TaggedStream<Base, Tag> {
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

    ///
    /// Creates a tag symbol by tagging a particular range within this stream
    ///
    pub fn tag_range(&self, range: Range<usize>, tag: Tag) -> TagSymbol<Base, Tag> {
        let tag_data = self.data[range].to_vec();

        TagSymbol::Tagged(tag, TaggedStream { data: tag_data })
    }

    ///
    /// Applies tags to the specified ranges in this stream
    ///
    /// The tag ranges must be in ascending order and must not overlap
    ///
    pub fn with_tags<I>(&self, tags: I) -> TaggedStream<Base, Tag> 
        where I : Iterator<Item=(Range<usize>, Tag)> {
        // The data that will make up the new stream
        let mut new_stream = vec![];

        // The last processed range (we haven't processed any range yet, so this starts at 0..0)
        let mut last_range = 0..0;

        // Build up the result from the tag list and the last range
        for tag in tags {
            // Fetch the information for this tag and check ordering
            let (next_range, next_tag)  = tag;
            let end                     = last_range.end;

            if end > next_range.start {
                panic!("Tags for with_tags must be sorted in order");
            }

            // Sometimes need to leave some data from this stream untagged
            if end < next_range.start {
                new_stream.extend(self.data[end..next_range.start].iter().cloned());
            }

            // Push the tag
            new_stream.push(self.tag_range(next_range.clone(), next_tag));

            // Update state
            last_range = next_range;
        }

        // Append anything left in the stream
        if last_range.end < self.data.len() {
            new_stream.extend(self.data[last_range.end..self.data.len()].iter().cloned());
        }

        // Final result
        TaggedStream { data: new_stream }
    }

    ///
    /// Runs the current values of this tagged stream through a tokenizer and tags anything it matches
    ///
    pub fn tokenize<DfaSymbol: Ord>(&self, dfa: &SymbolRangeDfa<DfaSymbol, Tag>) -> TaggedStream<Base, Tag> {
        unimplemented!();
    }
}

impl<Base: Clone+Ord, Tag: Clone+Ord> Index<usize> for TaggedStream<Base, Tag> {
    type Output = TagSymbol<Base, Tag>;

    fn index(&self, index: usize) -> &TagSymbol<Base, Tag> {
        &self.data[index]
    }
}

impl<'a, Base: Clone+Ord, Tag: Clone+Ord> SymbolSource<'a, TagSymbol<Base, Tag>> for &'a TaggedStream<Base, Tag> {
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
        #[derive(Clone, PartialEq, Eq, Copy, PartialOrd, Ord)]
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

    #[test]
    fn can_tag_everything_with_tags() {
        #[derive(Clone, PartialEq, Eq, Copy, PartialOrd, Ord)]
        enum Tags {
            Hello,
            World
        }

        let original: TaggedStream<char, Tags> = TaggedStream::from_reader(&mut "HelloWorld".read_symbols());
        let tagged = original.with_tags(vec![(0..5, Tags::Hello), (5..10, Tags::World)].iter().cloned());

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

    #[test]
    fn with_tags_preserves_middle() {
        #[derive(Clone, PartialEq, Eq, Copy, PartialOrd, Ord)]
        enum Tags {
            Hello,
            World
        }

        let original: TaggedStream<char, Tags> = TaggedStream::from_reader(&mut "HelloWorld".read_symbols());
        let tagged = original.with_tags(vec![(0..4, Tags::Hello), (6..10, Tags::World)].iter().cloned());

        assert!(tagged.len() == 4);
        assert!(tagged[1] == TagSymbol::Untagged('o'));
        assert!(tagged[2] == TagSymbol::Untagged('W'));

        if let TagSymbol::Tagged(ref tag, ref stream) = tagged[0] {
            assert!(*tag == Tags::Hello);
            assert!(stream.len() == 4);
            assert!(stream[0] == TagSymbol::Untagged('H'));
            assert!(stream[1] == TagSymbol::Untagged('e'));
            assert!(stream[2] == TagSymbol::Untagged('l'));
            assert!(stream[3] == TagSymbol::Untagged('l'));
        } else {
            assert!(false);
        }

        if let TagSymbol::Tagged(ref tag, ref stream) = tagged[3] {
            assert!(*tag == Tags::World);
            assert!(stream.len() == 4);
            assert!(stream[0] == TagSymbol::Untagged('o'));
            assert!(stream[1] == TagSymbol::Untagged('r'));
            assert!(stream[2] == TagSymbol::Untagged('l'));
            assert!(stream[3] == TagSymbol::Untagged('d'));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn with_tags_preserves_end() {
        #[derive(Clone, PartialEq, Eq, Copy, PartialOrd, Ord)]
        enum Tags {
            Hello
        }

        let original: TaggedStream<char, Tags> = TaggedStream::from_reader(&mut "HelloWorld".read_symbols());
        let tagged = original.with_tags(vec![(0..5, Tags::Hello)].iter().cloned());

        assert!(tagged.len() == 6);

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

        assert!(tagged[1] == TagSymbol::Untagged('W'));
        assert!(tagged[2] == TagSymbol::Untagged('o'));
        assert!(tagged[3] == TagSymbol::Untagged('r'));
        assert!(tagged[4] == TagSymbol::Untagged('l'));
        assert!(tagged[5] == TagSymbol::Untagged('d'));
    }

    #[test]
    fn with_tags_preserves_start() {
        #[derive(Clone, PartialEq, Eq, Copy, PartialOrd, Ord)]
        enum Tags {
            World
        }

        let original: TaggedStream<char, Tags> = TaggedStream::from_reader(&mut "HelloWorld".read_symbols());
        let tagged = original.with_tags(vec![(5..10, Tags::World)].iter().cloned());

        assert!(tagged.len() == 6);

        if let TagSymbol::Tagged(ref tag, ref stream) = tagged[5] {
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

        assert!(tagged[0] == TagSymbol::Untagged('H'));
        assert!(tagged[1] == TagSymbol::Untagged('e'));
        assert!(tagged[2] == TagSymbol::Untagged('l'));
        assert!(tagged[3] == TagSymbol::Untagged('l'));
        assert!(tagged[4] == TagSymbol::Untagged('o'));
    }
}
