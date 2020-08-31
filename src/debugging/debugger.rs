use std::collections::HashMap;

use crate::evaluate::{self, definition, kernels};
use crate::environment::{memory};

use super::debug_operations;


pub struct DebugState<'a> {
    pub stepping: bool,
    pub breakpoints: Vec<memory::Address>,
    pub forth: evaluate::Forth<'a, kernels::DefaultKernel>,
    pub debug_operations: HashMap<String, debug_operations::DebugOperation>,
}

impl <'a> DebugState<'a> {
    fn new() -> Self {
        Self {
            stepping: false,
            breakpoints: Vec::new(),
            forth: evaluate::Forth::<kernels::DefaultKernel>::new(),
            debug_operations: debug_operations::DEBUG_OPERATIONS.iter().map(|(s, o)| (s.to_string(), *o)).collect()
        }
    }

    fn debug(&mut self, state: &mut evaluate::ForthState, io: evaluate::ForthIO) {
        let input_stream = io.input_stream;
        let output_stream = io.output_stream;
        loop {
            let result = match self.forth.evaluate(input_stream, output_stream) {
                Result::Err(evaluate::Error::UnknownWord(name)) => match self.debug_operations.get(&name).ok_or(evaluate::Error::UnknownWord(name)) {
                    Ok(op) => Ok(op(self, state, evaluate::ForthIO { input_stream, output_stream })),
                    error => error.map(|_|())
                }
                result => result
            };

            if let Result::Err(_) = result {
                break;
            }
        }
    }    
}

pub struct DebugKernel<'a, NK: kernels::Kernel> {
    debug_state: DebugState<'a>,
    next_kernel: NK
}

impl<'a, NK: kernels::Kernel> kernels::Kernel for DebugKernel<'a, NK> {
    type NextKernel = NK;
    fn new() -> Self {
        Self {
            debug_state: DebugState::new(),
            next_kernel: NK::new()
        }
    }

    fn get_next_kernel(&mut self) -> &mut Self::NextKernel { &mut self.next_kernel }

    fn evaluate(&mut self, state: &mut evaluate::ForthState, io: evaluate::ForthIO) -> evaluate::ForthResult {
        let hit_breakpoint = state.instruction_pointer
            .and_then(|instruction_pointer| self.debug_state.breakpoints.iter().find(|addr| **addr == instruction_pointer));
        if let Some(breakpoint) = hit_breakpoint {
            io.output_stream.writeln(&format!("Hit breakpoint at {:#x}", breakpoint.to_offset()));
            self.debug_state.debug(state, io);
        } else if self.debug_state.stepping {
            io.output_stream.writeln("stepped");
            self.debug_state.debug(state, io);
        } else if let Some(true) = state.current_instruction.map(|current_instruction| current_instruction.to_offset() == DEBUG_OPERATION_XT.to_offset()) {
            // checking if the current instruction being executed is the DEBUG operation.  hook, and start debugging from there
            io.output_stream.writeln("Now debugging Forth process");
            self.debug_state.debug(state, io);
        }

        Ok(())
    }
}

const DEBUG_OPERATION_XT: definition::ExecutionToken = definition::ExecutionToken::Operation(debug);
pub fn debug(_: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { Ok(()) }