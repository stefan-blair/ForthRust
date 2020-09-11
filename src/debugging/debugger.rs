use std::collections::HashMap;

use crate::io;
use crate::evaluate::{self, definition, kernels};
use crate::environment::{memory};

use super::debug_operations;


pub struct DebugState<'a> {
    pub debugging: bool,
    pub stepping: bool,
    pub breakpoints: Vec<memory::Address>,
    pub current_error: Option<evaluate::Error>,
    pub forth: evaluate::Forth<'a, kernels::DefaultKernel>,
    pub debug_operations: HashMap<String, debug_operations::DebugOperation>,
}

impl <'a> DebugState<'a> {
    fn new() -> Self {
        Self {
            debugging: false,
            stepping: false,
            breakpoints: Vec::new(),
            current_error: None,
            forth: evaluate::Forth::<kernels::DefaultKernel>::new(),
            debug_operations: debug_operations::DEBUG_OPERATIONS.iter().map(|(s, o)| (s.to_string(), *o)).collect()
        }
    }

    fn debug(&mut self, state: &mut evaluate::ForthState, io: evaluate::ForthIO) {
        // initially display the state of execution
        let input_stream = io.input_stream;
        let output_stream = io.output_stream;
        debug_operations::view_state(self, state, evaluate::ForthIO { input_stream, output_stream });

        // await and execute debug commands
        self.debugging = true;
        while self.debugging {
            let result = match self.forth.evaluate(input_stream, output_stream) {
                Result::Err(evaluate::Error::UnknownWord(name)) => {
                    match self.debug_operations.get(&name).ok_or(evaluate::Error::UnknownWord(name)) {
                    Ok(op) => Ok(op(self, state, evaluate::ForthIO { input_stream, output_stream })),
                    error => error.map(|_|())
                }}
                result => {
                    result
                }
            };

            if let Result::Err(_) = result {
                break;
            }
        }
    }    
}

pub struct DebugKernel<'a, NK: kernels::Kernel> {
    debug_state: DebugState<'a>,
    input_stream: Option<io::tokens::TokenStream<'a>>,
    output_stream: Option<Box<dyn io::output_stream::OutputStream + 'a>>,
    next_kernel: NK
}

impl<'a, NK: kernels::Kernel> DebugKernel<'a, NK> {
    pub fn init_io<I: Iterator<Item = char> + 'a, O: io::output_stream::OutputStream + 'a>(&mut self, input: I, output: O) {
        self.input_stream = Some(io::tokens::TokenStream::new(input));
        self.output_stream = Some(Box::new(output));
    }
}

impl<'a, NK: kernels::Kernel> kernels::Kernel for DebugKernel<'a, NK> {
    type NextKernel = NK;
    fn new(state: &mut evaluate::ForthState) -> Self {
        state.add_operations(vec![
            ("DEBUG", false, debug)
        ]);
            
        Self {
            debug_state: DebugState::new(),
            input_stream: None, output_stream: None,
            next_kernel: NK::new(state)
        }
    }

    fn get_next_kernel(&mut self) -> &mut Self::NextKernel { &mut self.next_kernel }

    fn evaluate(&mut self, state: &mut evaluate::ForthState, _: evaluate::ForthIO) -> evaluate::ForthResult {
        let io = if let (Some(input), Some(output)) = (self.input_stream.as_mut(), self.output_stream.as_mut().map(|x| x.as_mut())) {
            evaluate::ForthIO::new(input, output)
        } else {
            return Ok(())
        };

        let debug_state = &mut self.debug_state;
        let hit_breakpoint = state.instruction_pointer
            // .map(|instruction_pointer| instruction_pointer.minus_cell(1))
            .and_then(|instruction_pointer| debug_state.breakpoints.iter().find(|addr| **addr == instruction_pointer));
        if let Some(breakpoint) = hit_breakpoint {
            io.output_stream.writeln("");
            io.output_stream.writeln("------------------------------------------------------");
            io.output_stream.writeln(&format!("Hit breakpoint at {:#x}", breakpoint.to_offset()));
            self.debug_state.debug(state, io);
        } else if self.debug_state.stepping {
            io.output_stream.writeln("");
            io.output_stream.writeln("------------------------------------------------------");
            io.output_stream.writeln("Stepped");
            self.debug_state.debug(state, io);
        } else if let Some(true) = state.current_instruction.map(|current_instruction| current_instruction == DEBUG_OPERATION_XT) {
            // checking if the current instruction being executed is the DEBUG operation.  hook, and start debugging from there
            io.output_stream.writeln("");
            io.output_stream.writeln("------------------------------------------------------");
            io.output_stream.writeln("Now debugging Forth process");
            self.debug_state.debug(state, io);
        }

        self.debug_state.current_error.take().map_or_else(|| Ok(()), |error| Err(error))
    }

    fn handle_error(&mut self, state: &mut evaluate::ForthState, _: evaluate::ForthIO, error: evaluate::Error) -> evaluate::ForthResult { 
        let io = if let (Some(input), Some(output)) = (self.input_stream.as_mut(), self.output_stream.as_mut().map(|x| x.as_mut())) {
            evaluate::ForthIO::new(input, output)
        } else {
            return Err(error)
        };

        match error {
            evaluate::Error::TokenStreamEmpty | evaluate::Error::Halt => return Err(error),
            _ => ()
        }

        self.debug_state.current_error = Some(error);
        self.debug_state.debug(state, io);
        match self.debug_state.current_error.take() {
            Some(error) => Err(error),
            None => Ok(())
        }
    }
}

const DEBUG_OPERATION_XT: definition::ExecutionToken = definition::ExecutionToken::Operation(debug);
pub fn debug(_: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { Ok(()) }