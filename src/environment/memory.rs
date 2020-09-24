use std::mem;

use crate::evaluate::{ForthState, Error, ForthResult};
use super::value::{self, ValueVariant};
use super::generic_numbers;
use super::generic_numbers::{ConvertOperations, AsValue};
use crate::environment::{stack, memory};


pub const PAGE_SIZE: usize = 0x1000;
pub const CELL_SIZE: usize = mem::size_of::<u64>();
pub const CELLS_PER_PAGE: usize = PAGE_SIZE / CELL_SIZE;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Address(usize);

impl Address {
    pub fn from_raw(raw: usize) -> Self {
        Self(raw)
    }

    pub fn debug_only_from_offset(offset: usize) -> Self {
        Self(offset)
    }

    pub fn debug_only_from_cell(offset: usize) -> Self {
        Self(offset * CELL_SIZE)
    }

    pub fn get_cell(self) -> usize {
        self.0 / CELL_SIZE
    }

    pub fn get_cell_byte(self) -> usize {
        self.0 % CELL_SIZE
    }

    pub fn less_than(self, other: Address) -> bool {
        self.0 < other.0
    }

    pub fn between(self, lower: Address, upper: Address) -> bool {
        self.0 >= lower.0 && self.0 < upper.0
    }

    pub fn equals(self, other: Address) -> bool {
        self.0 == other.0
    }

    pub fn offset_from(self, base: Address) -> usize {
        self.0 - base.0
    }

    pub fn cell_offset_from(self, base: Address) -> usize {
        self.offset_from(base) / CELL_SIZE
    }

    /**
     * If the address is not aligned to the size of a cell, get the next cell.
     */
    pub fn nearest_cell(&self) -> Self {
        let cell_size = CELL_SIZE;
        Self(((self.0 + (cell_size - 1)) / 8) * 8)
    }

    pub fn increment_cell(&mut self) {
        self.0 += CELL_SIZE;
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn add(&mut self, n: usize) {
        self.0 += n;
    }

    pub fn subtract(&mut self, n: usize) {
        self.0 -= n;
    }

    pub fn plus_cell(self, i: usize) -> Self {
        Address(self.0 + (i * CELL_SIZE))
    }

    pub fn minus_cell(self, i: usize) -> Self {
        Address(self.0 - (i * CELL_SIZE))
    }

    pub fn plus(self, i: usize) -> Self {
        Address(self.0 + i)
    }

    pub fn to_number(self) -> generic_numbers::Number {
        self.0 as generic_numbers::Number
    }

    pub fn as_raw(self) -> usize {
        self.0
    }
}

impl ValueVariant for Address {
    fn push_to_stack(self, stack: &mut stack::Stack) {
        stack.push(self.to_number());
    }

    fn pop_from_stack(stack: &mut stack::Stack) -> Result<Self, Error> {
        stack.pop().map(|number: generic_numbers::Number| Self::from_raw(number as usize))
    }

    fn write_to_memory(self, memory: &mut dyn memory::MemorySegment, address: memory::Address) -> Result<(), Error> {
        memory.write_value(address, value::Value::Number(self.to_number()))
    }

    fn read_from_memory(memory: &dyn memory::MemorySegment, address: memory::Address) -> Result<Self, Error> {
        memory.read_value(address).map(|v| Self::from_raw(v.to_number() as usize))
    }

    fn push_to_memory(self, memory: &mut memory::Memory) {
        memory.push(self.to_number())
    }

    fn size() -> usize {
        1
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        format!("Address({:#x})", self.0)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct MemoryPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool
}

impl MemoryPermissions {
    pub fn with(read: bool, write: bool, execute: bool) -> Self {
        Self { read, write, execute }
    }

    pub fn readonly() -> Self {
        Self::with(true, false, false)
    }

    pub fn readwrite() -> Self {
        Self::with(true, true, false)
    }
    
    pub fn all() -> Self {
        Self::with(true, true, true)
    }

    pub fn with_write(&self) -> Self {
        Self::with(self.read, true, self.execute)
    }

    pub fn with_execute(&self) -> Self {
        Self::with(self.read, self.write, true)
    }

    pub fn allows(&self, other: &Self) -> bool {
        if other.read && !self.read {
            false
        } else if other.write && !self.write {
            false
        } else if other.execute && !self.execute {
            false
        } else {
            true
        }
    }
}

impl ToString for MemoryPermissions {
    fn to_string(&self) -> String {
        let read = if self.read { 'r' } else { '_' };
        let write = if self.write { 'w' } else { '_'};
        let execute = if self.execute { 'x' } else { '_' };

        format!("{}{}{}", read, write, execute)
    }
}

type MutableSegmentGetter = for<'a> fn(&'a mut ForthState) -> &'a mut (dyn MemorySegment + 'a);
type SegmentGetter = for<'a> fn(&'a ForthState) -> &'a (dyn MemorySegment + 'a);

#[derive(Clone, Copy)]
pub enum MappingType {
    Empty,
    Named {
        getter: SegmentGetter, 
        mutable_getter: MutableSegmentGetter, 
        name: &'static str
    },
    Anonymous {
        index: usize,
    }
}

#[derive(Clone, Copy)]
pub struct MemoryMapping {
    pub base: Address,
    pub permissions: MemoryPermissions,
    pub mapping_type: MappingType,
}

impl MemoryMapping {
    fn new(base: Address, permissions: MemoryPermissions, mapping_type: MappingType) -> Self {
        Self { base, permissions, mapping_type }
    }

    pub fn named(base: Address, permissions: MemoryPermissions, getter: SegmentGetter, mutable_getter: MutableSegmentGetter, name: &'static str) -> Self {
        Self::new(base, permissions, MappingType::Named { getter, mutable_getter, name })
    }

    pub fn anonymous(base: Address, permissions: MemoryPermissions, index: usize) -> Self {
        Self::new(base, permissions, MappingType::Anonymous { index })
    }

    pub fn empty(base: Address, permissions: MemoryPermissions) -> Self {
        Self::new(base, permissions, MappingType::Empty)
    }

    pub fn get_offset(&self, address: Address) -> Result<usize, Error> {
        if address.less_than(self.base) {
            Err(Error::InvalidAddress)
        } else {
            Ok(address.offset_from(self.base))
        }
    }
}

// contains a vector of memory mappings sorted by start
pub struct MemoryMap {
    // TODO implement later for speedup, test speedup
    // cache: MemoryMapping, 
    entries: Vec<MemoryMapping>
}

impl MemoryMap {
    pub fn new(mut entries: Vec<MemoryMapping>) -> Self {
        entries.sort_by_key(|a| a.base.as_raw());
        Self{ entries }
    }

    pub fn get_entries<'a>(&'a self) -> &'a Vec<MemoryMapping> {
        &self.entries
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn get(&self, address: Address) -> Result<MemoryMapping, Error> {
        if address.less_than(self.entries[0].base) {
            return Err(Error::InvalidAddress)
        }

        // binary search for the correct section.  a match means the address is between the current and the next entry
        let mut start = 0;
        let mut end = self.entry_count();
        loop {
            let middle = start + (end - start) / 2;
            if address.less_than(self.entries[middle].base) {
                end = middle - 1;
            } else if middle == self.entry_count() - 1 || address.less_than(self.entries[middle + 1].base) {
                break Ok(self.entries[middle])
            } else {
                start = middle + 1
            }
        }
    }

    pub fn add(&mut self, mapping: MemoryMapping) -> ForthResult {
        match self.entries.binary_search_by_key(&mapping.base.as_raw(), |a| a.base.as_raw()) {
            Ok(_) => Err(Error::InvalidAddress),
            Err(i) => Ok(self.entries.insert(i, mapping)),
        }
    }
}

pub trait MemorySegment {
    fn get_base(&self) -> Address;
    fn get_end(&self) -> Address;
    fn check_address(&self, address: Address) -> Result<(), Error> {
        if address.between(self.get_base(), self.get_end()) {
            Ok(())
        } else {
            Err(Error::InvalidAddress)
        }
    }
    fn write_value(&mut self, address: Address, value: value::Value) -> Result<(), Error>;
    fn read_value(&self, address: Address) -> Result<value::Value, Error>;
}

pub struct Memory {
    base: Address,
    /* 
    The length may not be the length of the memory.  memory is the underlying
    representation of the memory, and it is allocated lazily.
     */
    length: usize,
    memory: Vec<value::Value>
}

impl Memory {
    pub fn new(base: usize) -> Self {
        Self { base: Address::from_raw(base), length: 0, memory: Vec::new() }
    }

    pub fn with_num_cells(mut self, num_cells: usize) -> Self {
        self.length = num_cells;
        self
    }

    pub fn top(&self) -> Address {
        self.base.plus_cell(self.length)
    }

    pub fn expand(&mut self, amount: usize) {
        self.length += amount;
    }

    pub fn push_value(&mut self, value: value::Value) {
        if self.memory.len() < self.length {
            self.memory.resize(self.length, 0.value());
        }
        self.length += 1;
        self.memory.push(value);
    }
    
    pub fn push<T: value::ValueVariant>(&mut self, value: T) {
        value.push_to_memory(self);
    }

    pub fn push_none<T: value::ValueVariant>(&mut self) {
        for _ in 0..T::size() {
            self.push_value(0.value())
        }
    }

    pub fn debug_only_get_vec<'a>(&'a self) -> &'a Vec<value::Value> {
        &self.memory
    }
}

impl MemorySegment for Memory {
    fn get_base(&self) -> Address {
        self.base
    }

    fn get_end(&self) -> Address {
        self.top()
    }

    fn write_value(&mut self, address: Address, value: value::Value) -> Result<(), Error> {
        self.check_address(address).map(|_| {
            let index = address.cell_offset_from(self.base);
            if index >= self.memory.len() {
                self.memory.resize(index + 1, 0.value())
            }
            self.memory[index] = value
        })
    }

    fn read_value(&self, address: Address) -> Result<value::Value, Error> {
        self.check_address(address).map(|_|{
            let index = address.cell_offset_from(self.base);
            if index >= self.memory.len() {
                0.value()
            } else {
                self.memory[index]
            }
        })
    }
}

impl generic_numbers::StackOperations<generic_numbers::Byte> for Memory {
    fn push_number_by_type(&mut self, byte: generic_numbers::Byte) {
        self.push_value(generic_numbers::Number::from_chunks(&[byte]).value())
    }

    fn pop_number_by_type(&mut self) -> Result<generic_numbers::Byte, Error> {
        // TODO fix this shit
        Err(Error::Exception(1))
    }
}

impl generic_numbers::StackOperations<generic_numbers::Number> for Memory {
    fn push_number_by_type(&mut self, number: generic_numbers::Number) {
        self.push_value(number.value())
    }

    fn pop_number_by_type(&mut self) -> Result<generic_numbers::Number, Error> {
        // TODO fix this shit
        Err(Error::Exception(1))
    }
}

impl generic_numbers::StackOperations<generic_numbers::DoubleNumber> for Memory {
    fn push_number_by_type(&mut self, double_number: generic_numbers::DoubleNumber) {
        double_number.to_chunks().iter().for_each(|c| self.push_value(c.value()))
    }

    fn pop_number_by_type(&mut self) -> Result<generic_numbers::DoubleNumber, Error> {
        // TODO fix this shit
        Err(Error::Exception(1))
    }
}

#[test]
fn memory_map_ordering_test() {
    let memory_map = MemoryMap::new(vec![
        MemoryMapping::empty(Address::from_raw(128), MemoryPermissions::readonly()),
        MemoryMapping::empty(Address::from_raw(32), MemoryPermissions::readonly().with_write()),
        MemoryMapping::empty(Address::from_raw(1024), MemoryPermissions::readonly().with_execute())
    ]);

    assert!(memory_map.get(Address::from_raw(50)).is_ok());
    assert_eq!(memory_map.get(Address::from_raw(130)).unwrap().base, Address::from_raw(128));
}