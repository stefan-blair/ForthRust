use super::value;
use super::generic_numbers;
use super::generic_numbers::{ConvertOperations, AsValue};


// contains stack in the vec, and offset contains the current base pointer (not used in data stack)
pub struct Stack(Vec<value::Value>);

impl Stack {
    pub fn new() -> Self {
        Stack(Vec::new())
    }

    pub(super) fn push_value(&mut self, value: value::Value) {
        self.0.push(value);
    }

    pub(super) fn pop_value(&mut self) -> Option<value::Value> {
        self.0.pop()
    }

    pub fn push<T: value::ValueVariant>(&mut self, value: T) {
        value.push_to_stack(self);
    }

    pub fn pop<T: value::ValueVariant>(&mut self) -> Option<T> {
        T::pop_from_stack(self)
    }

    pub fn peek<T: value::ValueVariant>(&mut self) -> Option<T> {
        self.pop().map(|value| {
            self.push(value);
            value
        })
    }

    pub fn to_vec(&self) -> Vec<value::Value> {
        self.0.clone()
    }

    pub fn debug_only_get_vec<'a>(&'a self) -> &'a Vec<value::Value> {
        &self.0
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