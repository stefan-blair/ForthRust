pub mod compiled_code;
pub mod definition;

use crate::operations;
use crate::environment::{memory, stack};
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
    
    // this isn't a bad error, just a result that the input stream has finished cleanly
    TokenStreamEmpty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    pub instruction_pointer: Option<memory::Address>,

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
            instruction_pointer: None,

            output_stream: output_stream::OutputStream::new()
        };

        for definition in operations::UNCOMPILED_OPERATIONS.iter() {
            let token_iterator = tokens::TokenStream::new(definition.chars());
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
            instruction_pointer: &mut self.instruction_pointer,

            output_stream: &mut self.output_stream
        }
    }

    pub fn evaluate(&mut self, mut input_stream: tokens::TokenStream) -> ForthResult {
        loop {
            let mut evaluator = self.evaluator(input_stream);

            match evaluator.step() {
                Result::Err(Error::TokenStreamEmpty) => break,
                Result::Err(error) => {
                    println!("error = {:?} before {:?}", error, evaluator.input_stream.next());
                    return Result::Err(error)
                },
                Result::Ok(_) => ()
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

    pub instruction_pointer: &'f mut Option<memory::Address>,
    pub execution_mode: &'f mut ExecutionMode,
}

impl<'f, 'i> ForthEvaluator<'f, 'i> {
    pub fn execute(&mut self, execution_token: definition::ExecutionToken) -> ForthResult {
        match execution_token {
            definition::ExecutionToken::DefinedOperation(address) => self.invoke_at(address),
            definition::ExecutionToken::Operation(fptr) => fptr(self),
            definition::ExecutionToken::CompiledOperation(_) => self.compiled_code.compiled_code.get(execution_token)(self),
            definition::ExecutionToken::Number(i) => Result::Ok(self.stack.push(i))
        }
    }

    fn invoke_at(&mut self, address: memory::Address) -> ForthResult {
        self.instruction_pointer.replace(address).map(|addr| {
            self.return_stack.push(addr.to_number());
        });

        self.execute_next()
    }

    pub fn return_from(&mut self) -> ForthResult {
        *self.instruction_pointer = self.return_stack.pop().and_then(|number| self.memory.address_from(number));
        Result::Ok(())
    }

    pub fn jump_to(&mut self, address: memory::Address) -> ForthResult {
        *self.instruction_pointer = Some(address);
        Result::Ok(())
    }

    pub fn set_compilemode(&mut self) -> ForthResult {
        *self.execution_mode = ExecutionMode::Compile;
        Result::Ok(())
    }

    pub fn set_interpretmode(&mut self) -> ForthResult {
        *self.execution_mode = ExecutionMode::Interpret;
        Result::Ok(())
    }

    fn execute_next(&mut self) -> ForthResult {
        if let Some(instruction_pointer) = *self.instruction_pointer {
            *self.instruction_pointer = Some(instruction_pointer.plus_cell(1));
            let xt = self.memory.read(instruction_pointer);
            self.execute(xt)
        } else {
            Result::Err(Error::InvalidAddress)
        }
    }

    fn step(&mut self) -> ForthResult {
        // attempt to execute the next instruction
        self.execute_next()
            // if theres no more instruction to execute, attempt to read the next token 
            .or_else(|_| {
                self.input_stream.next().ok_or(Error::TokenStreamEmpty)
                    .and_then(|token| self.definitions.get_from_token(token).ok_or(Error::UnknownWord))
                    .and_then(|definition| if *self.execution_mode == ExecutionMode::Compile && !definition.immediate {
                        Result::Ok(self.memory.push(definition.execution_token.value()))
                    } else {
                        self.execute(definition.execution_token)
                    })
            })
    }
}
