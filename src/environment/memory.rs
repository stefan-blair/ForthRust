use std::mem;

use crate::evaluate::{Error};
use super::value::{self, ValueVariant};
use super::generic_numbers;
use super::generic_numbers::{ConvertOperations, AsValue};
use crate::environment::{stack, memory};

pub type ValueSize = u64;
pub type Offset = usize;
pub const CELL_SIZE: Offset = mem::size_of::<ValueSize>();

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Address(Offset);

impl Address {
    fn from_offset(offset: Offset) -> Self {
        Self(offset)
    }

    pub fn debug_only_from_offset(offset: Offset) -> Self {
        Self(offset)
    }

    pub fn debug_only_from_cell(offset: Offset) -> Self {
        Self(offset * CELL_SIZE)
    }

    pub fn get_cell(self) -> Offset {
        self.0 / CELL_SIZE
    }

    pub fn get_cell_byte(self) -> Offset {
        self.0 % CELL_SIZE
    }

    pub fn increment_cell(&mut self) {
        self.0 += CELL_SIZE;
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn less_than(self, other: Address) -> bool {
        self.0 < other.0
    }

    /**
     * If the address is not aligned to the size of a cell, get the next cell.
     */
    pub fn nearest_cell(&self) -> Self {
        let cell_size = CELL_SIZE;
        Self(((self.0 + (cell_size - 1)) / 8) * 8)
    }

    pub fn plus_cell(self, i: Offset) -> Self {
        Address(self.0 + (i * CELL_SIZE))
    }

    pub fn minus_cell(self, i: Offset) -> Self {
        Address(self.0 - (i * CELL_SIZE))
    }

    pub fn plus(self, i: Offset) -> Self {
        Address(self.0 + i)
    }

    pub fn to_number(self) -> generic_numbers::Number {
        self.0 as generic_numbers::Number
    }

    pub fn to_offset(self) -> Offset {
        self.0
    }
}

impl ValueVariant for Address {
    fn push_to_stack(self, stack: &mut stack::Stack) {
        stack.push(self.to_number());
    }

    fn pop_from_stack(stack: &mut stack::Stack) -> Result<Self, Error> {
        stack.pop().map(|number: generic_numbers::Number| Self::from_offset(number as Offset))
    }

    fn write_to_memory(self, memory: &mut memory::Memory, address: memory::Address) -> Result<(), Error> {
        memory.write(address, self.to_number())
    }

    fn read_from_memory(memory: &memory::Memory, address: memory::Address) -> Result<Self, Error> {
        memory.read(address).map(|number: generic_numbers::Number| Self::from_offset(number as Offset))
    }

    fn push_to_memory(self, memory: &mut memory::Memory) {
        memory.push(self.to_number())
    }

    fn null() -> Self {
        Self::from_offset(0)
    }
}

pub struct Memory(Vec<value::Value>);

impl Memory {
    pub fn new() -> Self {
        Memory(vec![0.value()])
    }

    pub fn top(&self) -> Address {
        Address((self.0.len() - 1) * CELL_SIZE)
    }

    pub fn expand(&mut self, amount: Offset) {
        self.0.resize(self.0.len() + amount, 0.value())
    }

    pub fn push_none(&mut self) {
        self.0.push(0.value());
    }

    pub fn check_address(&self, address: Address) -> Result<(), Error> {
        if address.get_cell() < self.0.len() {
            Ok(())
        } else {
            Err(Error::InvalidAddress)
        }
    }

    pub fn write_value(&mut self, address: Address, value: value::Value) -> Result<(), Error> {
        self.check_address(address).map(|_| {
            self.0[address.get_cell()] = value
        })
    }

    pub fn read_value(&self, address: Address) -> Result<value::Value, Error> {
        self.check_address(address).map(|_|{
            self.0[address.get_cell()]
        })
    }

    pub fn push_value(&mut self, value: value::Value) {
        self.0.pop();
        self.0.push(value);
        self.0.push(0.value());
    }

    pub fn write<T: value::ValueVariant>(&mut self, address: Address, number: T) -> Result<(), Error> {
        number.write_to_memory(self, address)
    }

    pub fn read<T: value::ValueVariant>(&self, address: Address) -> Result<T, Error> {
        T::read_from_memory(self, address)
    }

    
    pub fn push<T: value::ValueVariant>(&mut self, value: T) {
        value.push_to_memory(self);
    }

    pub fn debug_only_get_vec<'a>(&'a self) -> &'a Vec<value::Value> {
        &self.0
    }
}

impl generic_numbers::MemoryOperations<generic_numbers::Byte> for Memory {
    fn read_number_by_type(&self, address: Address) -> Result<generic_numbers::Byte, Error> {
        self.read_value(address).map(|value| value.to_number().to_chunks()[address.get_cell_byte()])
    }

    fn write_number_by_type(&mut self, address: Address, byte: generic_numbers::Byte) -> Result<(), Error> {
        let value = self.read_value(address)?;
        let mut bytes = value.to_number().to_chunks();
        bytes[address.get_cell_byte()] = byte;
        self.write_value(address, generic_numbers::Number::from_chunks(&bytes).value())
    }

    fn push_number_by_type(&mut self, byte: generic_numbers::Byte) {
        self.push_value(generic_numbers::Number::from_chunks(&[byte]).value())
    }
}

impl generic_numbers::MemoryOperations<generic_numbers::Number> for Memory {
    fn read_number_by_type(&self, address: Address) -> Result<generic_numbers::Number, Error> {
        self.read_value(address).map(|value| value.to_number())
    }

    fn write_number_by_type(&mut self, address: Address, number: generic_numbers::Number) -> Result<(), Error> {
        self.write_value(address, number.value())
    }

    fn push_number_by_type(&mut self, number: generic_numbers::Number) {
        self.push_value(number.value())
    }
}

impl generic_numbers::MemoryOperations<generic_numbers::DoubleNumber> for Memory {
    fn read_number_by_type(&self, address: Address) -> Result<generic_numbers::DoubleNumber, Error> {
        let a = self.read_value(address)?;
        let b = self.read_value(address.plus_cell(1))?;
        Ok(generic_numbers::DoubleNumber::from_chunks(&[a.to_number(), b.to_number()]))
    }

    fn write_number_by_type(&mut self, mut address: Address, number: generic_numbers::DoubleNumber) -> Result<(), Error> {
        let chunks = number.to_chunks();
        let mut address_probe = address;
        
        // first do a check to make sure this is a valid write position
        for _ in chunks.iter() {
            self.check_address(address_probe)?;
            address_probe.increment_cell();
        }

        // then atomically do the write
        for chunk in chunks {
            self.0[address.get_cell()] = chunk.value();
            address.increment_cell();
        }

        Ok(())
    }

    fn push_number_by_type(&mut self, double_number: generic_numbers::DoubleNumber) {
        double_number.to_chunks().iter().for_each(|c| self.push_value(c.value()))
    }
}
