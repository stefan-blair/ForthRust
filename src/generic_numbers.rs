use std::ops;
use std::convert;
use std::cmp;
use std::fmt;
use std::mem;

use super::memory;

/**
 * Interface for stack and memory to implement for each generic number type.
 */
pub trait StackOperations<T> { 
    fn push_number_by_type(&mut self, value: T);
    fn pop_number_by_type(&mut self) -> Option<T>;
}

pub trait MemoryOperations<T> {
    fn read_number_by_type(&self, address: memory::Address) -> T;
    fn write_number_by_type(&mut self, address: memory::Address, value: T);
}

/**
 * There are some numeric operations, negate and absolute value, that can by default only be performed on 
 * signed numbers.  Create a trait such that signed numbers can perform these operations, while unsigned
 * numbers will simply return unchanged.
 */
pub trait SignedOperations {
    fn neg(self) -> Self;
    fn abs(self) -> Self;
}

macro_rules! signed_signed_operations {
    ($type:ty) => {
        impl SignedOperations for $type {
            fn neg(self) -> Self {
                -self
            }

            fn abs(self) -> Self {
                self.abs()
            }
        }
    };
}

macro_rules! unsigned_signed_operations {
    ($type:ty) => {
        impl SignedOperations for $type {
            fn neg(self) -> Self {
                self
            }

            fn abs(self) -> Self {
                self
            }
        }
    };
}

/**
 * Define a "generic number" type, which is what gets manipulated
 * 
 * maybe we can combine GenericNumberType and GenericNumber into one thing?
 * i mean, could be just adding unnecessary indirection
 * 
 */
pub trait GenericNumberType: fmt::Debug + Clone + Copy + Eq + PartialEq + cmp::Ord +
    convert::From<bool> + SignedOperations +
    ops::Add<Output=Self> + ops::Sub<Output=Self> + ops::Mul<Output=Self> + ops::Div<Output=Self> + ops::Rem<Output=Self> + 
    ops::Shl<Output=Self> + ops::Shr<Output=Self> + ops::BitAnd<Output=Self> + ops::BitOr<Output=Self> {
    fn as_one() -> Self;
    fn as_zero() -> Self;
}

macro_rules! generic_number_type {
    ($type:ty) => {
        impl GenericNumberType for $type {
            fn as_one() -> Self { 1 as $type }
            fn as_zero() -> Self { 0 as $type }
        }
    };
}
   
pub trait GenericNumber: std::marker::Sized {
    type NumberType: GenericNumberType;
    fn from_raw_number(number: Self::NumberType) -> Self;
    fn raw_number(self) -> Self::NumberType;
    fn push_to_stack(self, stack: &mut memory::Stack);
    fn pop_from_stack(stack: &mut memory::Stack) -> Option<Self>;
    fn write_to_memory(self, memory: &mut memory::Memory, address: memory::Address);
    fn read_from_memory(memory: &mut memory::Memory, address: memory::Address) -> Self;
}

pub trait SignedGenericNumberType: GenericNumberType {
    
}

pub trait SignedGenericNumber: GenericNumber {
    type NumberType: GenericNumberType;
    type UnsignedNumberType: GenericNumberType;
    fn to_unsigned(self) -> Self::UnsignedNumberType;
}

macro_rules! generic_number {
    ($name:ident, $type:ty, $unsigned_name:ident, $unsigned_type:ty) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub struct $name(pub $type);
        signed_signed_operations!($type);
        generic_number_type!($type);

        impl GenericNumber for $name {
            type NumberType = $type;

            fn from_raw_number(number: Self::NumberType) -> Self { Self(number) }
            fn raw_number(self) -> Self::NumberType { self.0 }
        
            fn push_to_stack(self, stack: &mut memory::Stack) {
                stack.push_number_by_type(self);
            }
        
            fn pop_from_stack(stack: &mut memory::Stack) -> Option<Self> {
                stack.pop_number_by_type()
            }

            fn write_to_memory(self, memory: &mut memory::Memory, address: memory::Address) {
                memory.write_number_by_type(address, self);
            }

            fn read_from_memory(memory: &mut memory::Memory, address: memory::Address) -> Self {
                memory.read_number_by_type(address)
            }
        }

        pub struct $unsigned_name(pub $unsigned_type);
        unsigned_signed_operations!($unsigned_type);
        generic_number_type!($unsigned_type);

        impl GenericNumber for $unsigned_name {
            type NumberType = $unsigned_type;

            fn from_raw_number(number: Self::NumberType) -> Self { Self(number) }
            fn raw_number(self) -> Self::NumberType { self.0 }

            fn push_to_stack(self, stack: &mut memory::Stack) {
                stack.push_number_by_type($name(self.0 as $type));
            }

            fn pop_from_stack(stack: &mut memory::Stack) -> Option<Self> {
                stack.pop_number_by_type().map(|number: $name| Self(number.0 as $unsigned_type))
            }

            fn write_to_memory(self, memory: &mut memory::Memory, address: memory::Address) {
                memory.write_number_by_type(address, $name(self.0 as $type))
            }

            fn read_from_memory(memory: &mut memory::Memory, address: memory::Address) -> Self {
                let number: $name = memory.read_number_by_type(address);
                Self(number.0 as $unsigned_type)
            }
        }

        impl SignedGenericNumber for $name {
            type NumberType = $type;
            type UnsignedNumberType = $unsigned_type;

            fn to_unsigned(self) -> Self::UnsignedNumberType {
                self.0 as $unsigned_type
            }
        }
    };
}

generic_number!(Byte, i8, UnsignedByte, u8);
generic_number!(Number, i64, UnsignedNumber, u64);
generic_number!(DoubleNumber, i128, UnsignedDoubleNumber, u128);

impl Number {
    pub fn value(self) -> memory::Value {
        memory::Value::Number(self.0)
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
                $large(chunks.iter().cloned()
                    // convert to unsigned type to avoid sign extension
                    .map(|chunk| chunk.0 as <$small as SignedGenericNumber>::UnsignedNumberType)
                    // map each chunk to an index
                    .enumerate()
                    // convert each chunk to a large sized number, and shift it to the proper spot
                    .map(|(i, chunk)| (chunk as <$large as SignedGenericNumber>::NumberType) << (i * mem::size_of::<<$small as GenericNumber>::NumberType>() * 8))
                    // combine all of the now large chunks by bitwise or-ing them together
                    .fold(0, |acc, i| acc | i))
            }

            fn from_chunk(chunk: $small) -> $large {
                Self::from_chunks(&[chunk])
            }
        
            fn to_chunks(self) -> Vec<$small> {
                // get the sizes of the two types of numbers
                let sizeof_small = mem::size_of::<<$small as GenericNumber>::NumberType>();
                let sizeof_large = mem::size_of::<<$large as GenericNumber>::NumberType>();
                // get a bitmask for the smaller chunk
                let small_mask: <$small as GenericNumber>::NumberType = 0 - 1;
                (0..(sizeof_large / sizeof_small)).map(|i| small_mask & (self.0 >> (i * sizeof_small * 8)) as <$small as GenericNumber>::NumberType).map(|x| $small(x)).collect()
            }
        }
    }
}

convert_operations!(Byte, Number);
convert_operations!(Number, DoubleNumber);
