use crate::memory;
use crate::generic_numbers;
use crate::generic_numbers::{ConvertOperations, AsValue};


// contains stack in the vec, and offset contains the current base pointer (not used in data stack)
pub struct Stack(Vec<memory::Value>, memory::Offset);

impl Stack {
    pub fn new() -> Self {
        Stack(Vec::new(), 0)
    }

    pub fn push(&mut self, value: memory::Value) {
        self.0.push(value);
    }

    pub fn pop(&mut self) -> Option<memory::Value> {
        self.0.pop()
    }

    pub fn push_number<T: generic_numbers::GenericNumber>(&mut self, number: T) {
        number.push_to_stack(self)
    }

    pub fn pop_number<T: generic_numbers::GenericNumber>(&mut self) -> Option<T> {
        T::pop_from_stack(self)
    }

    pub fn peek(&self) -> Option<memory::Value> {
        self.0.last().map(|x| *x)
    }

    pub fn to_vec(&self) -> Vec<memory::Value> {
        self.0.clone()
    }
}

impl generic_numbers::StackOperations<generic_numbers::Byte> for Stack {
    fn push_number_by_type(&mut self, byte: generic_numbers::Byte) {
        self.0.push(generic_numbers::Number::from_chunks(&[byte]).value())
    }

    fn pop_number_by_type(&mut self) -> Option<generic_numbers::Byte> {
        self.0.pop().map(|x| x.to_number().to_chunks()[0])
    }
}

impl generic_numbers::StackOperations<generic_numbers::Number> for Stack {
    fn push_number_by_type(&mut self, number: generic_numbers::Number) {
        self.0.push(number.value())
    }

    fn pop_number_by_type(&mut self) -> Option<generic_numbers::Number> {
        self.0.pop().map(|x| x.to_number())
    }
}

impl generic_numbers::StackOperations<generic_numbers::DoubleNumber> for Stack {
    fn push_number_by_type(&mut self, double_number: generic_numbers::DoubleNumber) {
        double_number.to_chunks().iter().for_each(|c| self.0.push(c.value()))
    }

    fn pop_number_by_type(&mut self) -> Option<generic_numbers::DoubleNumber> {
        // theres a bug in from_chunks... its for some reason not just forming the number, its extending it with 1s
        self.0.pop()
            .and_then(|x| self.0.pop().map(|y| (x, y)))
            .map(|(upper, lower)| generic_numbers::DoubleNumber::from_chunks(&[lower.to_number(), upper.to_number()]))
    }
}