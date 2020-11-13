use crate::evaluate::{ForthResult, Error};
use super::value;
use super::generic_numbers::{self, ConvertOperations, AsValue};
use super::memory::{Address, MemorySegment};
use super::units::{Bytes, Cells};


// contains stack in the vec, and offset contains the current base pointer (not used in data stack)
pub struct Stack {
    base: Address,
    stack: Vec<value::Value>,
    frame_offset: usize,
}

impl Stack {
    pub fn new(base: usize) -> Self {
        Self { 
            base: Address::from_raw(Bytes::bytes(base)), 
            stack: Vec::new(),
            frame_offset: 0,
        }
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

    pub fn push_frame(&mut self) {
        self.push(self.frame_offset as generic_numbers::UnsignedNumber);
        self.frame_offset = self.stack.len();
    }

    pub fn pop_frame(&mut self) -> ForthResult {
        self.stack.truncate(self.frame_offset);
        if self.frame_offset > 0 {
            self.frame_offset = self.pop()?;    
        }
        Ok(())
    }

    pub fn read_from_frame<T: value::ValueVariant>(&self, offset: usize) -> Result<T, Error> {
        T::read_from_memory(self, self.get_base().plus_cell(Cells::cells(self.frame_offset + offset)))
    }

    pub fn write_to_frame<T: value::ValueVariant>(&mut self, offset: usize, value: T) -> ForthResult {
        let address = self.get_base().plus_cell(Cells::cells(self.frame_offset + offset));
        value.write_to_memory(self, address)
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
        self.cell_offset(address).map(|cells| self.stack[cells.get_cells()] = value)
    }

    fn read_value(&self, address: Address) -> Result<value::Value, Error> {
        self.cell_offset(address).map(|cells| self.stack[cells.get_cells()])
    }

    fn write_values(&mut self, address: Address, values: &[value::Value]) -> ForthResult {
        // get the start and end indexes
        let start = self.cell_offset(address)?.get_cells();
        let end = self.cell_offset(address.plus_cell(Cells::cells(values.len() - 1)))?.get_cells();

        // copy the given values into the slice
        let slice = &mut self.stack[start..end + 1];
        slice.copy_from_slice(values);

        Ok(())    
    }

    fn read_values(&self, address: Address, len: Cells) -> Result<Vec<value::Value>, Error> {
        // get the start and end indexes
        let start = self.cell_offset(address)?.get_cells();
        let end = self.cell_offset(address.plus_cell(len - Cells::one()))?.get_cells();

        // allocate the results vector
        let mut results = vec![0.value(); len.get_cells()];
        results.copy_from_slice(&self.stack[start..end + 1]);

        Ok(results)
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