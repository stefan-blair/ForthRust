use super::stack;
use super::memory;
use super::generic_numbers;
use crate::evaluate::{self, Error};


pub trait ValueVariant: std::marker::Sized + Copy + Clone {
    // connector functions used for stack and memory operations
    fn push_to_stack(self, stack: &mut stack::Stack);
    fn pop_from_stack(stack: &mut stack::Stack) -> Result<Self, Error>;
    fn write_to_memory(self, memory: &mut memory::Memory, address: memory::Address) -> Result<(), Error>;
    fn read_from_memory(memory: &memory::Memory, address: memory::Address) -> Result<Self, Error>;
    fn push_to_memory(self, memory: &mut memory::Memory);
    // the size, in number of cells (aka, the size of one Value)
    fn size() -> memory::Offset;
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

    fn pop_from_stack(stack: &mut stack::Stack) -> Result<Self, Error> {
        stack.pop_value()
    }

    fn write_to_memory(self, memory: &mut memory::Memory, address: memory::Address) -> Result<(), Error> {
        memory.write_value(address, self)
    }

    fn read_from_memory(memory: &memory::Memory, address: memory::Address) -> Result<Self, Error> {
        memory.read_value(address)
    }

    fn push_to_memory(self, memory: &mut memory::Memory) {
        memory.push_value(self)
    }

    fn size() -> memory::Offset {
        1
    }
}

#[derive(Copy, Clone)]
pub struct DoubleValue(Value, Value);

impl ValueVariant for DoubleValue {
    fn push_to_stack(self, stack: &mut stack::Stack) {
        stack.push(self.1);
        stack.push(self.0);
    }

    fn pop_from_stack(stack: &mut stack::Stack) -> Result<Self, Error> {
        Ok(DoubleValue(stack.pop()?, stack.pop()?))
    }

    fn write_to_memory(self, memory: &mut memory::Memory, address: memory::Address) -> Result<(), Error> {
        memory.check_address(address).and(memory.check_address(address.plus_cell(1))).and_then(|_| {
            memory.write(address, self.0)?;
            memory.write(address.plus_cell(1), self.1)?;
            Ok(())
        })
    }

    fn read_from_memory(memory: &memory::Memory, address: memory::Address) -> Result<Self, Error> {
        let a = memory.read(address)?;
        let b = memory.read(address.plus_cell(1))?;
        Ok(DoubleValue(a, b))
    }

    fn push_to_memory(self, memory: &mut memory::Memory) {
        memory.push(self.0);
        memory.push(self.1);
    }

    fn size() -> memory::Offset {
        2
    }
}