use std::ops;
use std::cmp;
use std::convert;
use crate::environment::{memory, value::ValueVariant, generic_numbers, stack};
use crate::evaluate::Error;


macro_rules! init_unit {
    ([$type:ident, $constructor:ident, $getter:ident, $to_name:ident, $bytes:expr], $([$other_type:ident, $other_constructor:ident, $other_getter:ident, $other_to_name:ident, $other_bytes:expr]),*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct $type(usize);
        
        impl $type {
            pub const fn $constructor(num: usize) -> Self {
                $type(num)
            }

            pub const fn zero() -> Self {
                Self::$constructor(0)
            }

            pub const fn one() -> Self {
                Self::$constructor(1)
            }

            pub const fn $getter(self) -> usize {
                self.0
            }

            // this should round up
            $(
                pub const fn $other_to_name(self) -> $other_type {
                    $other_type((self.0 * $bytes + $other_bytes - 1) / $other_bytes)
                }
            )*
        }

        // implement basic arithmetic operations for the types
        impl ops::Add for $type {
            type Output = $type;

            fn add(self, rhs: $type) -> Self::Output {
                $type(self.0 + rhs.0)
            }
        }

        impl ops::Sub for $type {
            type Output = $type;

            fn sub(self, rhs: $type) -> Self::Output {
                $type(self.0 - rhs.0)
            }
        }

        impl ops::Mul<usize> for $type {
            type Output = $type;

            fn mul(self, rhs: usize) -> Self::Output {
                $type(self.0 * rhs)
            }
        }

        impl ops::Div<usize> for $type {
            type Output = $type;

            fn div(self, rhs: usize) -> Self::Output {
                $type(self.0 / rhs)
            }
        }

        impl ops::Div<$type> for $type {
            type Output = usize;

            fn div(self, rhs: $type) -> Self::Output {
                self.0 / rhs.0
            }
        }

        impl ops::AddAssign for $type {
            fn add_assign(&mut self, other: Self) {
                *self = Self(self.0 + other.0);
            }
        }

        impl ops::SubAssign for $type {
            fn sub_assign(&mut self, other: Self) {
                *self = Self(self.0 - other.0);
            }
        }

        impl cmp::Ord for $type {
            fn cmp(&self, other: &$type) -> cmp::Ordering {
                self.0.cmp(&other.0)
            }
        }

        impl cmp::PartialOrd for $type {
            fn partial_cmp(&self, other: &$type) -> Option<cmp::Ordering> {
                Some(self.0.cmp(&other.0))
            }
        }

        impl convert::From<usize> for $type {
            fn from(size: usize) -> Self {
                Self::$constructor(size)
            }
        }

        impl convert::From<generic_numbers::Number> for $type {
            fn from(size: generic_numbers::Number) -> Self {
                Self::$constructor(size as usize)
            }
        }

        impl convert::From<generic_numbers::UnsignedNumber> for $type {
            fn from(size: generic_numbers::UnsignedNumber) -> Self {
                Self::$constructor(size as usize)
            }
        }

        impl ToString for $type {
            fn to_string(&self) -> String {
                format!("{} {}", self.0, std::stringify!($type))
            }
        }

        impl ValueVariant for $type {
            fn push_to_stack(self, stack: &mut stack::Stack) {
                (self.0 as generic_numbers::UnsignedNumber).push_to_stack(stack)
            }

            fn pop_from_stack(stack: &mut stack::Stack) -> Result<Self, Error> {
                generic_numbers::UnsignedNumber::pop_from_stack(stack)
                    .map(|number| Self(number as usize))
            }

            fn write_to_memory(self, memory: &mut dyn memory::MemorySegment, address: memory::Address) -> Result<(), Error> {
                (self.0 as generic_numbers::UnsignedNumber).write_to_memory(memory, address)
            }

            fn read_from_memory(memory: &dyn memory::MemorySegment, address: memory::Address) -> Result<Self, Error> {
                generic_numbers::UnsignedNumber::read_from_memory(memory, address)
                    .map(|number| Self(number as usize))
            }

            fn push_to_memory(self, memory: &mut memory::Memory) {
                (self.0 as generic_numbers::UnsignedNumber).push_to_memory(memory)
            }

            fn size() -> usize {
                generic_numbers::UnsignedNumber::size()
            }
        }
    };
}

/**
 * Some weird macro stuff to call init_unit on each element of the passed in array, along with an array of the "other" units, 
 * in order to form the conversion functions.
 */
macro_rules! init_units {
    (
        [$type:ident, $constructor:ident, $getter:ident, $to_name:ident, $bytes:expr], 
        $([$tail_type:ident, $tail_constructor:ident, $tail_getter:ident, $tail_to_name:ident, $tail_bytes:expr]),*
    ) => {
        init_unit!(
            [$type, $constructor, $getter, $to_name, $bytes], 
            $([$tail_type, $tail_constructor, $tail_getter, $tail_to_name, $tail_bytes]),*
        );
        init_units!(
            [$type, $constructor, $getter, $to_name, $bytes]; 
            $([$tail_type, $tail_constructor, $tail_getter, $tail_to_name, $tail_bytes]),*
        );
    };
    (
        $([$head_type:ident, $head_constructor:ident, $head_getter:ident, $head_to_name:ident, $head_bytes:expr]),+; 
        [$type:ident, $constructor:ident, $getter:ident, $to_name:ident, $bytes:expr], 
        $([$tail_type:ident, $tail_constructor:ident, $tail_getter:ident, $tail_to_name:ident, $tail_bytes:expr]),+
    ) => {
        init_unit!(
            [$type, $constructor, $getter, $to_name, $bytes], 
            $([$head_type, $head_constructor, $head_getter, $head_to_name, $head_bytes]),+, 
            $([$tail_type, $tail_constructor, $tail_getter, $tail_to_name, $tail_bytes]),+
        );
        init_units!(
            $([$head_type, $head_constructor, $head_getter, $head_to_name, $head_bytes]),+, 
            [$type, $constructor, $getter, $to_name, $bytes]; 
            $([$tail_type, $tail_constructor, $tail_getter, $tail_to_name, $tail_bytes]),+
        );
    };
    ($([$head_type:ident, $head_constructor:ident, $head_getter:ident, $head_to_name:ident, $head_bytes:expr]),+; [$type:ident, $constructor:ident, $getter:ident, $to_name:ident, $bytes:expr]) => {
        init_unit!(
            [$type, $constructor, $getter, $to_name, $bytes], 
            $([$head_type, $head_constructor, $head_getter, $head_to_name, $head_bytes]),+
        );
    };
}

init_units!(
    [Bytes, bytes, get_bytes, to_bytes, 1],
    [Cells, cells, get_cells, to_cells, memory::CELL_SIZE],
    [Pages, pages, get_pages, to_pages, memory::PAGE_SIZE]
);

impl Bytes {
    pub fn containing_cells(self) -> Cells {
        Cells::cells(self.0 / memory::CELL_SIZE)
    }
}

#[test]
fn basic_test() {
    let some_cells = Cells::cells(5);
    let some_bytes = Bytes::bytes(40);

    assert_eq!(some_cells.to_bytes(), some_bytes);
    assert_eq!(some_cells, some_bytes.to_cells());
}

#[test]
fn round_up_test() {
    assert_eq!(Bytes::bytes(6).to_cells(), Cells::cells(1));
    assert_eq!(Bytes::bytes(16).to_cells(), Cells::cells(2));
    assert_eq!(Bytes::bytes(17).to_cells(), Cells::cells(3));
}

#[test]
fn subtraction_test() {
    let pages_1 = Pages::pages(10);
    let pages_2 = Pages::pages(5);

    assert_eq!(Pages::pages(5), pages_1 - pages_2);
}