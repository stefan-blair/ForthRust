pub mod definition;
pub mod kernels;

use crate::operations;
use crate::environment::{memory::{self, MemorySegment}, stack, value::{self, ValueVariant}};
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
    InsufficientPermissions,
    NoMoreTokens,
    Exception(u64),
    
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

pub struct ForthIO<'a, 'i, 'o> {
    pub input_stream: &'a mut tokens::TokenStream<'i>,
    pub output_stream: &'a mut (dyn output_stream::OutputStream + 'o)
}

impl<'a, 'i, 'o> ForthIO<'a, 'i, 'o> {
    pub fn new(input_stream: &'a mut tokens::TokenStream<'i>, output_stream: &'a mut (dyn output_stream::OutputStream + 'o)) -> Self {
        Self { input_stream, output_stream }
    }

    pub fn borrow<'b>(&'b mut self) -> ForthIO<'b, 'i, 'o> {
        ForthIO { input_stream: self.input_stream, output_stream: self.output_stream }
    }
}

pub struct Forth<'a, 'i, 'o, KERNEL: kernels::Kernel> {
    pub state: ForthState<'a, 'i, 'o>,
    pub kernel: KERNEL
}

impl<'a, 'i, 'o, KERNEL: kernels::Kernel> Forth<'a, 'i, 'o, KERNEL> {
    pub fn new() -> Self {
        let mut state = ForthState::new();
        let kernel = KERNEL::new(&mut state);
        let mut forth_machine = Self { state, kernel };

        for definition in operations::UNCOMPILED_OPERATIONS.iter() {
            forth_machine.evaluate_string(definition).unwrap_or_else(|error| panic!("Failed to parse preset definition: {:?} {:?}", definition, error));
        }

        forth_machine
    }

    pub fn set_output_stream<O: output_stream::OutputStream + 'o> (&mut self, output: O) {
        self.state.output_stream = Box::new(output)
    }

    pub fn with_output_stream<O: output_stream::OutputStream + 'o> (mut self, output: O) -> Self {
        self.state.output_stream = Box::new(output);
        self
    }

    pub fn set_input_string(&mut self, input: &'i str) {
        self.set_input_stream(input.chars())
    }

    pub fn with_input_string(mut self, input: &'i str) -> Self {
        self.set_input_string(input);
        self
    }

    pub fn with_input_stream<I: Iterator<Item = char> + 'i>(mut self, stream: I) -> Self {
        self.set_input_stream(stream);
        self
    }

    pub fn set_input_stream<I: Iterator<Item = char> + 'i>(&mut self, stream: I) {
        self.state.input_stream = tokens::TokenStream::new(stream);
    }

    pub fn evaluate_string(&mut self, input: &'i str) -> ForthResult {
        self.set_input_string(input);
        self.evaluate()
    }

    pub fn evaluate_stream<I: Iterator<Item = char> + 'i>(&mut self, stream: I) -> ForthResult {
        self.set_input_stream(stream);
        self.evaluate()
    }
    
    pub fn evaluate(&mut self) -> ForthResult {    
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
pub struct ForthState<'a, 'i, 'o> {
    pub definitions: definition::DefinitionSet,
    pub compiled_instructions: compiled_instructions::CompiledInstructions<'a>,
    // the return stack is not actually used as a return stack, but is still provided for other uses
    pub return_stack: stack::Stack,
    pub stack: stack::Stack,
    pub heap: memory::Memory,
    pub pad: memory::Memory,

    // keeps track of different memory segments, where they are mapped to (virtually), and their permissions
    memory_map: memory::MemoryMap,
    // different fields of the state can be accessed by the running program as memory
    internal_state_memory: InternalStateMemory,

    execution_mode: ExecutionMode,
    // pointer to the next instruction to execute
    instruction_pointer: Option<memory::Address>,
    // contains the current instruction, if any, being executed
    current_instruction: Option<definition::ExecutionToken>,

    pub output_stream: Box<dyn output_stream::OutputStream + 'o>,
    pub input_stream: tokens::TokenStream<'i>
}

impl<'a, 'i, 'o> ForthState<'a, 'i, 'o> {
    pub fn new() -> Self {
        let return_stack = stack::Stack::new(0x56cadeace000);
        let stack = stack::Stack::new(0x7aceddead000);
        let heap = memory::Memory::new(0x7feaddead000);
        let pad = memory::Memory::new(0x76beaded5000);

        let internal_state_memory = InternalStateMemory::new(0x5deadbeef000);

        let memory_map = memory::MemoryMap::new(vec![
            memory::MemoryMapping::new(heap.get_base(), memory::MemoryPermissions::all(), |state| &state.heap, |state| &mut state.heap, "heap"),
            memory::MemoryMapping::new(stack.get_base(), memory::MemoryPermissions::readwrite(), |state| &state.stack, |state| &mut state.stack, "stack"),
            memory::MemoryMapping::new(return_stack.get_base(), memory::MemoryPermissions::readwrite(), |state| &state.return_stack, |state| &mut state.return_stack, "return stack"),
            memory::MemoryMapping::new(pad.get_base(), memory::MemoryPermissions::readwrite(), |state| &state.pad, |state| &mut state.pad, "pad"),
            memory::MemoryMapping::new(internal_state_memory.get_base(), memory::MemoryPermissions::readonly(), |state| state, |state| state, "[internal mappings]"),
        ]);

        Self {
            compiled_instructions: compiled_instructions::CompiledInstructions::new(),
            definitions: definition::DefinitionSet::new(),

            heap, stack, return_stack, pad,  memory_map, internal_state_memory,

            execution_mode: ExecutionMode::Interpret,
            instruction_pointer: None,
            current_instruction: None,

            output_stream: Box::new(output_stream::DropOutputStream::new()),
            input_stream: tokens::TokenStream::empty(),
        }.with_operations(operations::get_operations())
    }

    pub fn get_forth_io<'b>(&'b mut self) -> ForthIO<'b, 'i, 'o> {
        ForthIO::new(&mut self.input_stream, self.output_stream.as_mut())
    }

    pub fn memory_map<'x>(&'x self) -> &'x memory::MemoryMap {
        &self.memory_map
    }

    pub fn internal_state_memory<'x>(&'x self) -> &'x InternalStateMemory {
        &self.internal_state_memory
    }

    pub fn execution_mode(&self) -> ExecutionMode {
        self.execution_mode
    }

    pub fn instruction_pointer(&self) -> Option<memory::Address> {
        self.instruction_pointer
    }

    pub fn current_instruction(&self) -> Option<definition::ExecutionToken> {
        self.current_instruction
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

    pub fn check_address(&self, address: memory::Address) -> ForthResult {
        let entry = self.memory_map.get(address)?;
        let getter = entry.getter;
        getter(self).check_address(address)
    }

    pub fn write<T: value::ValueVariant>(&mut self, address: memory::Address, value: T) -> Result<(), Error> {
        let entry = self.memory_map.get(address)?;
        if entry.permissions.write {
            let mutable_getter = entry.mutable_getter;
            value.write_to_memory(mutable_getter(self), address)
        } else {
            Err(Error::InsufficientPermissions)
        }
    }

    pub fn read<T: value::ValueVariant>(&self, address: memory::Address) -> Result<T, Error> {
        let entry = self.memory_map.get(address)?;
        if entry.permissions.read {
            let getter = entry.getter;
            T::read_from_memory(getter(self), address)
        } else {
            Err(Error::InsufficientPermissions)
        }
    }

    fn read_instruction_pointer(&self) -> Result<definition::ExecutionToken, Error> {
        let address = self.instruction_pointer.ok_or(Error::InvalidAddress)?;
        let entry = self.memory_map.get(address)?;
        if entry.permissions.execute {
            let getter = entry.getter;
            definition::ExecutionToken::read_from_memory(getter(self), address)
        } else {
            Err(Error::InsufficientPermissions)
        }
    }

    pub fn execute(&mut self, execution_token: definition::ExecutionToken) -> ForthResult {
        match execution_token {
            definition::ExecutionToken::Definition(address) => self.call(address),
            definition::ExecutionToken::LeafOperation(fptr) => fptr(self),
            definition::ExecutionToken::CompiledInstruction(_) => self.compiled_instructions.get(execution_token).execute(self),
            definition::ExecutionToken::Number(i) => Ok(self.stack.push(i))
        }
    }

    pub fn call(&mut self, address: memory::Address) -> ForthResult {
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
        self.read_instruction_pointer().map(|current_instruction| self.current_instruction = Some(current_instruction))
            .or_else(|_| self.input_stream.next().ok().ok_or(Error::TokenStreamEmpty)
            .and_then(|token| self.definitions.get_from_token(token))
            .map(|definition| if self.execution_mode == ExecutionMode::Compile && !definition.immediate {
                self.heap.push(definition.execution_token.value());
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

#[derive(Clone, Copy)]
pub struct StateRegister {
    pub address: memory::Address,
    read: fn(&ForthState) -> value::Value,
    write: fn(&mut ForthState, value::Value),
}

impl StateRegister {
    fn new(address: memory::Address, read: fn(&ForthState) -> value::Value, write: fn(&mut ForthState, value::Value)) -> Self {
        Self { address, read, write }
    }
}

pub struct InternalStateMemory {
    pub base: memory::Address,
    // each state register can be accessed as both a member of the internal state memory, 
    pub execution_mode: StateRegister,
    members: Vec<StateRegister>
}

impl InternalStateMemory {
    fn new(base: usize) -> Self {
        // a helper structure for building the internal state memory
        struct Builder {
            base: memory::Address,
            members: Vec<StateRegister>
        };

        impl Builder {
            fn new(base: usize) -> Self {
                Self { base: memory::Address::from_raw(base), members: Vec::new() }
            }

            fn add(&mut self, read: fn(&ForthState) -> value::Value, write: fn(&mut ForthState, value::Value)) -> StateRegister {
                let new = StateRegister::new(self.base.plus_cell(self.members.len()), read, write);
                self.members.push(new);
                new
            }
        }
        
        let mut builder = Builder::new(base);
        let execution_mode = builder.add(
            |state| value::Value::Number(match state.execution_mode {
                ExecutionMode::Compile => 1,
                ExecutionMode::Interpret => 0
            }),
            |state, value| state.execution_mode = match value.to_number() {
                0 => ExecutionMode::Interpret,
                _ => ExecutionMode::Compile 
            }
        );

        Self { 
            base: memory::Address::from_raw(base),
            execution_mode,
            members: builder.members
        }
    }
    
    fn get_base(&self) -> memory::Address {
        self.base
    }
}

impl memory::MemorySegment for ForthState<'_, '_, '_> {
    fn get_base(&self) -> memory::Address {
        self.internal_state_memory.base
    }

    fn get_end(&self) -> memory::Address {
        self.internal_state_memory.base.plus_cell(self.internal_state_memory.members.len())
    }

    fn check_address(&self, address: memory::Address) -> Result<(), Error> {
        if address.between(self.get_base(), self.get_end()){
            Ok(())
        } else {
            Err(Error::InvalidAddress)
        }
    }

    fn write_value(&mut self, address: memory::Address, value: value::Value) -> Result<(), Error> {
        self.check_address(address)?;
        let write_function = self.internal_state_memory.members[address.cell_offset_from(self.get_base())].write;
        Ok(write_function(self, value))
    }

    fn read_value(&self, address: memory::Address) -> Result<value::Value, Error> {
        self.check_address(address)?;
        let read_function = self.internal_state_memory.members[address.cell_offset_from(self.get_base())].read;
        Ok(read_function(self))
    }
}
