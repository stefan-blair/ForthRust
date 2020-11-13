use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::environment::{memory, generic_numbers, stack, value};
use crate::operations;
use crate::io::tokens;
use super::{ForthResult, Error};


#[derive(Clone, Copy)]
pub enum ExecutionToken {
    LeafOperation(operations::Operation),
    CompiledInstruction(usize),
    Definition(memory::Address),
    Number(generic_numbers::Number),
}

impl ExecutionToken {
    pub fn to_offset(self) -> usize {
        match self {
            Self::LeafOperation(fptr) => fptr as usize,
            Self::CompiledInstruction(i) => i,
            Self::Definition(address) => address.as_raw(),
            Self::Number(i) => i as usize
        }
    }

    pub fn value(self) -> value::Value {
        value::Value::ExecutionToken(self)
    }
}

impl ToString for ExecutionToken {
    fn to_string(&self) -> String {
        match *self {
            Self::LeafOperation(operation) => format!("operation @ {}", (operation as usize)),
            Self::CompiledInstruction(offset) => format!("compiled instruction @ offset {}", offset),
            Self::Definition(address) => format!("definition @ {:#x}", address.to_number()),
            Self::Number(number) => format!("push {}", number)
        }
    }
}

impl Hash for ExecutionToken {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let index = match self {
            Self::LeafOperation(_) => 0,
            Self::CompiledInstruction(_) => 1,
            Self::Definition(_) => 2,
            Self::Number(_) => 4,
        };
        index.hash(state);
        self.to_offset().hash(state);
    }
}

impl PartialEq for ExecutionToken {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Self::LeafOperation(op_1), Self::LeafOperation(op_2)) => (op_1 as usize) == (op_2 as usize),
            (Self::CompiledInstruction(offset_1), Self::CompiledInstruction(offset_2)) => offset_1 == offset_2,
            (Self::Definition(address_1), Self::Definition(address_2)) => address_1 == address_2,
            (Self::Number(i), Self::Number(j)) => i == j,
            _ => false
        }
    }
}
impl Eq for ExecutionToken {}

impl value::ValueVariant for ExecutionToken {
    fn push_to_stack(self, stack: &mut stack::Stack) {
        stack.push(self.value())
    }

    fn pop_from_stack(stack: &mut stack::Stack) -> Result<Self, Error> {
        match stack.pop()? {
            value::Value::ExecutionToken(xt) => Ok(xt),
            _ => Err(Error::InvalidExecutionToken)
        }
    }

    fn write_to_memory(self, memory: &mut dyn memory::MemorySegment, address: memory::Address) -> Result<(), Error> {
        memory.write_value(address, self.value())
    }
    
    fn read_from_memory(memory: &dyn memory::MemorySegment, address: memory::Address) -> Result<Self, Error> {
        memory.read_value(address).map(|value| match value {
            value::Value::ExecutionToken(xt) => xt,
            value::Value::Number(n) => ExecutionToken::Number(n)

        })
    }

    fn push_to_memory(self, memory: &mut memory::Memory) {
        memory.push_value(self.value())
    }

    fn size() -> usize {
        1
    }
}

#[derive(Clone, Copy)]
pub struct Definition {
    pub immediate: bool,
    pub execution_token: ExecutionToken,
}

impl Definition {
    pub fn new(execution_token: ExecutionToken, immediate: bool) -> Self {
        Self { execution_token, immediate }
    }
}

pub enum NameTag {
    Definition(usize),
    TempDefinition(usize),
}

pub struct DefinitionTable {
    nametag_map: HashMap<String, usize>,
    definitions: Vec<Definition>,
    most_recent: usize,

    temp_nametag_map: HashMap<String, usize>,
    temp_definitions: Vec<Definition>,
}

impl DefinitionTable {
    pub fn new() -> Self {
        Self::from_definitions(Vec::new(), HashMap::new())
    }

    pub fn from_definitions(definitions: Vec<Definition>, nametag_map: HashMap<String, usize>) -> Self {
        Self {
            nametag_map,
            definitions,
            most_recent: 0,

            temp_nametag_map: HashMap::new(),
            temp_definitions: Vec::new(),
        }
    }
    
    pub fn get_from_token(&self, token: tokens::Token) -> Result<Definition, Error> {
        match token {
            tokens::Token::Integer(i) => Ok(Definition::new(ExecutionToken::Number(i), false)),
            tokens::Token::Word(word) => self.get_from_str(&word),
        }
    }

    pub fn get_from_str(&self, name: &str) -> Result<Definition, Error> {
        self.nametag_map.get(name).map(|nametag| self.definitions[*nametag])
            .or_else(|| self.temp_nametag_map.get(name).map(|nametag| self.temp_definitions[*nametag]))
            .ok_or(Error::UnknownWord(name.to_string()))
    }

    pub fn get_by_index(&self, index: usize) -> Result<Definition, Error> {
        if index >= self.definitions.len() {
            Err(Error::InvalidNumber)
        } else {
            Ok(self.definitions[index])
        }
    }

    pub fn set_by_index(&mut self, index: usize, definition: Definition) -> ForthResult {
        if index >= self.definitions.len() {
            Err(Error::InvalidNumber)
        } else {
            self.definitions[index] = definition;
            Ok(())
        }
    }
    
    pub fn get_temp_by_index(&self, index: usize) -> Result<Definition, Error> {
        if index >= self.temp_definitions.len() {
            Err(Error::InvalidNumber)
        } else {
            Ok(self.temp_definitions[index])
        }
    }

    pub fn set_temp_by_index(&mut self, index: usize, definition: Definition) -> ForthResult {
        if index >= self.temp_definitions.len() {
            Err(Error::InvalidNumber)
        } else {
            self.temp_definitions[index] = definition;
            Ok(())
        }
    }

    pub fn get_nametag(&self, name: &str) -> Result<NameTag, Error> {
        self.nametag_map.get(name).map(|nametag| NameTag::Definition(*nametag))
            .or_else(|| self.temp_nametag_map.get(name).map(|nametag| NameTag::TempDefinition(*nametag)))
            .ok_or(Error::UnknownWord(name.to_string()))
    } 

    // fn get_nametag(&self, word: &str) -> Result<NameTag, Error> {
    //     self.nametag_map.get(word).map(|x| *x).ok_or(Error::UnknownWord(String::from(word)))
    // }

    pub fn make_most_recent_immediate(&mut self) {
        self.definitions[self.most_recent].immediate = true
    }

    pub fn most_recent_definition(&self) -> &Definition {
        &self.definitions[self.most_recent]
    }

    pub fn add(&mut self, word: String, definition: Definition) {
        let index = self.definitions.len();
        self.nametag_map.insert(word, index);
        self.definitions.push(definition);
        self.most_recent = index;
    }

    pub fn add_temp(&mut self, word: String, definition: Definition) {
        let index = self.temp_definitions.len();
        self.temp_nametag_map.insert(word, index);
        self.temp_definitions.push(definition);
    }

    pub fn clear_temp(&mut self) {
        self.temp_nametag_map = HashMap::new();
        self.temp_definitions = Vec::new();
    }

    pub fn debug_only_get_name(&self, execution_token: ExecutionToken) -> Option<String> {
        fn equal(a: ExecutionToken, b: ExecutionToken) -> bool {
            match (a, b) {
                (ExecutionToken::Number(a), ExecutionToken::Number(b)) => a == b,
                (ExecutionToken::LeafOperation(a), ExecutionToken::LeafOperation(b)) => (a as usize) == (b as usize),
                (ExecutionToken::Definition(a), ExecutionToken::Definition(b)) => a == b,
                (ExecutionToken::CompiledInstruction(a), ExecutionToken::CompiledInstruction(b)) => a == b,
                _ => false
            }
        }

        for (word, xt) in self.nametag_map.iter().map(|(word, index)| (word, self.get_by_index(*index).unwrap().execution_token)) {
            if equal(execution_token, xt) {
                return Some(word.clone())
            }
        }

        None
    }

    pub fn debug_only_get_nametag_map(&self) -> &HashMap<String, usize> {
        return &self.nametag_map;
    }
}