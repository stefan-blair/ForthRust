use std::mem;

use super::value;
use super::generic_numbers;
use super::generic_numbers::{ConvertOperations, AsValue};

pub type ValueSize = u64;
pub type Offset = usize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Address(Offset);

impl Address {
    pub fn get_cell(self) -> Offset {
        self.0 / mem::size_of::<ValueSize>()
    }

    pub fn get_cell_byte(self) -> Offset {
        self.0 % mem::size_of::<ValueSize>()
    }

    pub fn increment_cell(&mut self) {
        self.0 += mem::size_of::<ValueSize>();
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
        let cell_size = mem::size_of::<ValueSize>();
        Self(((self.0 + (cell_size - 1)) / 8) * 8)
    }

    pub fn plus_cell(self, i: Offset) -> Self {
        Address(self.0 + (i * mem::size_of::<ValueSize>()))
    }

    pub fn minus_cell(self, i: Offset) -> Self {
        Address(self.0 - (i * mem::size_of::<ValueSize>()))
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

pub struct Memory(Vec<value::Value>);

impl Memory {
    pub fn new() -> Self {
        Memory(vec![0.value()])
    }

    pub fn address_from(&self, number: generic_numbers::Number) -> Option<Address> {
        let possible_address = number as Offset;
        if possible_address / mem::size_of::<ValueSize>() < self.0.len() {
            Some(Address(possible_address))
        } else {
            None
        }
    }

    pub fn address_from_cell(&self, number: generic_numbers::Number) -> Option<Address> {
        let possible_address = number as Offset;
        if possible_address < self.0.len() {
            Some(Address(possible_address * mem::size_of::<ValueSize>()))
        } else {
            None
        }
    }

    pub fn top(&self) -> Address {
        Address((self.0.len() - 1) * mem::size_of::<ValueSize>())
    }

    pub fn expand(&mut self, amount: Offset) {
        self.0.resize(self.0.len() + amount, 0.value())
    }

    pub fn push_none(&mut self) {
        self.0.push(0.value());
    }

    pub fn push(&mut self, value: value::Value) {
        self.0.pop();
        self.0.push(value);
        self.0.push(0.value());
    }

    pub fn write_value(&mut self, address: Address, value: value::Value) {
        self.0[address.get_cell()] = value
    }

    pub fn read_value(&self, address: Address) -> value::Value {
        self.0[address.get_cell()]
    }

    pub fn write<T: value::ValueVariant>(&mut self, address: Address, number: T) {
        number.write_to_memory(self, address)
    }

    pub fn read<T: value::ValueVariant>(&self, address: Address) -> T {
        T::read_from_memory(self, address)
    }

    pub fn debug_only_get_vec<'a>(&'a self) -> &'a Vec<value::Value> {
        &self.0
    }
}

impl generic_numbers::MemoryOperations<generic_numbers::Byte> for Memory {
    fn read_number_by_type(&self, address: Address) -> generic_numbers::Byte {
        self.0[address.get_cell()].to_number().to_chunks()[address.get_cell_byte()]
    }

    fn write_number_by_type(&mut self, address: Address, byte: generic_numbers::Byte) {
        let mut bytes = self.0[address.get_cell()].to_number().to_chunks();
        bytes[address.get_cell_byte()] = byte;
        self.0[address.get_cell()] = generic_numbers::Number::from_chunks(&bytes).value();
    }
}

impl generic_numbers::MemoryOperations<generic_numbers::Number> for Memory {
    fn read_number_by_type(&self, address: Address) -> generic_numbers::Number {
        self.0[address.get_cell()].to_number()
    }

    fn write_number_by_type(&mut self, address: Address, number: generic_numbers::Number) {
        self.0[address.get_cell()] = number.value();
    }
}

impl generic_numbers::MemoryOperations<generic_numbers::DoubleNumber> for Memory {
    fn read_number_by_type(&self, address: Address) -> generic_numbers::DoubleNumber {
        let chunks = [self.0[address.get_cell()].to_number(), self.0[address.plus_cell(1).get_cell()].to_number()];
        generic_numbers::DoubleNumber::from_chunks(&chunks)
    }

    fn write_number_by_type(&mut self, mut address: Address, number: generic_numbers::DoubleNumber) {
        for chunk in number.to_chunks() {
            self.0[address.get_cell()] = chunk.value();
            address.increment_cell();
        }
    }
}
