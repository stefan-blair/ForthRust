use std::mem;

use super::operations;
use super::generic_numbers;
use super::generic_numbers::{ConvertOperations, AsValue};

pub type ValueSize = u64;
pub type Offset = usize;

#[derive(Clone, Copy)]
pub enum ExecutionToken {
    Operation(operations::Operation),
    DefinedOperation(Offset),
    Number(generic_numbers::Number),
}

impl ExecutionToken {
    pub fn to_offset(self) -> Offset {
        match self {
            Self::Operation(_) => 0,
            Self::DefinedOperation(i) => i,
            Self::Number(i) => i as Offset
        }
    }

    pub fn value(self) -> Value {
        Value::ExecutionToken(self)
    }
}

#[derive(Copy, Clone)]
pub enum Value {
    Number(generic_numbers::Number),
    ExecutionToken(ExecutionToken),
}

impl Value {
    pub fn to_raw_number(self) -> generic_numbers::Number {
        match self {
            Self::Number(i) => i,
            Self::ExecutionToken(execution_token) => execution_token.to_offset() as generic_numbers::Number
        }
    }

    pub fn to_number(self) -> generic_numbers::Number {
        self.to_raw_number()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

    pub fn plus_cell(self, i: Offset) -> Self {
        Address(self.0 + (i * mem::size_of::<ValueSize>()))
    }

    pub fn to_number(self) -> generic_numbers::Number {
        self.0 as generic_numbers::Number
    }
}

pub struct Memory(Vec<Value>);

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

    pub fn top(&self) -> Address {
        Address((self.0.len() - 1) * mem::size_of::<ValueSize>())
    }

    pub fn expand(&mut self, amount: Offset) {
        self.0.resize(self.0.len() + amount, 0.value())
    }

    pub fn push_none(&mut self) {
        self.0.push(0.value());
    }

    pub fn push(&mut self, value: Value) {
        self.0.pop();
        self.0.push(value);
        self.0.push(0.value());
    }

    pub fn read(&self, address: Address) -> Value {
        self.0[address.get_cell()]
    }

    pub fn write(&mut self, address: Address, value: Value) {
        self.0[address.get_cell()] = value
    }

    pub fn write_number<T: generic_numbers::GenericNumber>(&mut self, address: Address, number: T) {
        number.write_to_memory(self, address)
    }

    pub fn read_number<T: generic_numbers::GenericNumber>(&mut self, address: Address) -> T {
        T::read_from_memory(self, address)
    }
}

impl generic_numbers::MemoryOperations<generic_numbers::Byte> for Memory {
    fn read_number_by_type(&self, address: Address) -> generic_numbers::Byte {
        self.0[address.get_cell()].to_number().to_chunks()[address.get_cell_byte()]
    }

    fn write_number_by_type(&mut self, address: Address, number: generic_numbers::Byte) {
        let mut bytes = self.0[address.get_cell()].to_number().to_chunks();
        bytes[address.get_cell_byte()] = number;
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
