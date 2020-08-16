use std::collections::HashMap;

pub mod compiled_code;

use crate::operations;
use crate::memory;
use crate::stack;
use crate::io;


pub type ForthResult = Result<(), Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    DivisionByZero,
    StackUnderflow,
    UnknownWord,
    InvalidWord,
    InvalidAddress,
    InvalidNumber,
    InvalidExecutionToken,
    AddressOutOfRange,
    NoMoreTokens,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ControlFlowState {
    Continue,
    Jump(memory::Address),
    Break
}

pub type CodeResult = Result<ControlFlowState, Error>;

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
    pub execution_token: memory::ExecutionToken
}

impl Definition {
    pub fn new(execution_token: memory::ExecutionToken, immediate: bool) -> Self {
        Self { execution_token, immediate }
    }
}

pub struct DefinitionSet {
    nametag_map: HashMap<String, NameTag>,
    most_recent: NameTag,
    definitions: Vec<Definition>
}

impl DefinitionSet {
    fn from_definitions(definitions: Vec<Definition>, nametag_map: HashMap<String, NameTag>) -> Self {
        DefinitionSet {
            nametag_map,
            definitions,
            most_recent: NameTag(0)
        }
    }
    
    pub fn get_from_token(&self, token: io::tokens::Token) -> Option<Definition> {
        match token {
            io::tokens::Token::Integer(i) => Some(Definition::new(memory::ExecutionToken::Number(i), false)),
            io::tokens::Token::Name(name) => self.nametag_map.get(&name).map(|nametag| self.get(*nametag))
        }
    }

    pub fn get(&self, nametag: NameTag) -> Definition {
        self.definitions[nametag.to_offset()]
    }

    pub fn _get_from_name(&self, name: &str) -> Definition {
        self.get(*self.nametag_map.get(name).unwrap())
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

    pub fn get_most_recent(&self) -> NameTag {
        self.most_recent
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ExecutionMode {
    Compile,
    Interpret,
}

/**
 * This struct contains the state required to execute / emulate the code
 */
pub struct ForthState {
    pub compiled_code: compiled_code::CompiledCodeSegment,

    pub definitions: DefinitionSet,

    // the return stack is not actually used as a return stack, but is still provided for other uses
    pub return_stack: stack::Stack,
    pub stack: stack::Stack,
    pub memory: memory::Memory,

    pub execution_mode: ExecutionMode,

    pub output_stream: io::output_stream::OutputStream,
}

impl ForthState {
    pub fn new() -> Self {
        let default_operations = operations::get_operations();
        let definitions = default_operations.iter().map(|(_, immediate, operation)| {
            Definition::new(memory::ExecutionToken::Operation(*operation), *immediate)
        }).collect();
        let nametag_map = default_operations.iter().enumerate().map(|(i, (name, _, _))| (name.to_string(), NameTag(i))).collect();
        let definitions = DefinitionSet::from_definitions(definitions, nametag_map);

        let mut new_forth_state = Self {
            compiled_code: compiled_code::CompiledCodeSegment::new(),
            definitions,

            return_stack: stack::Stack::new(),
            stack: stack::Stack::new(),
            memory: memory::Memory::new(),

            execution_mode: ExecutionMode::Interpret,

            output_stream: io::output_stream::OutputStream::new()
        };

        for definition in operations::UNCOMPILED_OPERATIONS.iter() {
            let token_iterator = io::tokens::TokenStream::from_string(definition);
            new_forth_state.evaluate(token_iterator).unwrap_or_else(|error| panic!("Failed to parse preset definition: {:?} {:?}", definition, error));
        }

        new_forth_state
    }

    fn evaluator<'f, 'i>(&'f mut self, input_stream: io::tokens::TokenStream<'i>) -> ForthEvaluator<'f, 'i> {
        ForthEvaluator {
            input_stream: input_stream,
            compiled_code: self.compiled_code.borrow(),

            definitions: &mut self.definitions,

            return_stack: &mut self.return_stack,
            stack: &mut self.stack,
            memory: &mut self.memory,

            execution_mode: &mut self.execution_mode,

            output_stream: &mut self.output_stream
        }
    }

    pub fn evaluate(&mut self, mut input_stream: io::tokens::TokenStream) -> ForthResult {
        let mut control_flow_state = ControlFlowState::Continue;
        while control_flow_state == ControlFlowState::Continue {
            let mut evaluator = self.evaluator(input_stream);

            match evaluator.evaluate_once() {
                Result::Ok(state) => control_flow_state = state,
                Result::Err(error) => {
                    println!("error = {:?} before {:?}", error, evaluator.input_stream.next());
                    return Result::Err(error)
                }
            }

            input_stream = evaluator.input_stream;
            let buffer = evaluator.compiled_code.buffer;
            self.compiled_code.restore(buffer);
        }

        Result::Ok(())
    }
}

/**
 * 
 */
pub struct ForthEvaluator<'f, 'i> {
    pub input_stream: io::tokens::TokenStream<'i>,

    pub compiled_code: compiled_code::CompilingCodeSegment<'f>,

    pub definitions: &'f mut DefinitionSet,

    pub return_stack: &'f mut stack::Stack,
    pub stack: &'f mut stack::Stack,
    pub memory: &'f mut memory::Memory,

    pub execution_mode: &'f mut ExecutionMode,

    pub output_stream: &'f mut io::output_stream::OutputStream
}

impl<'f, 'i> ForthEvaluator<'f, 'i> {
    pub fn execute(&mut self, execution_token: memory::ExecutionToken) -> CodeResult {
        match execution_token {
            memory::ExecutionToken::Operation(fptr) => fptr(self),
            memory::ExecutionToken::DefinedOperation(_) => self.compiled_code.compiled_code.get(execution_token)(self),
            memory::ExecutionToken::Number(i) => {
                self.stack.push(memory::Value::Number(i));
                Result::Ok(ControlFlowState::Continue)
            }
        }
    }

    pub fn execute_at(&mut self, mut address: memory::Address) -> ForthResult {
        while let memory::Value::ExecutionToken(xt) = self.memory.read(address) {
            match self.execute(xt) {
                Result::Ok(ControlFlowState::Continue) => address.increment_cell(),
                Result::Ok(ControlFlowState::Break) => break,
                Result::Ok(ControlFlowState::Jump(new_address)) => address = new_address,
                Result::Err(error) => return Result::Err(error)
            }
        }

        Result::Ok(())
    }

    pub fn compile(&mut self, token: io::tokens::Token) -> CodeResult {
        let definition = match self.definitions.get_from_token(token) {
            Some(definition) => definition,
            None => return Result::Err(Error::UnknownWord)
        };

        if definition.immediate {
            match self.execute(definition.execution_token) {
                Result::Ok(_) => (),
                Result::Err(error) => return Result::Err(error)
            }
        } else {
            self.memory.push(definition.execution_token.value());
        }

        Result::Ok(ControlFlowState::Continue)
    }

    pub fn evaluate_once(&mut self) -> CodeResult {
        match self.input_stream.next() {
            Some(token) => if let ExecutionMode::Compile = self.execution_mode {
                self.compile(token)
            } else {
                self.definitions.get_from_token(token).ok_or(Error::UnknownWord).and_then(|definition| self.execute(definition.execution_token))
            }
            None => Result::Ok(ControlFlowState::Break)
        }
    }
}
