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
//! # Countable
//!
//! This is similar to the unstable `Step` trait. It's used for values that are countable; that is, which have a clear following
//! and previous value. Unlike `Step` we have an implementation for `char`, which is useful for where we want to match strings.
//!

// TODO: could make next/prev return Option<Self> which would let us deal with max/min values. However, we use this internally
// where we can expect this not to matter.

use std::char;

///
/// Trait implemented by types that can be counted
///
pub trait Countable {
    fn next(&self) -> Self;
    fn prev(&self) -> Self;
}

impl Countable for usize { 
    fn next(&self) -> Self { *self+1 }
    fn prev(&self) -> Self { *self-1 }
}

impl Countable for u8 { 
    fn next(&self) -> Self { *self+1 }
    fn prev(&self) -> Self { *self-1 }
}

impl Countable for u16 { 
    fn next(&self) -> Self { *self+1 }
    fn prev(&self) -> Self { *self-1 }
}

impl Countable for u32 { 
    fn next(&self) -> Self { *self+1 }
    fn prev(&self) -> Self { *self-1 }
}

impl Countable for isize { 
    fn next(&self) -> Self { *self+1 }
    fn prev(&self) -> Self { *self-1 }
}

impl Countable for i8 { 
    fn next(&self) -> Self { *self+1 }
    fn prev(&self) -> Self { *self-1 }
}

impl Countable for i16 { 
    fn next(&self) -> Self { *self+1 }
    fn prev(&self) -> Self { *self-1 }
}

impl Countable for i32 { 
    fn next(&self) -> Self { *self+1 }
    fn prev(&self) -> Self { *self-1 }
}

impl Countable for u64 { 
    fn next(&self) -> Self { *self+1 }
    fn prev(&self) -> Self { *self-1 }
}

impl Countable for i64 { 
    fn next(&self) -> Self { *self+1 }
    fn prev(&self) -> Self { *self-1 }
}

impl Countable for char { 
    fn next(&self) -> Self { char::from_u32((*self as u32)+1).unwrap_or('\u{0000}') }
    fn prev(&self) -> Self { char::from_u32((*self as u32)-1).unwrap_or('\u{ffff}') }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_get_next_prev_i8() {
        let val: i8 = 1;

        assert!(val.next() == 2);
        assert!(val.prev() == 0);
    }

    #[test]
    fn can_get_next_prev_u8() {
        let val: u8 = 1;

        assert!(val.next() == 2);
        assert!(val.prev() == 0);
    }

    #[test]
    fn can_get_next_prev_i16() {
        let val: i16 = 1;

        assert!(val.next() == 2);
        assert!(val.prev() == 0);
    }

    #[test]
    fn can_get_next_prev_u16() {
        let val: u16 = 1;

        assert!(val.next() == 2);
        assert!(val.prev() == 0);
    }

    #[test]
    fn can_get_next_prev_i32() {
        let val: i32 = 1;

        assert!(val.next() == 2);
        assert!(val.prev() == 0);
    }

    #[test]
    fn can_get_next_prev_u32() {
        let val: u32 = 1;

        assert!(val.next() == 2);
        assert!(val.prev() == 0);
    }

    #[test]
    fn can_get_next_prev_i64() {
        let val: i64 = 1;

        assert!(val.next() == 2);
        assert!(val.prev() == 0);
    }

    #[test]
    fn can_get_next_prev_u64() {
        let val: u64 = 1;

        assert!(val.next() == 2);
        assert!(val.prev() == 0);
    }

    #[test]
    fn can_get_next_prev_char() {
        let val = 'b';

        assert!(val.next() == 'c');
        assert!(val.prev() == 'a');
    }
}
