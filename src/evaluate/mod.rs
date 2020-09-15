pub mod definition;
pub mod kernels;

use crate::operations;
use crate::environment::{memory, stack};
use crate::io::{tokens, output_stream};
use crate::compiled_instructions;


pub type ForthResult = Result<(), Error>;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    DivisionByZero,
    StackUnderflow,
    UnknownWord(String),
    InvalidWord,
    InvalidAddress,
    InvalidNumber,
    InvalidExecutionToken,
    AddressOutOfRange,
    NoMoreTokens,
    
    // this isn't a bad error, just a result that the input stream has finished cleanly
    TokenStreamEmpty,
    // this isn't a bad error, just a result that some command has asked to halt execution
    Halt
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionMode {
    Compile,
    Interpret,
}

pub struct ForthIO<'a, 't> {
    pub input_stream: &'a mut tokens::TokenStream<'t>,
    pub output_stream: &'a mut dyn output_stream::OutputStream
}

impl<'a, 't> ForthIO<'a, 't> {
    pub fn new(input_stream: &'a mut tokens::TokenStream<'t>, output_stream: &'a mut dyn output_stream::OutputStream) -> Self {
        Self { input_stream, output_stream }
    }

    pub fn borrow<'b>(&'b mut self) -> ForthIO<'b, 't> {
        ForthIO { input_stream: self.input_stream, output_stream: self.output_stream }
    }
}

pub struct Forth<'a, 'i, 'ro, 'o, KERNEL: kernels::Kernel> {
    pub state: ForthState<'a, 'i, 'ro, 'o>,
    pub kernel: KERNEL
}

impl<'a, 'i, 'ro, 'o, KERNEL: kernels::Kernel> Forth<'a, 'i, 'ro, 'o, KERNEL> {
    pub fn new() -> Self {
        let mut state = ForthState::new();
        let kernel = KERNEL::new(&mut state);
        let mut forth_machine = Self { state, kernel };

        for definition in operations::UNCOMPILED_OPERATIONS.iter() {
            forth_machine.evaluate(tokens::TokenStream::new(definition.chars())).unwrap_or_else(|error| panic!("Failed to parse preset definition: {:?} {:?}", definition, error));
        }

        forth_machine
    }

    pub fn set_output_stream<O: output_stream::OutputStream + 'o> (&mut self, output: &'ro mut O) {
        self.state.output_stream = output_stream::OptionalOutputStream::with(output);
    }

    pub fn with_output_stream<O: output_stream::OutputStream + 'o> (mut self, output: &'ro mut O) -> Self {
        self.state.output_stream = output_stream::OptionalOutputStream::with(output);
        self
    }

    pub fn evaluate_string(&mut self, input: &'i str) -> ForthResult {
        self.evaluate_stream(input.chars())
    }

    pub fn evaluate_stream<I: Iterator<Item = char> + 'i>(&mut self, stream: I) -> ForthResult {
        self.evaluate(tokens::TokenStream::new(stream))
    }

    pub fn evaluate(&mut self, input_stream: tokens::TokenStream<'i>) -> ForthResult {    
        self.state.input_stream = input_stream;
        loop {
            match self.kernel.evaluate_chain(&mut self.state)
                    .and_then(|_| self.state.execute_current_instruction())
                    .or_else(|error| self.kernel.handle_error_chain(&mut self.state, error)) 
                    .and_then(|_| self.state.fetch_current_instruction())
                    .or_else(|error| self.kernel.handle_error_chain(&mut self.state, error)) {
                Err(Error::TokenStreamEmpty) | Err(Error::Halt) => break,
                Err(error) => return Err(error),
                Ok(_) => ()
            }
        }
        Ok(())
    }
}

/**
 * This struct contains the state required to execute / emulate the code
 */
pub struct ForthState<'a, 'i, 'ro, 'o> {
    pub definitions: definition::DefinitionSet,
    pub compiled_instructions: compiled_instructions::CompiledInstructions<'a>,
    // the return stack is not actually used as a return stack, but is still provided for other uses
    pub return_stack: stack::Stack,
    pub stack: stack::Stack,
    pub memory: memory::Memory,

    pub execution_mode: ExecutionMode,
    // pointer to the next instruction to execute
    pub instruction_pointer: Option<memory::Address>,
    // contains the current instruction, if any, being executed
    pub current_instruction: Option<definition::ExecutionToken>,

    pub output_stream: output_stream::OptionalOutputStream<'ro, 'o>,
    pub input_stream: tokens::TokenStream<'i>
}

// split it up into some sort of ForthState vs. ForthMachine, so ForthMachine -> ForthState -> ForthEvaluator ...
impl<'a, 'i, 'ro, 'o> ForthState<'a, 'i, 'ro, 'o> {
    pub fn new() -> Self {
        Self {
            compiled_instructions: compiled_instructions::CompiledInstructions::new(),
            definitions: definition::DefinitionSet::new(),

            return_stack: stack::Stack::new(),
            stack: stack::Stack::new(),
            memory: memory::Memory::new(),

            execution_mode: ExecutionMode::Interpret,
            instruction_pointer: None,
            current_instruction: None,

            output_stream: output_stream::OptionalOutputStream::empty(),
            input_stream: tokens::TokenStream::empty(),
        }.with_operations(operations::get_operations())
    }

    pub fn add_operations(&mut self, operations: operations::OperationTable) {
        for (word, immediate, operation) in operations {
            self.definitions.add(word.to_string(), definition::Definition::new(definition::ExecutionToken::LeafOperation(operation), immediate));
        };
    }

    pub fn with_operations(mut self, operations: operations::OperationTable) -> Self {
        self.add_operations(operations);
        self
    }

    pub fn execute(&mut self, execution_token: definition::ExecutionToken) -> ForthResult {
        match execution_token {
            definition::ExecutionToken::Definition(address) => self.invoke_at(address),
            definition::ExecutionToken::LeafOperation(fptr) => fptr(self),
            definition::ExecutionToken::CompiledInstruction(_) => self.compiled_instructions.get(execution_token).execute(self),
            definition::ExecutionToken::Number(i) => Ok(self.stack.push(i))
        }
    }

    fn invoke_at(&mut self, address: memory::Address) -> ForthResult {
        self.instruction_pointer.replace(address).map(|addr| {
            self.return_stack.push(addr.to_number());
        });
        Ok(())
    }

    pub fn return_from(&mut self) -> ForthResult {
        self.instruction_pointer = self.return_stack.pop().ok();
        Ok(())
    }

    pub fn jump_to(&mut self, address: memory::Address) -> ForthResult {
        self.instruction_pointer = Some(address);
        Ok(())
    }

    pub fn set_compilemode(&mut self) -> ForthResult {
        self.execution_mode = ExecutionMode::Compile;
        Ok(())
    }

    pub fn set_interpretmode(&mut self) -> ForthResult {
        self.execution_mode = ExecutionMode::Interpret;
        Ok(())
    }

    fn fetch_current_instruction(&mut self) -> ForthResult {
        self.instruction_pointer.ok_or(Error::InvalidAddress)
            .map(|instruction_pointer| {
                // fetch the current instruction
                self.current_instruction = self.memory.read(instruction_pointer).ok();
            }).or_else(|_| self.input_stream.next().ok().ok_or(Error::TokenStreamEmpty)
            .and_then(|token| self.definitions.get_from_token(token))
            .map(|definition| if self.execution_mode == ExecutionMode::Compile && !definition.immediate {
                self.memory.push(definition.execution_token.value());
                self.current_instruction = None;
            } else {
                self.current_instruction = Some(definition.execution_token);
            })
        )
    }

    fn execute_current_instruction(&mut self) -> ForthResult {
        // execute the current instruction, 'take'ing it so its None, and incrementing the current instruction pointer to the next position for the next iteration 
        self.instruction_pointer = self.instruction_pointer.map(|ip| ip.plus_cell(1));
        match self.current_instruction.take() {
            Some(xt) => self.execute(xt),
            None => Ok(())
        }
    }
}
