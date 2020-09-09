use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::environment::{memory, generic_numbers, stack, value};
use crate::operations;
use crate::io::tokens;
use super::Error;


#[derive(Clone, Copy)]
pub enum ExecutionToken {
    Operation(operations::Operation),
    CompiledOperation(memory::Offset),
    DefinedOperation(memory::Address),
    Number(generic_numbers::Number),
}

impl ExecutionToken {
    pub fn to_offset(self) -> memory::Offset {
        match self {
            Self::Operation(fptr) => fptr as memory::Offset,
            Self::CompiledOperation(i) => i,
            Self::DefinedOperation(address) => address.to_offset(),
            Self::Number(i) => i as memory::Offset
        }
    }

    pub fn value(self) -> value::Value {
        value::Value::ExecutionToken(self)
    }
}

impl Hash for ExecutionToken {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let index = match self {
            Self::Operation(_) => 0,
            Self::CompiledOperation(_) => 1,
            Self::DefinedOperation(_) => 2,
            Self::Number(_) => 3,
        };
        index.hash(state);
        self.to_offset().hash(state);
    }
}

impl PartialEq for ExecutionToken {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Self::Operation(op_1), Self::Operation(op_2)) => (op_1 as usize) == (op_2 as usize),
            (Self::CompiledOperation(offset_1), Self::CompiledOperation(offset_2)) => offset_1 == offset_2,
            (Self::DefinedOperation(address_1), Self::DefinedOperation(address_2)) => address_1 == address_2,
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

    fn pop_from_stack(stack: &mut stack::Stack) -> Option<Self> {
        stack.pop().and_then(|value| match value {
            value::Value::ExecutionToken(xt) => Some(xt),
            _ => None
        })
    }

    fn write_to_memory(self, memory: &mut memory::Memory, address: memory::Address) -> bool{
        memory.write_value(address, self.value())
    }

    fn read_from_memory(memory: &memory::Memory, address: memory::Address) -> Option<Self> {
        memory.read_value(address).map(|value| match value {
            value::Value::ExecutionToken(xt) => xt,
            value::Value::Number(n) => ExecutionToken::Number(n)

        })
    }

    fn push_to_memory(self, memory: &mut memory::Memory) {
        memory.push_value(self.value())
    }

    fn null() -> Self {
        Self::Number(0)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NameTag(pub memory::Offset);

impl NameTag {
    pub fn to_offset(self) -> memory::Offset {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct Definition {
    pub immediate: bool,
    pub execution_token: ExecutionToken
}

impl Definition {
    pub fn new(execution_token: ExecutionToken, immediate: bool) -> Self {
        Self { execution_token, immediate }
    }
}

pub struct DefinitionSet {
    nametag_map: HashMap<String, NameTag>,
    most_recent: NameTag,
    definitions: Vec<Definition>
}

impl DefinitionSet {
    pub fn new() -> Self {
        Self::from_definitions(Vec::new(), HashMap::new())
    }

    pub fn from_definitions(definitions: Vec<Definition>, nametag_map: HashMap<String, NameTag>) -> Self {
        DefinitionSet {
            nametag_map,
            definitions,
            most_recent: NameTag(0)
        }
    }
    
    pub fn get_from_token(&self, token: tokens::Token) -> Result<Definition, Error> {
        match token {
            tokens::Token::Integer(i) => Ok(Definition::new(ExecutionToken::Number(i), false)),
            tokens::Token::Name(name) => self.nametag_map.get(&name).map(|nametag| self.get(*nametag)).ok_or(Error::UnknownWord(name))
        }
    }

    pub fn get(&self, nametag: NameTag) -> Definition {
        self.definitions[nametag.to_offset()]
    }

    pub fn get_nametag(&self, name: &str) -> Option<NameTag> {
        self.nametag_map.get(name).map(|x| *x)
    }

    pub fn make_immediate(&mut self, nametag: NameTag) {
        self.definitions[nametag.to_offset()].immediate = true;
    }

    pub fn add(&mut self, name: String, definition: Definition) -> NameTag {
        let nametag = NameTag(self.definitions.len());
        self.nametag_map.insert(name, nametag);
        self.definitions.push(definition);
        self.most_recent = nametag;

        nametag
    }

    pub fn set(&mut self, nametag: NameTag, definition: Definition) -> NameTag {
        self.definitions[nametag.to_offset()] = definition;
        nametag
    }

    pub fn get_most_recent_nametag(&self) -> NameTag {
        self.most_recent
    }

    pub fn debug_only_get_name(&self, execution_token: ExecutionToken) -> Option<String> {
        fn equal(a: ExecutionToken, b: ExecutionToken) -> bool {
            match (a, b) {
                (ExecutionToken::Number(a), ExecutionToken::Number(b)) => a == b,
                (ExecutionToken::Operation(a), ExecutionToken::Operation(b)) => (a as usize) == (b as usize),
                (ExecutionToken::DefinedOperation(a), ExecutionToken::DefinedOperation(b)) => a == b,
                (ExecutionToken::CompiledOperation(a), ExecutionToken::CompiledOperation(b)) => a == b,
                _ => false
            }
        }

        for (name, xt) in self.nametag_map.iter().map(|(name, key)| (name, self.get(*key).execution_token)) {
            if equal(execution_token, xt) {
                return Some(name.clone())
            }
        }

        None
    }

    pub fn debug_only_get_nametag_map(&self) -> &HashMap<String, NameTag> {
        return &self.nametag_map;
    }
}