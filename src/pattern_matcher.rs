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
//! The Matcher trait is implemented by objects that can match patterns against the left-hand side of a stream. It's a fairly
//! baseline implementation: it's up to the caller to implement things like rewinding in order to perform tokenisation. That is,
//! Matchers are greedy and may (indeed, are likely to) consume more characters than the longest match while trying to find
//! a longer one.
//!

/*
 * Trait not implemented due to Rust type system limitations
 *
 * We want to write type State<'a> and have that in MatchAction, otherwise states need to live too long to be useful.
 *

///
/// Matcher that can read an input stream of type `Symbol` and find the longest matching pattern, which it will identify using
/// `OutputSymbol`
///
pub trait PatternMatcher<InputSymbol, OutputSymbol> {
    type State;

    ///
    /// Creates a state that begins matching this pattern
    ///
    fn start<'a>(&'a self) -> MatchAction<'a, OutputSymbol, Self::State>;
}
*/

///
/// Action to be taken after a matcher receives a symbol
///
pub enum MatchAction<'a, OutputSymbol: 'static, State: Sized> {
    // State is also always: MatchingState<InputSymbol, OutputSymbol> (important to know that as its how More is used)
    //
    // However, rust complains that InputSymbol is unused if we declare it in MatchAction and that it is undeclared if we don't
    // If this wasn't an enum, could make it an associated type? But this is an enum so we can't do that.
    // Ie, MatchAction<InputSymbol, OutputSymbol, State: MatchingState<InputSymbol, OutputSymbol>> = Nope (InputSymbol is unused apparently even though it's required which seems to me to mean it's used)
    // and also MatchAction<OutputSymbol, State: MatchingState<DontCare, OutputSymbol> = Nope (DontCare is not declared)

    /// The pattern does not match
    Reject,

    /// The pattern matched a certain number of symbols (which may be fewer than were passed to the matcher)
    Accept(usize, &'a OutputSymbol),

    /// The matcher needs more symbols to decide if the pattern matches and has entered a new state
    More(State)
}

impl<'a, OutputSymbol: PartialEq+'static, State: Sized> MatchAction<'a, OutputSymbol, State> {
    ///
    /// Returns true if this is an accepting state with the specified symbol
    ///
    pub fn is_accepted(&self, symbol: &OutputSymbol) -> bool {
        match self {
            &Accept(_, sym) => sym == symbol,
            _ => false
        }
    }
}

///
/// Represents a state during a pattern matching operation
///
pub trait MatchingState<'a, InputSymbol, OutputSymbol> : Sized {
    ///
    /// Matches the next symbol
    ///
    fn next(self, symbol: InputSymbol) -> MatchAction<'a, OutputSymbol, Self>;

    ///
    /// There are no more symbols available (this can only return `Reject` or `Accept`)
    ///
    fn finish(self) -> MatchAction<'a, OutputSymbol, Self>;
}

pub use MatchAction::*;
