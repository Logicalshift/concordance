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

///
/// Represents a symbol in a tagged stream
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TagSymbol<Base: Clone, Tag: Clone> {
    Untagged(Vec<Base>),
    Tagged(Tag, Vec<TagSymbol<Base, Tag>>)
}
