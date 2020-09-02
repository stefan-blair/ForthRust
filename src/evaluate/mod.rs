pub mod compiled_code;
pub mod definition;
pub mod kernels;

use crate::operations;
use crate::environment::{memory, stack};
use crate::io::{tokens, output_stream};


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

pub struct Forth<'a, KERNEL: kernels::Kernel> {
    pub state: ForthState<'a>,
    kernel: KERNEL
}

impl<'a, KERNEL: kernels::Kernel> Forth<'a, KERNEL> {
    pub fn new() -> Self {
        let mut state = ForthState::new();
        let kernel = KERNEL::new(&mut state);
        let mut forth_machine = Self { state, kernel };

        let mut dummy_output = output_stream::DropOutputStream::new();
        for definition in operations::UNCOMPILED_OPERATIONS.iter() {
            let mut token_iterator = tokens::TokenStream::new(definition.chars());
            forth_machine.evaluate(&mut token_iterator, &mut dummy_output).unwrap_or_else(|error| panic!("Failed to parse preset definition: {:?} {:?}", definition, error));
        }

        forth_machine
    }

    pub fn evaluate_string<'b, O: output_stream::OutputStream + 'b> (&mut self, input: &'b str, output: &'b mut O) -> ForthResult {
        self.evaluate_stream(input.chars(), output)
    }

    pub fn evaluate_stream<'b, I: Iterator<Item = char> + 'b, O: output_stream::OutputStream + 'b>(&mut self, stream: I, output: &mut O) -> ForthResult {
        self.evaluate(&mut tokens::TokenStream::new(stream), output)
    }

    pub fn evaluate<'f, 't>(&'f mut self, input_stream: &mut tokens::TokenStream<'t>, output_stream: &mut dyn output_stream::OutputStream) -> ForthResult {    
        loop {
            match self.kernel.evaluate_chain(&mut self.state, ForthIO { input_stream, output_stream })
                    .and_then(|_| self.state.step(input_stream, output_stream)) 
                    .or_else(|error| self.kernel.handle_error_chain(&mut self.state, ForthIO { input_stream, output_stream }, error)) {
                Err(Error::TokenStreamEmpty) | Err(Error::Halt) => break,
                Err(error) => {
                    println!("error = {:?}", error);
                    return Err(error)
                },
                Ok(_) => ()
            }
        }
        Ok(())
    }
}

/**
 * This struct contains the state required to execute / emulate the code
 */
pub struct ForthState<'a> {
    pub definitions: definition::DefinitionSet,
    pub compiled_code: compiled_code::CompiledCodeSegment<'a>,
    // the return stack is not actually used as a return stack, but is still provided for other uses
    pub return_stack: stack::Stack,
    pub stack: stack::Stack,
    pub memory: memory::Memory,

    pub execution_mode: ExecutionMode,
    // pointer to the next instruction to execute
    pub instruction_pointer: Option<memory::Address>,
    // contains the current instruction, if any, being executed
    pub current_instruction: Option<definition::ExecutionToken>,
}

// split it up into some sort of ForthState vs. ForthMachine, so ForthMachine -> ForthState -> ForthEvaluator ...
impl<'a> ForthState<'a> {
    pub fn new() -> Self {
        Self {
            compiled_code: compiled_code::CompiledCodeSegment::new(),
            definitions: definition::DefinitionSet::new(),

            return_stack: stack::Stack::new(),
            stack: stack::Stack::new(),
            memory: memory::Memory::new(),

            execution_mode: ExecutionMode::Interpret,
            instruction_pointer: None,
            current_instruction: None,
        }.with_operations(operations::get_operations())
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

    fn get_evaluator<'f, 't, 'i, 'o>(&'f mut self, input_stream: &'i mut tokens::TokenStream<'t>, output_stream: &'o mut dyn output_stream::OutputStream) -> ForthEvaluator<'f, 'i, 'o, 't, '_, 'a> {
        ForthEvaluator {
            input_stream: input_stream,
            output_stream: output_stream,

            compiled_code: self.compiled_code.borrow(),

            definitions: &mut self.definitions,

            return_stack: &mut self.return_stack,
            stack: &mut self.stack,
            memory: &mut self.memory,

            execution_mode: &mut self.execution_mode,
            instruction_pointer: &mut self.instruction_pointer,
            current_instruction: &mut self.current_instruction
        }
    }

    fn step<'f, 't>(&'f mut self, input_stream: &mut tokens::TokenStream<'t>, output_stream: &mut dyn output_stream::OutputStream) -> ForthResult {
        let mut evaluator = self.get_evaluator(input_stream, output_stream);
        
        let result = evaluator.execute_next().and_then(|_|evaluator.load_next_instruction());

        let buffer = evaluator.compiled_code.buffer;
        self.compiled_code.restore(buffer);

        return result;
    }
}

pub struct ForthEvaluator<'f, 'i, 'o, 't, 'a, 'b> {
    pub input_stream: &'i mut tokens::TokenStream<'t>,
    pub output_stream: &'o mut dyn output_stream::OutputStream,

    pub compiled_code: compiled_code::CompilingCodeSegment<'a, 'b>,

    pub definitions: &'f mut definition::DefinitionSet,

    pub return_stack: &'f mut stack::Stack,
    pub stack: &'f mut stack::Stack,
    pub memory: &'f mut memory::Memory,

    pub execution_mode: &'f mut ExecutionMode,
    pub instruction_pointer: &'f mut Option<memory::Address>,
    pub current_instruction: &'f mut Option<definition::ExecutionToken>,
}

impl<'f, 'i, 'o, 't, 'a, 'b> ForthEvaluator<'f, 'i, 'o, 't, 'a, 'b> {
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
        Ok(())
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

    fn load_next_instruction(&mut self) -> ForthResult {
        self.instruction_pointer.ok_or(Error::InvalidAddress)
            .map(|instruction_pointer| {
                // load the current instruction
                *self.current_instruction = Some(self.memory.read(instruction_pointer));
        }).or_else(|_| self.input_stream.next().ok_or(Error::TokenStreamEmpty)
            .and_then(|token| self.definitions.get_from_token(token))
            .map(|definition| if *self.execution_mode == ExecutionMode::Compile && !definition.immediate {
                self.memory.push(definition.execution_token.value());
                *self.current_instruction = None;
            } else {
                *self.current_instruction = Some(definition.execution_token);
            })
        )
    }

    fn execute_next(&mut self) -> ForthResult {
        // increment the instruction pointer, execute the current instruction
        *self.instruction_pointer = self.instruction_pointer.map(|ip| ip.plus_cell(1));
        match self.current_instruction.take() {
            Some(xt) => self.execute(xt),
            None => Ok(())
        }
    }
}
