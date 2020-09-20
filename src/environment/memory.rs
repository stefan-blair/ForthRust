use std::mem;

use crate::evaluate::{ForthState, Error};
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
    pub fn from_raw(raw: usize) -> Self {
        Self(raw)
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

    pub fn less_than(self, other: Address) -> bool {
        self.0 < other.0
    }

    pub fn between(self, lower: Address, upper: Address) -> bool {
        self.0 >= lower.0 && self.0 < upper.0
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

    pub fn as_raw(self) -> Offset {
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

    fn size() -> Offset {
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

type MutableMemorySegmentGetter = for<'a> fn(&'a mut ForthState) -> &'a mut (dyn MemorySegment + 'a);
type MemorySegmentGetter = for<'a> fn(&'a ForthState) -> &'a (dyn MemorySegment + 'a);

#[derive(Clone, Copy)]
pub enum MemoryMappingType {
    // this type of mapping is for a normal memory segment, like the stack or the heap
    Normal(MutableMemorySegmentGetter, MemorySegmentGetter),
    // this type of mapping is for a special location in memory, like the `state`
    Special(fn(&ForthState) -> value::Value),
    Empty
}

#[derive(Clone, Copy)]
pub struct MemoryMapping {
    pub base: Address,
    pub permissions: MemoryPermissions,
    pub mapping_type: MemoryMappingType
}

impl MemoryMapping {
    pub fn new(base: Address, permissions: MemoryPermissions, getter: MemorySegmentGetter, mutable_getter: MutableMemorySegmentGetter) -> Self {
        Self { base, permissions, mapping_type: MemoryMappingType::Normal(mutable_getter, getter) }
    }

    pub fn empty(base: Address, permissions: MemoryPermissions) -> Self {
        Self { base, permissions, mapping_type: MemoryMappingType::Empty }
    }

    pub fn special_mapping(base: Address, permissions: MemoryPermissions, resolver: fn(&ForthState) -> value::Value) -> Self {
        Self { base, permissions, mapping_type: MemoryMappingType::Special(resolver) }
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

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn get(&self, address: Address) -> Result<&MemoryMapping, Error> {
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
                break Ok(&self.entries[middle])
            } else {
                start = middle + 1
            }
        }
    }
}

pub trait MemorySegment {
    fn get_base(&self) -> Address;
    fn check_address(&self, address: Address) -> Result<(), Error>;
    fn write_value(&mut self, address: Address, value: value::Value) -> Result<(), Error>;
    fn read_value(&self, address: Address) -> Result<value::Value, Error>;
}

pub struct Memory{
    base: Address,
    memory: Vec<value::Value>
}

impl Memory {
    pub fn new(base: usize) -> Self {
        Self { base: Address::from_raw(base), memory: Vec::new()}
    }

    pub fn top(&self) -> Address {
        self.base.plus_cell(self.memory.len())
    }

    pub fn expand(&mut self, amount: Offset) {
        self.memory.resize(self.memory.len() + amount, 0.value())
    }

    pub fn push_value(&mut self, value: value::Value) {
        self.memory.push(value);
    }
    
    pub fn push<T: value::ValueVariant>(&mut self, value: T) {
        value.push_to_memory(self);
    }

    pub fn push_none<T: value::ValueVariant>(&mut self) {
        for _ in 0..T::size() {
            self.memory.push(0.value());
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

    fn check_address(&self, address: Address) -> Result<(), Error> {
        if address.between(self.base, self.top()) {
            Ok(())
        } else {
            Err(Error::InvalidAddress)
        }
    }

    fn write_value(&mut self, address: Address, value: value::Value) -> Result<(), Error> {
        self.check_address(address).map(|_| {
            self.memory[address.cell_offset_from(self.base)] = value
        })
    }

    fn read_value(&self, address: Address) -> Result<value::Value, Error> {
        self.check_address(address).map(|_|{
            self.memory[address.cell_offset_from(self.base)]
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