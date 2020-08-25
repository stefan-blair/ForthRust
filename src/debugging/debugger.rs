use std::mem;

use crate::io::{self, output_stream::OutputStream};
use crate::evaluate::{self, compiled_code, definition};
use crate::environment::{stack, memory};

use super::debug_operations;

/**
 * The debugger itself is implemented in forth!  its a separate forth interpreter, but it has some additional keywords.
 */

struct DebuggerClosure<'a> {
    debug_target: &'a DebugTarget<'a>,
    operation: debug_operations::DebugOperation
}

impl<'a> DebuggerClosure<'a> {
    fn new(debug_target: &'a DebugTarget<'a>, operation: debug_operations::DebugOperation) -> Self {
        Self { debug_target, operation }
    }

    fn call(&self, debug_state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
        let operation = self.operation;
        operation(debug_state, *self.debug_target);
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(super) struct DebugTarget<'a> {
    pub definitions: &'a definition::DefinitionSet,

    pub return_stack: &'a stack::Stack,
    pub stack: &'a stack::Stack,
    pub memory: &'a memory::Memory,

    pub instruction_pointer: &'a Option<memory::Address>,
    pub execution_mode: &'a evaluate::ExecutionMode,
}

pub fn debug(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let debug_target = DebugTarget {
        definitions: state.definitions,
        return_stack: state.return_stack,
        stack: state.stack,
        memory: state.memory,
        instruction_pointer: state.instruction_pointer,
        execution_mode: state.execution_mode
    };

    // add an operation to exit the debugger
    let mut debug_state = evaluate::ForthState::new()
        .with_operations(vec![("END", false, |_| Err(evaluate::Error::TokenStreamEmpty))]);

    // add in the remainder of the debug operations
    for (name, operation) in debug_operations::DEBUG_OPERATIONS.iter() {
        let xt = debug_state.compiled_code.add(Box::new(move |state| Ok(operation(state, debug_target))));
        debug_state.definitions.add(name.to_string(), evaluate::definition::Definition::new(xt, false));
    }

    state.output_stream.writeln("Debugging.  Use the END command to resume execution.");
    // borrow the io streams from the debugged state and run the debugger
    let result = debug_state.evaluate(state.input_stream, state.output_stream);

    result
}
