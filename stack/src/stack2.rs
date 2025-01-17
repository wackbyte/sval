/*!
A fixed-size, stateful stack for streams.
*/

/*
/!\ CAREFUL /!\

This module contains unsafe code with some tricky
invariants based on the state of the current slot.

We use a combination of property-based testing
and a reasonable test suite to try ensure safety
is maintained, but any changes here should be
reviewed carefully.
*/

use crate::std::fmt;

struct Expecting {
    expecting: &'static str,
}

impl fmt::Display for Expecting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "expecting {}", self.expecting)
    }
}

/**
The expected position in the stream.
*/
#[derive(Clone)]
pub struct Pos(RawStack, u8);

/**
The depth of a position.

All positions within a map or sequence are guaranteed
to have the same depth or greater.
*/
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Depth(usize);

#[cfg(all(feature = "alloc", any(test, feature = "test")))]
impl Depth {
    pub fn root() -> Self {
        Depth(0)
    }
}

impl Pos {
    /**
    Whether the current position is a map key.
    */
    pub fn is_key(&self) -> bool {
        (self.0 as u8) & Slot::MASK_SLOT == Slot::IS_MAP_KEY
    }

    /**
    Whether the current position is a map value.
    */
    pub fn is_value(&self) -> bool {
        (self.0 as u8) & Slot::MASK_SLOT == Slot::IS_MAP_VALUE
    }

    /**
    Whether the current position is a sequence element.
    */
    pub fn is_elem(&self) -> bool {
        (self.0 as u8) & Slot::MASK_SLOT == Slot::IS_SEQ_ELEM
    }

    /**
    Whether the current position is a map value or sequence element.
    */
    pub fn is_value_elem(&self) -> bool {
        (self.0 as u8) & Slot::MASK_VALUE_ELEM != 0
    }

    /**
    Whether the current position is an empty map.
    */
    pub fn is_empty_map(&self) -> bool {
        unimplemented!()
    }

    /**
    Whether the current position is an empty sequence.
    */
    pub fn is_empty_seq(&self) -> bool {
        unimplemented!()
    }

    /**
    The depth of this position.
    */
    pub fn depth(&self) -> Depth {
        Depth(self.1 as usize)
    }
}

enum Slot {}

impl Slot {
    const IS_EMPTY: u8 = 0b0000_0000;
    const IS_MAP_KEY: u8 = 0b0000_0010;
    const IS_MAP_VALUE: u8 = 0b0000_0110;
    const IS_SEQ_ELEM: u8 = 0b0000_1000;

    #[allow(dead_code)]
    const RESERVED: u8 = 0b0001_0000;

    const MASK_VALUE_ELEM: u8 = 0b0000_1100;

    const NEEDS_ITEM: u8 = 0b0000_0001;
    const NEEDS_MAP_KEY: u8 = 0b0000_0100;
    const NEEDS_MAP_VALUE: u8 = 0b0000_0010;
    const NEEDS_SEQ_ELEM: u8 = 0b0000_1000;

    const MASK_SLOT: u8 = u8::MAX >> (u8::BITS as u8 - Slot::BITS);

    // NOTE: This leaves us with 4 "spare" bits at the end of a 64bit stack
    // This is where we could encode whether or not the map or sequence is empty
    const BITS: u8 = 5;
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

type RawStack = u64;

#[derive(Clone)]
pub struct Stack {
    inner: RawStack,
    depth: u8,
}

impl Stack {
    const MAX_DEPTH: u8 = Self::BITS / Slot::BITS;

    const BITS: u8 = RawStack::BITS as u8;

    const MASK_SLOT_BEGIN: RawStack =
        (RawStack::MAX << Slot::BITS) ^ (Slot::NEEDS_ITEM as RawStack) << Slot::BITS;

    /**
    Create a new stack.
    */
    pub fn new() -> Self {
        Stack {
            inner: Slot::NEEDS_ITEM as RawStack,
            depth: 0,
        }
    }

    /**
    Clear the stack so that it can be re-used.

    Any state it currently contains will be lost.
    */
    pub fn clear(&mut self) {
        *self = Stack::new();
    }

    /**
    Push a primitive.

    A primitive is a simple value that isn't a map or sequence.
    That includes:

    - [`Arguments`](struct.Arguments.html)
    - `u64`, `i64`, `u128`, `i128`
    - `f64`
    - `bool`
    - `char`, `&str`
    - `Option<T>`.
    */
    pub fn primitive(&mut self) -> Result<Pos, crate::Error> {
        const MASK: u8 = Slot::MASK_SLOT & Slot::NEEDS_ITEM;
        const VALID: u8 = Slot::NEEDS_ITEM;
        const EXPECT_NEXT: RawStack = Slot::NEEDS_ITEM as RawStack;

        if self.inner as u8 & MASK == VALID {
            self.inner ^= EXPECT_NEXT;

            Ok(Pos(self.inner, self.depth))
        } else {
            Err(crate::Error::custom(&"a primitive"))
        }
    }

    /**
    Begin a new map.

    The map must be completed by calling `map_end`.
    */
    pub fn map_begin(&mut self) -> Result<Pos, crate::Error> {
        const MASK: u8 = Slot::MASK_SLOT & Slot::NEEDS_ITEM;
        const VALID: u8 = Slot::NEEDS_ITEM;
        const EXPECT: RawStack = (Slot::NEEDS_MAP_KEY | Slot::NEEDS_MAP_VALUE) as RawStack;

        if self.depth == Self::MAX_DEPTH {
            return Err(crate::Error::custom(&"more depth at the start of a map"));
        }

        if self.inner as u8 & MASK == VALID {
            self.inner = (self.inner << Slot::BITS) & Self::MASK_SLOT_BEGIN | EXPECT;
            self.depth += 1;

            Ok(Pos(self.inner, self.depth))
        } else {
            Err(crate::Error::custom(&"the start of a map"))
        }
    }

    /**
    Begin a map key.

    The key will be implicitly completed by the value
    that follows it.
    */
    pub fn map_key(&mut self) -> Result<Pos, crate::Error> {
        const MASK: u8 = Slot::MASK_SLOT;
        const VALID: u8 = Slot::NEEDS_MAP_KEY | Slot::NEEDS_MAP_VALUE;
        const EXPECT: RawStack = (Slot::NEEDS_MAP_KEY | Slot::NEEDS_ITEM) as RawStack;

        if self.inner as u8 & MASK == VALID {
            self.inner ^= EXPECT;

            Ok(Pos(self.inner, self.depth))
        } else {
            Err(crate::Error::custom(&"a map key"))
        }
    }

    /**
    Begin a map value.

    The value will be implicitly completed by the value
    that follows it.
    */
    pub fn map_value(&mut self) -> Result<Pos, crate::Error> {
        const MASK: u8 = Slot::MASK_SLOT;
        const VALID: u8 = Slot::NEEDS_MAP_VALUE;
        const EXPECT: RawStack = (Slot::NEEDS_MAP_KEY | Slot::NEEDS_ITEM) as RawStack;

        if self.inner as u8 & MASK == VALID {
            self.inner ^= EXPECT;

            Ok(Pos(self.inner, self.depth))
        } else {
            Err(crate::Error::custom(&"a map value"))
        }
    }

    /**
    Complete the current map.
    */
    pub fn map_end(&mut self) -> Result<Pos, crate::Error> {
        const MASK: u8 = Slot::MASK_SLOT;
        const VALID: u8 = Slot::NEEDS_MAP_KEY | Slot::NEEDS_MAP_VALUE;

        if self.inner as u8 & MASK == VALID {
            self.inner >>= Slot::BITS;
            self.depth -= 1;

            Ok(Pos(self.inner, self.depth))
        } else {
            Err(crate::Error::custom(&"the end of a map"))
        }
    }

    /**
    Begin a new sequence.

    the sequence must be completed by calling `seq_end`.
    */
    pub fn seq_begin(&mut self) -> Result<Pos, crate::Error> {
        const MASK: u8 = Slot::MASK_SLOT & Slot::NEEDS_ITEM;
        const VALID: u8 = Slot::NEEDS_ITEM;
        const EXPECT: RawStack = (Slot::NEEDS_SEQ_ELEM) as RawStack;

        if self.depth == Self::MAX_DEPTH {
            return Err(crate::Error::custom(
                &"more depth at the start of a sequence",
            ));
        }

        if self.inner as u8 & MASK == VALID {
            self.inner = (self.inner << Slot::BITS) & Self::MASK_SLOT_BEGIN | EXPECT;
            self.depth += 1;

            Ok(Pos(self.inner, self.depth))
        } else {
            Err(crate::Error::custom(&"the start of a sequence"))
        }
    }

    /**
    Begin a sequence element.

    The element will be implicitly completed by the value
    that follows it.
    */
    pub fn seq_elem(&mut self) -> Result<Pos, crate::Error> {
        const MASK: u8 = Slot::MASK_SLOT;
        const VALID: u8 = Slot::NEEDS_SEQ_ELEM;
        const EXPECT: RawStack = Slot::NEEDS_ITEM as RawStack;

        if self.inner as u8 & MASK == VALID {
            self.inner |= EXPECT;

            Ok(Pos(self.inner, self.depth))
        } else {
            Err(crate::Error::custom(&"a sequence element"))
        }
    }

    /**
    Complete the current sequence.
    */
    pub fn seq_end(&mut self) -> Result<Pos, crate::Error> {
        const MASK: u8 = Slot::MASK_SLOT;
        const VALID: u8 = Slot::NEEDS_SEQ_ELEM;

        if self.inner as u8 & MASK == VALID {
            self.inner >>= Slot::BITS;
            self.depth -= 1;

            Ok(Pos(self.inner, self.depth))
        } else {
            Err(crate::Error::custom(&"the end of a sequence"))
        }
    }

    /**
    Whether or not the stack has seen a complete and valid stream.
    */
    pub fn can_end(&self) -> bool {
        // In order to end the stream, the stack must be completed
        const MASK: u8 = !Slot::NEEDS_ITEM;
        const VALID: u8 = Slot::IS_EMPTY;

        self.depth == 0 && self.inner as u8 & MASK == VALID
    }

    /**
    Complete the stack.

    This stack may be re-used after being completed
    by calling `begin`.
    */
    pub fn end(&mut self) -> Result<(), crate::Error> {
        if self.can_end() {
            Ok(())
        } else {
            Err(crate::Error::custom(&"the end of the stream"))
        }
    }
}
