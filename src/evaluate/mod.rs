pub mod definition;
pub mod kernels;
pub mod config;

use crate::operations;
use crate::environment::{memory::{self, MemorySegment}, stack, heap, value::{self, ValueVariant}, units::{Bytes, Cells, Pages}};
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
    InvalidSize,
    InsufficientMemory,
    
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
    pub kernel: KERNEL,
}

/**
 * Provides a default method to construct the Forth object
 * using the default constructor.
 */
impl Forth<'_, '_, '_, kernels::DefaultKernel> {
    pub fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<'a, 'i, 'o, KERNEL: kernels::Kernel> Forth<'a, 'i, 'o, KERNEL> {
    pub fn new(config: config::ForthConfig) -> Self {
        let mut state = ForthState::new(config);
        let kernel = KERNEL::new(&mut state);
        let mut forth = Self { state, kernel };

        for definition in operations::UNCOMPILED_OPERATIONS.iter() {
            forth.evaluate_string(definition).unwrap_or_else(|error| panic!("Failed to parse preset definition: {:?} {:?}", definition, error));
        }

        forth
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
    pub output_stream: Box<dyn output_stream::OutputStream + 'o>,
    pub input_stream: tokens::TokenStream<'i>,

    // keeps track of different memory segments, where they are mapped to (virtually), and their permissions
    memory_map: memory::MemoryMap,
    // different fields of the state can be accessed by the running program as memory
    internal_state_memory: InternalStateMemory,
    // a vector of unnamed anonymous pages
    anonymous_pages: Vec<memory::Memory>,
    // the address of the base of the next anonymous page
    next_anonymous_mapping: memory::Address,
    // named memory segments
    pub return_stack: stack::Stack,
    pub stack: stack::Stack,
    pub data_space: memory::Memory,
    pub pad: memory::Memory,
    pub heap: heap::Heap,

    execution_mode: ExecutionMode,
    // pointer to the next instruction to execute
    instruction_pointer: Option<memory::Address>,
    // contains the current instruction, if any, being executed
    current_instruction: Option<definition::ExecutionToken>,
    pub definitions: definition::DefinitionTable,
    pub compiled_instructions: compiled_instructions::CompiledInstructions<'a>,

    config: config::ForthConfig
}

impl<'a, 'i, 'o> ForthState<'a, 'i, 'o> {
    // initialization
    pub fn new(config: config::ForthConfig) -> Self {
        let return_stack = stack::Stack::new(config.return_stack_addr);
        let stack = stack::Stack::new(config.stack_addr);
        let data_space = memory::Memory::new(config.data_space_addr);
        let pad = memory::Memory::new(config.pad_addr);
        let heap = heap::Heap::new(config.heap_addr);

        let internal_state_memory = InternalStateMemory::new(config.internal_state_memory_addr);

        let memory_map = memory::MemoryMap::new(vec![
            memory::MemoryMapping::special(data_space.get_base(), memory::MemoryPermissions::all(), |state| &state.data_space, |state| &mut state.data_space).with_name("data_space"),
            memory::MemoryMapping::special(stack.get_base(), memory::MemoryPermissions::readwrite(), |state| &state.stack, |state| &mut state.stack).with_name("stack"),
            memory::MemoryMapping::special(return_stack.get_base(), memory::MemoryPermissions::readwrite(), |state| &state.return_stack, |state| &mut state.return_stack).with_name("return_stack"),
            memory::MemoryMapping::special(pad.get_base(), memory::MemoryPermissions::readwrite(), |state| &state.pad, |state| &mut state.pad).with_name("pad"),
            memory::MemoryMapping::special(heap.get_base(), memory::MemoryPermissions::readwrite(), |state| &state.heap, |state| &mut state.heap).with_name("heap"),
            memory::MemoryMapping::special(internal_state_memory.get_base(), memory::MemoryPermissions::readonly(), |state| state, |state| state).with_name("[internal mappings]"),
        ]);

        Self {
            compiled_instructions: compiled_instructions::CompiledInstructions::new(),
            definitions: definition::DefinitionTable::new(),

            data_space, stack, return_stack, pad, heap, memory_map, internal_state_memory, 
            anonymous_pages: Vec::new(),
            next_anonymous_mapping: memory::Address::from_raw(Bytes::bytes(config.anonymous_mappings_addr)),

            execution_mode: ExecutionMode::Interpret,
            instruction_pointer: None,
            current_instruction: None,

            output_stream: Box::new(output_stream::DropOutputStream::new()),
            input_stream: tokens::TokenStream::empty(),

            config
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
    
    // getters
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

    pub fn config(&self) -> &config::ForthConfig {
        &self.config
    }

    // memory operations
    fn get_memory_segment<'x>(&'x self, mapping: memory::MemoryMapping) -> Result<&'x dyn MemorySegment, Error> {
        match mapping.mapping_type {
            memory::MappingType::Empty => Err(Error::InvalidAddress),
            memory::MappingType::Special { getter, .. } => Ok(getter(self)),
            memory::MappingType::Anonymous { index: i } => Ok(&self.anonymous_pages[i])
        }
    }

    fn get_mut_memory_segment<'x>(&'x mut self, mapping: memory::MemoryMapping) -> Result<&'x mut dyn MemorySegment, Error> {
        match mapping.mapping_type {
            memory::MappingType::Empty => Err(Error::InvalidAddress),
            memory::MappingType::Special { mutable_getter, .. } => Ok(mutable_getter(self)),
            memory::MappingType::Anonymous { index: i } => Ok(&mut self.anonymous_pages[i])
        }
    }

    pub fn check_address(&self, address: memory::Address) -> ForthResult {
        self.get_memory_segment(self.memory_map.get(address)?)?.check_address(address)
    }

    pub fn write<T: value::ValueVariant>(&mut self, address: memory::Address, value: T) -> Result<(), Error> {
        let entry = self.memory_map.get(address)?;
        if entry.permissions.write {
            value.write_to_memory(self.get_mut_memory_segment(entry)?, address)
        } else {
            Err(Error::InsufficientPermissions)
        }
    }

    pub fn read<T: value::ValueVariant>(&self, address: memory::Address) -> Result<T, Error> {
        let entry = self.memory_map.get(address)?;
        if entry.permissions.read {
            T::read_from_memory(self.get_memory_segment(entry)?, address)
        } else {
            Err(Error::InsufficientPermissions)
        }
    }

    fn read_instruction_pointer(&self) -> Result<definition::ExecutionToken, Error> {
        let address = self.instruction_pointer.ok_or(Error::InvalidAddress)?;
        let entry = self.memory_map.get(address)?;
        if entry.permissions.execute {
            definition::ExecutionToken::read_from_memory(self.get_memory_segment(entry)?, address)
        } else {
            Err(Error::InsufficientPermissions)
        }
    }

    pub fn create_anonymous_mapping(&mut self, num_pages: Pages) -> Result<memory::Address, Error> {
        let base = self.next_anonymous_mapping;
        self.next_anonymous_mapping.add(num_pages.to_bytes());

        self.create_anonymous_mapping_at(base, num_pages)
    }

    pub fn create_anonymous_mapping_at(&mut self, address: memory::Address, num_pages: Pages) -> Result<memory::Address, Error> {
        let index = self.anonymous_pages.len();

        self.anonymous_pages
            .push(memory::Memory::new(address.as_raw())
            .with_num_cells(num_pages.to_cells()));
        self.memory_map.add(memory::MemoryMapping::anonymous(address, memory::MemoryPermissions::readwrite(), index))
            .map(|_| address)
    }

    // execution instructions
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
        self.return_stack.push_frame();

        Ok(())
    }

    pub fn return_from(&mut self) -> ForthResult {
        self.return_stack.pop_frame()?;
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
                self.data_space.push(definition.execution_token.value());
                self.current_instruction = None;
            } else {
                self.current_instruction = Some(definition.execution_token);
            })
        )
    }

    fn execute_current_instruction(&mut self) -> ForthResult {
        // execute the current instruction, 'take'ing it so its None, and incrementing the current instruction pointer to the next position for the next iteration 
        self.instruction_pointer = self.instruction_pointer.map(|ip| ip.plus_cell(Cells::one()));
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
                Self { base: memory::Address::from_raw(Bytes::bytes(base)), members: Vec::new() }
            }

            fn add(&mut self, read: fn(&ForthState) -> value::Value, write: fn(&mut ForthState, value::Value)) -> StateRegister {
                let new = StateRegister::new(self.base.plus_cell(Cells::cells(self.members.len())), read, write);
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
            base: memory::Address::from_raw(Bytes::bytes(base)),
            execution_mode,
            members: builder.members
        }
    }

    fn len(&self) -> Cells {
         Cells::cells(self.members.len())
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
        self.internal_state_memory.base.plus_cell(self.internal_state_memory.len())
    }

    fn write_value(&mut self, address: memory::Address, value: value::Value) -> Result<(), Error> {
        self.check_address(address)?;
        let index = address.offset_from(self.get_base()).to_cells().get_cells();
        let write_function = self.internal_state_memory.members[index].write;
        Ok(write_function(self, value))
    }

    fn read_value(&self, address: memory::Address) -> Result<value::Value, Error> {
        self.check_address(address)?;
        let index = address.offset_from(self.get_base()).to_cells().get_cells();
        let read_function = self.internal_state_memory.members[index].read;
        Ok(read_function(self))
    }
}
