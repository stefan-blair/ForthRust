use std::collections::HashMap;

use crate::io;
use crate::evaluate::{self, definition, kernels};
use crate::environment::{memory};

use super::debug_operations;


pub struct DebugState<'a, 'i, 'o> {
    pub debugging: bool,
    pub stepping: bool,
    pub breakpoints: Vec<memory::Address>,
    pub current_error: Option<evaluate::Error>,
    pub forth: evaluate::Forth<'a, 'i, 'o, kernels::DefaultKernel>,
    pub debug_operations: HashMap<String, debug_operations::DebugOperation>,
}

impl <'a, 'i, 'o> DebugState<'a, 'i, 'o> {
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

    fn debug(&mut self, state: &mut evaluate::ForthState) {
        debug_operations::view_state(self, state);
        // await and execute debug commands
        self.debugging = true;
        while self.debugging {
            let result = match self.forth.evaluate() {
                Result::Err(evaluate::Error::UnknownWord(word)) => {
                    match self.debug_operations.get(&word).ok_or(evaluate::Error::UnknownWord(word)) {
                    Ok(op) => Ok(op(self, state)),
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

pub struct DebugKernel<'a, 'i, 'o, NK: kernels::Kernel> {
    debug_state: DebugState<'a, 'i, 'o>,
    next_kernel: NK
}

impl<'a, 'i, 'o, NK: kernels::Kernel> DebugKernel<'a, 'i, 'o, NK> {
    pub fn init_io<I: Iterator<Item = char> + 'i, O: io::output_stream::OutputStream + 'o>(&mut self, input: I, output: O) {
        self.debug_state.forth.set_output_stream(output);
        self.debug_state.forth.set_input_stream(input);
    }
}

impl<'a, 'i, 'o, NK: kernels::Kernel> kernels::Kernel for DebugKernel<'a, 'i, 'o, NK> {
    type NextKernel = NK;
    fn new(state: &mut evaluate::ForthState) -> Self {
        state.add_operations(vec![
            ("DEBUG", false, debug)
        ]);
            
        Self {
            debug_state: DebugState::new(),
            next_kernel: NK::new(state)
        }
    }

    fn get_next_kernel(&mut self) -> &mut Self::NextKernel { &mut self.next_kernel }

    fn evaluate(&mut self, state: &mut evaluate::ForthState) -> evaluate::ForthResult {
        let debug_state = &mut self.debug_state;
        let hit_breakpoint = state.instruction_pointer
            .and_then(|instruction_pointer| debug_state.breakpoints.iter().find(|addr| **addr == instruction_pointer).map(|addr| *addr));
        if let Some(breakpoint) = hit_breakpoint {
            debug_state.forth.state.output_stream.writeln("");
            debug_state.forth.state.output_stream.writeln("------------------------------------------------------");
            debug_state.forth.state.output_stream.writeln(&format!("Hit breakpoint at {:#x}", breakpoint.to_offset()));
            debug_state.debug(state);
        } else if debug_state.stepping {
            debug_state.forth.state.output_stream.writeln("");
            debug_state.forth.state.output_stream.writeln("------------------------------------------------------");
            debug_state.forth.state.output_stream.writeln("Stepped");
            self.debug_state.debug(state);
        } else if let Some(true) = state.current_instruction.map(|current_instruction| current_instruction == DEBUG_OPERATION_XT) {
            // checking if the current instruction being executed is the DEBUG operation.  hook, and start debugging from there
            debug_state.forth.state.output_stream.writeln("");
            debug_state.forth.state.output_stream.writeln("------------------------------------------------------");
            debug_state.forth.state.output_stream.writeln("Now debugging Forth process");
            self.debug_state.debug(state);
        }

        self.debug_state.current_error.take().map_or_else(|| Ok(()), |error| Err(error))
    }

    fn handle_error(&mut self, state: &mut evaluate::ForthState, error: evaluate::Error) -> evaluate::ForthResult { 
        match error {
            evaluate::Error::TokenStreamEmpty | evaluate::Error::Halt => return Err(error),
            _ => ()
        }

        self.debug_state.current_error = Some(error);
        self.debug_state.debug(state);
        match self.debug_state.current_error.take() {
            Some(error) => Err(error),
            None => Ok(())
        }
    }
}

const DEBUG_OPERATION_XT: definition::ExecutionToken = definition::ExecutionToken::LeafOperation(debug);
pub fn debug(_: &mut evaluate::ForthState) -> evaluate::ForthResult { Ok(()) }