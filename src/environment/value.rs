use super::stack;
use super::memory;
use super::generic_numbers;
use crate::evaluate;


pub trait ValueVariant: std::marker::Sized + Copy + Clone {
    fn push_to_stack(self, stack: &mut stack::Stack);
    fn pop_from_stack(stack: &mut stack::Stack) -> Option<Self>;
    fn write_to_memory(self, memory: &mut memory::Memory, address: memory::Address);
    fn read_from_memory(memory: &mut memory::Memory, address: memory::Address) -> Self;
}

#[derive(Copy, Clone)]
pub enum Value {
    Number(generic_numbers::Number),
    ExecutionToken(evaluate::definition::ExecutionToken),
}

impl Value {
    pub fn to_number(self) -> generic_numbers::Number {
        match self {
            Self::Number(i) => i,
            Self::ExecutionToken(execution_token) => execution_token.to_offset() as generic_numbers::Number
        }
    }
}

impl ValueVariant for Value {
    fn push_to_stack(self, stack: &mut stack::Stack) {
        stack.push_value(self);
    }

    fn pop_from_stack(stack: &mut stack::Stack) -> Option<Self> {
        stack.pop_value()
    }

    fn write_to_memory(self, memory: &mut memory::Memory, address: memory::Address) {
        memory.write_value(address, self)
    }

    fn read_from_memory(memory: &mut memory::Memory, address: memory::Address) -> Self {
        memory.read_value(address)
    }    
}

#[derive(Copy, Clone)]
pub struct DoubleValue(Value, Value);

impl ValueVariant for DoubleValue {
    fn push_to_stack(self, stack: &mut stack::Stack) {
        stack.push(self.1);
        stack.push(self.0);
    }

    fn pop_from_stack(stack: &mut stack::Stack) -> Option<Self> {
        match (stack.pop(), stack.pop()) {
            (Some(a), Some(b)) => Some(DoubleValue(a, b)),
            _ => None
        }
    }

    fn write_to_memory(self, memory: &mut memory::Memory, address: memory::Address) {
        memory.write(address, self.0);
        memory.write(address.plus_cell(1), self.1);
    }

    fn read_from_memory(memory: &mut memory::Memory, address: memory::Address) -> Self {
        DoubleValue(memory.read(address), memory.read(address.plus_cell(1)))
    }
}