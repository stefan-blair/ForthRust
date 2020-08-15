use std::mem;

use super::operations;
use super::generic_numbers;
use super::generic_numbers::{ConvertOperations, AsValue};

pub type ValueSize = u64;
pub type Offset = usize;
pub type NumberType = i64;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Address(Offset);

impl Address {
    pub fn to_offset(self) -> Offset {
        self.0
    }

    pub fn get_cell(self) -> Offset {
        self.0 / mem::size_of::<ValueSize>()
    }

    pub fn get_cell_byte(self) -> Offset {
        self.0 % mem::size_of::<ValueSize>()
    }

    pub fn with_offset(self, offset: Offset) -> Self {
        Address(offset)
    }

    pub fn increment_cell(&mut self) {
        self.0 += mem::size_of::<ValueSize>();
    }

    pub fn plus_cell(self, i: Offset) -> Self {
        Address(self.0 + (i * mem::size_of::<ValueSize>()))
    }

    pub fn value(self) -> Value {
        Value::Address(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NameTag(pub Offset);

impl NameTag {
    pub fn to_offset(self) -> Offset {
        self.0
    }
}

#[derive(Clone, Copy)]
pub enum ExecutionToken {
    Operation(operations::Operation),
    DefinedOperation(Offset),
    Number(NumberType),
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
    Number(NumberType),
    Address(Address),
    ExecutionToken(ExecutionToken),
}

impl Value {
    pub fn to_raw_number(self) -> NumberType {
        match self {
            Self::Number(i) => i,
            Self::Address(address) => address.to_offset() as NumberType,
            Self::ExecutionToken(execution_token) => execution_token.to_offset() as NumberType
        }
    }

    pub fn to_number(self) -> generic_numbers::Number {
        self.to_raw_number()
    }
}

// contains stack in the vec, and offset contains the current base pointer (not used in data stack)
pub struct Stack(Vec<Value>, Offset);

impl Stack {
    pub fn new() -> Self {
        Stack(Vec::new(), 0)
    }

    pub fn push(&mut self, value: Value) {
        self.0.push(value);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.0.pop()
    }

    pub fn push_number<T: generic_numbers::GenericNumber>(&mut self, number: T) {
        number.push_to_stack(self)
    }

    pub fn pop_number<T: generic_numbers::GenericNumber>(&mut self) -> Option<T> {
        T::pop_from_stack(self)
    }

    pub fn peek(&self) -> Option<Value> {
        self.0.last().map(|x| *x)
    }

    pub fn to_vec(&self) -> Vec<Value> {
        self.0.clone()
    }
}

impl generic_numbers::StackOperations<generic_numbers::Byte> for Stack {
    fn push_number_by_type(&mut self, byte: generic_numbers::Byte) {
        self.0.push(generic_numbers::Number::from_chunks(&[byte]).to_value())
    }

    fn pop_number_by_type(&mut self) -> Option<generic_numbers::Byte> {
        self.0.pop().map(|x| x.to_number().to_chunks()[0])
    }
}

impl generic_numbers::StackOperations<generic_numbers::Number> for Stack {
    fn push_number_by_type(&mut self, number: generic_numbers::Number) {
        self.0.push(number.to_value())
    }

    fn pop_number_by_type(&mut self) -> Option<generic_numbers::Number> {
        self.0.pop().map(|x| x.to_number())
    }
}

impl generic_numbers::StackOperations<generic_numbers::DoubleNumber> for Stack {
    fn push_number_by_type(&mut self, double_number: generic_numbers::DoubleNumber) {
        double_number.to_chunks().iter().for_each(|c| self.0.push(c.to_value()))
    }

    fn pop_number_by_type(&mut self) -> Option<generic_numbers::DoubleNumber> {
        // theres a bug in from_chunks... its for some reason not just forming the number, its extending it with 1s
        self.0.pop()
            .and_then(|x| self.0.pop().map(|y| (x, y)))
            .map(|(upper, lower)| generic_numbers::DoubleNumber::from_chunks(&[lower.to_number(), upper.to_number()]))
    }
}

pub struct Memory(Vec<Value>);

impl Memory {
    pub fn new() -> Self {
        Memory(Vec::new())
    }

    pub fn top(&self) -> Address {
        Address(self.0.len() * mem::size_of::<ValueSize>())
    }

    pub fn expand(&mut self, amount: Offset) {
        self.0.resize(self.0.len() + amount, 0.to_value())
    }

    pub fn push_none(&mut self) {
        self.0.push(0.to_value());
    }

    pub fn push(&mut self, value: Value) {
        self.0.push(value);
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
        self.0[address.get_cell()] = generic_numbers::Number::from_chunks(&bytes).to_value();
    }
}

impl generic_numbers::MemoryOperations<generic_numbers::Number> for Memory {
    fn read_number_by_type(&self, address: Address) -> generic_numbers::Number {
        self.0[address.get_cell()].to_number()
    }

    fn write_number_by_type(&mut self, address: Address, number: generic_numbers::Number) {
        self.0[address.get_cell()] = number.to_value();
    }
}

impl generic_numbers::MemoryOperations<generic_numbers::DoubleNumber> for Memory {
    fn read_number_by_type(&self, address: Address) -> generic_numbers::DoubleNumber {
        let chunks = [self.0[address.get_cell()].to_number(), self.0[address.plus_cell(1).get_cell()].to_number()];
        generic_numbers::DoubleNumber::from_chunks(&chunks)
    }

    fn write_number_by_type(&mut self, mut address: Address, number: generic_numbers::DoubleNumber) {
        for chunk in number.to_chunks() {
            self.0[address.get_cell()] = chunk.to_value();
            address.increment_cell();
        }
    }
}
