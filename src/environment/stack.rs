use crate::evaluate::Error;
use super::value;
use super::generic_numbers::{self, ConvertOperations, AsValue};
use super::memory::{Address, MemorySegment};
use super::units::{Bytes, Cells};


// contains stack in the vec, and offset contains the current base pointer (not used in data stack)
pub struct Stack {
    base: Address,
    stack: Vec<value::Value>
}

impl Stack {
    pub fn new(base: usize) -> Self {
        Self { base: Address::from_raw(Bytes::bytes(base)), stack: Vec::new() }
    }

    pub(super) fn push_value(&mut self, value: value::Value) {
        self.stack.push(value);
    }

    pub(super) fn pop_value(&mut self) -> Result<value::Value, Error> {
        self.stack.pop().ok_or(Error::StackUnderflow)
    }

    pub fn push<T: value::ValueVariant>(&mut self, value: T) {
        value.push_to_stack(self);
    }

    pub fn pop<T: value::ValueVariant>(&mut self) -> Result<T, Error> {
        T::pop_from_stack(self)
    }

    pub fn peek<T: value::ValueVariant>(&mut self) -> Result<T, Error> {
        self.pop().map(|value| {
            self.push(value);
            value
        })
    }

    pub fn len(&self) -> Cells {
        Cells::cells(self.stack.len())
    }

    pub fn to_vec(&self) -> Vec<value::Value> {
        self.stack.clone()
    }

    pub fn debug_only_get_vec<'a>(&'a self) -> &'a Vec<value::Value> {
        &self.stack
    }
}

impl MemorySegment for Stack {
    fn get_base(&self) -> Address {
        self.base
    }

    fn get_end(&self) -> Address {
        self.base.plus_cell(self.len())
    }

    fn write_value(&mut self, address: Address, value: value::Value) -> Result<(), Error> {
        self.check_address(address).map(|_| {
            self.stack[address.offset_from(self.base).to_cells().get_cells()] = value
        })
    }

    fn read_value(&self, address: Address) -> Result<value::Value, Error> {
        self.check_address(address).map(|_|{
            self.stack[address.offset_from(self.base).to_cells().get_cells()]
        })
    }
}

impl generic_numbers::StackOperations<generic_numbers::Byte> for Stack {
    fn push_number_by_type(&mut self, byte: generic_numbers::Byte) {
        self.stack.push(generic_numbers::Number::from_chunks(&[byte]).value())
    }

    fn pop_number_by_type(&mut self) -> Result<generic_numbers::Byte, Error> {
        self.pop_value().map(|x| x.to_number().to_chunks()[0])
    }
}

impl generic_numbers::StackOperations<generic_numbers::Number> for Stack {
    fn push_number_by_type(&mut self, number: generic_numbers::Number) {
        self.stack.push(number.value())
    }

    fn pop_number_by_type(&mut self) -> Result<generic_numbers::Number, Error> {
        self.pop_value().map(|x| x.to_number())
    }
}

impl generic_numbers::StackOperations<generic_numbers::DoubleNumber> for Stack {
    fn push_number_by_type(&mut self, double_number: generic_numbers::DoubleNumber) {
        double_number.to_chunks().iter().for_each(|c| self.stack.push(c.value()))
    }

    fn pop_number_by_type(&mut self) -> Result<generic_numbers::DoubleNumber, Error> {
        self.pop_value()
            .and_then(|x| self.pop_value().map(|y| (x, y)))
            .map(|(upper, lower)| generic_numbers::DoubleNumber::from_chunks(&[lower.to_number(), upper.to_number()]))
    }
}