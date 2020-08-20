pub mod compiled_code;
pub mod definition;

use crate::operations;
use crate::environment::{value, memory, stack};
use crate::io::{tokens, output_stream};


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

    pub definitions: definition::DefinitionSet,

    // the return stack is not actually used as a return stack, but is still provided for other uses
    pub return_stack: stack::Stack,
    pub stack: stack::Stack,
    pub memory: memory::Memory,

    pub execution_mode: ExecutionMode,

    pub output_stream: output_stream::OutputStream,
}

impl ForthState {
    pub fn new() -> Self {
        let default_operations = operations::get_operations();
        let definitions = default_operations.iter().map(|(_, immediate, operation)| {
            definition::Definition::new(definition::ExecutionToken::Operation(*operation), *immediate)
        }).collect();
        let nametag_map = default_operations.iter().enumerate().map(|(i, (name, _, _))| (name.to_string(), definition::NameTag(i))).collect();
        let definitions = definition::DefinitionSet::from_definitions(definitions, nametag_map);

        let mut new_forth_state = Self {
            compiled_code: compiled_code::CompiledCodeSegment::new(),
            definitions,

            return_stack: stack::Stack::new(),
            stack: stack::Stack::new(),
            memory: memory::Memory::new(),

            execution_mode: ExecutionMode::Interpret,

            output_stream: output_stream::OutputStream::new()
        };

        for definition in operations::UNCOMPILED_OPERATIONS.iter() {
            let token_iterator = tokens::TokenStream::from_string(definition);
            new_forth_state.evaluate(token_iterator).unwrap_or_else(|error| panic!("Failed to parse preset definition: {:?} {:?}", definition, error));
        }

        new_forth_state
    }

    fn evaluator<'f, 'i>(&'f mut self, input_stream: tokens::TokenStream<'i>) -> ForthEvaluator<'f, 'i> {
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

    pub fn evaluate(&mut self, mut input_stream: tokens::TokenStream) -> ForthResult {
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
    pub input_stream: tokens::TokenStream<'i>,
    pub output_stream: &'f mut output_stream::OutputStream,

    pub compiled_code: compiled_code::CompilingCodeSegment<'f>,

    pub definitions: &'f mut definition::DefinitionSet,

    pub return_stack: &'f mut stack::Stack,
    pub stack: &'f mut stack::Stack,
    pub memory: &'f mut memory::Memory,

    pub execution_mode: &'f mut ExecutionMode,
}

impl<'f, 'i> ForthEvaluator<'f, 'i> {
    pub fn execute(&mut self, execution_token: definition::ExecutionToken) -> CodeResult {
        match execution_token {
            definition::ExecutionToken::Operation(fptr) => fptr(self),
            definition::ExecutionToken::DefinedOperation(_) => self.compiled_code.compiled_code.get(execution_token)(self),
            definition::ExecutionToken::Number(i) => {
                self.stack.push(i);
                Result::Ok(ControlFlowState::Continue)
            }
        }
    }

    pub fn execute_at(&mut self, mut address: memory::Address) -> ForthResult {
        while let value::Value::ExecutionToken(xt) = self.memory.read(address) {
            match self.execute(xt) {
                Result::Ok(ControlFlowState::Continue) => address.increment_cell(),
                Result::Ok(ControlFlowState::Break) => break,
                Result::Ok(ControlFlowState::Jump(new_address)) => address = new_address,
                Result::Err(error) => return Result::Err(error)
            }
        }

        Result::Ok(())
    }

    pub fn compile(&mut self, token: tokens::Token) -> CodeResult {
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
