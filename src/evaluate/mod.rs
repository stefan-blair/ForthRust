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
pub struct ForthState<'a> {
    pub compiled_code: compiled_code::CompiledCodeSegment<'a>,

    pub definitions: definition::DefinitionSet,

    // the return stack is not actually used as a return stack, but is still provided for other uses
    pub return_stack: stack::Stack,
    pub stack: stack::Stack,
    pub memory: memory::Memory,

    pub execution_mode: ExecutionMode,
    pub instruction_pointer: Option<memory::Address>,
}

impl<'a> ForthState<'a> {
    pub fn new() -> Self {
        let mut new_forth_state = Self {
            compiled_code: compiled_code::CompiledCodeSegment::new(),
            definitions: definition::DefinitionSet::new(),

            return_stack: stack::Stack::new(),
            stack: stack::Stack::new(),
            memory: memory::Memory::new(),

            execution_mode: ExecutionMode::Interpret,
            instruction_pointer: None,
        }.with_operations(operations::get_operations());

        let mut dummy_output = output_stream::DropOutputStream::new();
        for definition in operations::UNCOMPILED_OPERATIONS.iter() {
            let token_iterator = tokens::TokenStream::new(definition.chars());
            new_forth_state.evaluate(token_iterator, &mut dummy_output).unwrap_or_else(|error| panic!("Failed to parse preset definition: {:?} {:?}", definition, error));
        }

        new_forth_state
    }

    pub fn add_operations(&mut self, operations: operations::OperationTable) {
        for (name, immediate, operation) in operations {
            self.definitions.add(name.to_string(), definition::Definition::new(definition::ExecutionToken::Operation(operation), immediate));
        };
    }

    pub fn with_operations(mut self, operations: operations::OperationTable) -> Self {
        self.add_operations(operations);
        self
    }

    pub fn evaluate<'f, 'i>(&'f mut self, mut input_stream: tokens::TokenStream<'i>, mut output_stream: &'i mut dyn output_stream::OutputStream) -> ForthResult {
        loop {
            let mut evaluator = ForthEvaluator {
                input_stream: input_stream,
                output_stream: output_stream,

                compiled_code: self.compiled_code.borrow(),
    
                definitions: &mut self.definitions,
    
                return_stack: &mut self.return_stack,
                stack: &mut self.stack,
                memory: &mut self.memory,
    
                execution_mode: &mut self.execution_mode,
                instruction_pointer: &mut self.instruction_pointer,
            };
            
            match evaluator.step() {
                Err(Error::TokenStreamEmpty) => break,
                Err(error) => {
                    println!("error = {:?}", error);
                    return Err(error)
                },
                Ok(_) => ()
            }

            input_stream = evaluator.input_stream;
            output_stream = evaluator.output_stream;

            let buffer = evaluator.compiled_code.buffer;
            self.compiled_code.restore(buffer);
        }

        Ok(())
    }
}

/**
 * 
 */
pub struct ForthEvaluator<'f, 'i, 'a, 'b> {
    pub input_stream: tokens::TokenStream<'i>,
    pub output_stream: &'i mut dyn output_stream::OutputStream,

    pub compiled_code: compiled_code::CompilingCodeSegment<'a, 'b>,

    pub definitions: &'f mut definition::DefinitionSet,

    pub return_stack: &'f mut stack::Stack,
    pub stack: &'f mut stack::Stack,
    pub memory: &'f mut memory::Memory,

    pub instruction_pointer: &'f mut Option<memory::Address>,
    pub execution_mode: &'f mut ExecutionMode,
}

impl<'f, 'i, 'a, 'b> ForthEvaluator<'f, 'i, 'a, 'b> {
    pub fn execute(&mut self, execution_token: definition::ExecutionToken) -> ForthResult {
        match execution_token {
            definition::ExecutionToken::DefinedOperation(address) => self.invoke_at(address),
            definition::ExecutionToken::Operation(fptr) => fptr(self),
            definition::ExecutionToken::CompiledOperation(_) => self.compiled_code.compiled_code.get(execution_token)(self),
            definition::ExecutionToken::Number(i) => Ok(self.stack.push(i))
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
        Ok(())
    }

    pub fn jump_to(&mut self, address: memory::Address) -> ForthResult {
        *self.instruction_pointer = Some(address);
        Ok(())
    }

    pub fn set_compilemode(&mut self) -> ForthResult {
        *self.execution_mode = ExecutionMode::Compile;
        Ok(())
    }

    pub fn set_interpretmode(&mut self) -> ForthResult {
        *self.execution_mode = ExecutionMode::Interpret;
        Ok(())
    }

    fn execute_next(&mut self) -> ForthResult {
        if let Some(instruction_pointer) = *self.instruction_pointer {
            *self.instruction_pointer = Some(instruction_pointer.plus_cell(1));
            let xt = self.memory.read(instruction_pointer);
            self.execute(xt)
        } else {
            Err(Error::InvalidAddress)
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
                        Ok(self.memory.push(definition.execution_token.value()))
                    } else {
                        self.execute(definition.execution_token)
                    })
            })
    }
}
