use std::ops;
use std::convert;
use std::cmp;
use std::fmt;
use std::mem;

use crate::evaluate::Error;
use super::{memory, stack, value, units::Cells};

/**
 * Interface for stack and memory to implement for each generic number type.
 */
pub trait StackOperations<T> { 
    fn push_number_by_type(&mut self, value: T);
    fn pop_number_by_type(&mut self) -> Result<T, Error>;
}

pub trait MemoryOperations {
    fn read_number(memory_segment: &dyn memory::MemorySegment, address: memory::Address) -> Result<Self, Error> where Self: Sized;
    fn write_number(self, memory_segment: &mut dyn memory::MemorySegment, address: memory::Address) -> Result<(), Error>;
}

/**
 * Define a "generic number" type, which is what gets manipulated.  This comes with some accompanying methods
 * used for interacting with the stack and memory.
 */
pub trait GenericNumber: fmt::Debug + Clone + Copy + Eq + PartialEq + cmp::Ord +
    convert::From<bool> + std::marker::Sized + value::ValueVariant + 
    ops::Add<Output=Self> + ops::Sub<Output=Self> + ops::Mul<Output=Self> + ops::Div<Output=Self> + ops::Rem<Output=Self> + 
    ops::Shl<Output=Self> + ops::Shr<Output=Self> + ops::BitAnd<Output=Self> + ops::BitOr<Output=Self> {
    type AsNumberType;

    fn as_one() -> Self;
    fn as_zero() -> Self;

    /*
     * There are some numeric operations, negate and absolute value, that can by default only be performed on 
     * signed numbers.  Create a trait such that signed numbers can perform these operations, while unsigned
     * numbers will simply return unchanged.
     */
    fn neg(self) -> Self;
    fn abs(self) -> Self;
}

/**
 * Quick trait to outline some additional requirements on a signed number.  Unsigned numbers are implemented off
 * of signed numbers, essentially by wrapping them and then casting the results.  This trait helps that implementation.
 */
pub trait SignedGenericNumber: GenericNumber {
    type NumberType;
    type UnsignedNumberType;
    fn to_unsigned(self) -> Self::UnsignedNumberType;
}

macro_rules! generic_number {
    ($name:ident, $type:ty, $cells: expr) => {
        // create a type alias so that $name can be used instead, and $type can change without breaking code
        pub type $name = $type;

        /**
         * Implement the GenericNumber trait for each type the exact same way.
         */
        impl GenericNumber for $type {
            type AsNumberType = $type;

            fn as_one() -> Self { 1 as $type }
            fn as_zero() -> Self { 0 as $type }

            fn neg(self) -> Self { -self }
            fn abs(self) -> Self { (self as $type).abs() }
        }

        impl value::ValueVariant for $type {
            fn push_to_stack(self, stack: &mut stack::Stack) {
                stack.push_number_by_type(self);
            }
        
            fn pop_from_stack(stack: &mut stack::Stack) -> Result<Self, Error> {
                stack.pop_number_by_type()
            }

            fn write_to_memory(self, memory: &mut dyn memory::MemorySegment, address: memory::Address) -> Result<(), Error> {
                self.write_number(memory, address)
            }
            
            fn read_from_memory(memory: &dyn memory::MemorySegment, address: memory::Address) -> Result<Self, Error> {
                Self::read_number(memory, address)
            }

            fn push_to_memory(self, memory: &mut memory::Memory) {
                memory.push_number_by_type(self)
            }

            fn size() -> usize {
                $cells
            }
        }
    };

    ($name:ident, $type:ty, $unsigned_name:ident, $unsigned_type:ty, $cells: expr) => {
        generic_number!($name, $type, $cells);

        /**
         * Implement the GenericNumber trait for the unsigned version of the number, this time simply wrapping the 
         * signed implementation and casting.
         */
        pub type $unsigned_name = $unsigned_type;
        impl GenericNumber for $unsigned_name {
            type AsNumberType = $unsigned_type;

            fn as_one() -> Self { 1 as $unsigned_type }
            fn as_zero() -> Self { 0 as $unsigned_type }

            fn neg(self) -> Self { self }
            fn abs(self) -> Self { self }
        }

        impl value::ValueVariant for $unsigned_type {
            fn push_to_stack(self, stack: &mut stack::Stack) {
                stack.push_number_by_type(self as $name);
            }

            fn pop_from_stack(stack: &mut stack::Stack) -> Result<Self, Error> {
                stack.pop_number_by_type().map(|number: $name| number as $unsigned_type)
            }

            fn write_to_memory(self, memory: &mut dyn memory::MemorySegment, address: memory::Address) -> Result<(), Error> {
                (self as $name).write_number(memory, address)
            }

            fn read_from_memory(memory: &dyn memory::MemorySegment, address: memory::Address) -> Result<Self, Error> {
                let number = $name::read_number(memory, address)?;
                Ok(number as $unsigned_type)
            }

            fn push_to_memory(self, memory: &mut memory::Memory) {
                memory.push_number_by_type(self as $name)
            }

            fn size() -> usize {
                $cells
            }
        }

        impl SignedGenericNumber for $name {
            type NumberType = $type;
            type UnsignedNumberType = $unsigned_type;

            fn to_unsigned(self) -> Self::UnsignedNumberType {
                self as $unsigned_type
            }
        }
    };
}

generic_number!(Byte, i8, UnsignedByte, u8, 1);
generic_number!(Number, i64, UnsignedNumber, u64, 1);
generic_number!(DoubleNumber, i128, UnsignedDoubleNumber, u128, 2);

/**
 * Syntactic sugar for Value::Number(_).  The other value types all have similar functions.
 */
pub trait AsValue {
    fn value(self) -> value::Value;
}

impl AsValue for Number {
    fn value(self) -> value::Value {
        value::Value::Number(self)
    }
}

impl MemoryOperations for Byte {
    fn read_number(memory_segment: &dyn memory::MemorySegment, address: memory::Address) -> Result<Self, Error> {
        memory_segment.read_value(address).map(|value| value.to_number().to_chunks()[address.get_cell_byte()])
    }

    fn write_number(self, memory_segment: &mut dyn memory::MemorySegment, address: memory::Address) -> Result<(), Error> {
        let value = memory_segment.read_value(address)?;
        let mut bytes = value.to_number().to_chunks();
        bytes[address.get_cell_byte()] = self;
        memory_segment.write_value(address, Number::from_chunks(&bytes).value())
    }
}

impl MemoryOperations for Number {
    fn read_number(memory_segment: &dyn memory::MemorySegment, address: memory::Address) -> Result<Self, Error> {
        memory_segment.read_value(address).map(|value| value.to_number())
    }

    fn write_number(self, memory_segment: &mut dyn memory::MemorySegment, address: memory::Address) -> Result<(), Error> {
        memory_segment.write_value(address, value::Value::Number(self))
    }
}

impl MemoryOperations for DoubleNumber {
    fn read_number(memory_segment: &dyn memory::MemorySegment, address: memory::Address) -> Result<Self, Error> {
        let a = memory_segment.read_value(address)?;
        let b = memory_segment.read_value(address.plus_cell(Cells::one()))?;
        Ok(DoubleNumber::from_chunks(&[a.to_number(), b.to_number()]))
    }

    fn write_number(self, memory_segment: &mut dyn memory::MemorySegment, mut address: memory::Address) -> Result<(), Error> {
        let chunks = self.to_chunks();
        let mut address_probe = address;
        
        // first do a check to make sure this is a valid write position
        for _ in chunks.iter() {
            memory_segment.check_address(address_probe)?;
            address_probe.increment_cell();
        }

        // then atomically do the write
        for chunk in chunks {
            memory_segment.write_value(address, chunk.value())?;
            address.increment_cell();
        }

        Ok(())
    }
}

/**
 * This trait defines how two numbers of different types / sizes would be converted
 * between each other in chunks.  For example, convertings an array of bytes into a i64.
 * The helper macro automatically generates such conversions for GenericNumber types
 */
pub trait ConvertOperations<T> {
    fn from_chunks(chunks: &[T]) -> Self;
    fn from_chunk(chunk: T) -> Self;
    fn to_chunks(self) -> Vec<T>;
}

macro_rules! convert_operations {
    ($small:ident, $large:ident) => {
        impl ConvertOperations<$small> for $large {
            fn from_chunks(chunks: &[$small]) -> $large {
                chunks.iter().cloned()
                    // convert to unsigned type to avoid sign extension
                    .map(|chunk| chunk as <$small as SignedGenericNumber>::UnsignedNumberType)
                    // map each chunk to an index
                    .enumerate()
                    // convert each chunk to a large sized number, and shift it to the proper spot
                    .map(|(i, chunk)| (chunk as <$large as SignedGenericNumber>::NumberType) << (i * mem::size_of::<<$small as GenericNumber>::AsNumberType>() * 8))
                    // combine all of the now large chunks by bitwise or-ing them together
                    .fold(0, |acc, i| acc | i)
            }

            fn from_chunk(chunk: $small) -> $large {
                Self::from_chunks(&[chunk])
            }
        
            fn to_chunks(self) -> Vec<$small> {
                // get the sizes of the two types of numbers
                let sizeof_small = mem::size_of::<<$small as GenericNumber>::AsNumberType>();
                let sizeof_large = mem::size_of::<<$large as GenericNumber>::AsNumberType>();
                // get a bitmask for the smaller chunk
                let small_mask: <$small as GenericNumber>::AsNumberType = 0 - 1;
                (0..(sizeof_large / sizeof_small)).map(|i| small_mask & (self >> (i * sizeof_small * 8)) as <$small as GenericNumber>::AsNumberType).collect()
            }
        }
    }
}

convert_operations!(Byte, Number);
convert_operations!(Number, DoubleNumber);
