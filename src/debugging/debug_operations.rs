use crate::evaluate;
use crate::environment;
use super::debugger;


fn stringify_address(addr: environment::memory::Address) -> String {
    format!("{:#x}", addr.to_offset())
}

fn stringify_execution_token(state: &evaluate::ForthEvaluator, xt: evaluate::definition::ExecutionToken) -> String {
    let name = match state.definitions.debug_only_get_name(xt) {
        Some(name) => format!("{} ", name),
        None => "".to_string()
    };

    match xt {
        evaluate::definition::ExecutionToken::Number(n) => format!("push {}", n),
        evaluate::definition::ExecutionToken::DefinedOperation(addr) => format!("{}(defined call @ {})", name, stringify_address(addr)),
        evaluate::definition::ExecutionToken::CompiledOperation(offset) => format!("{}(call compiled operation)", name),
        evaluate::definition::ExecutionToken::Operation(_) => format!("{}(builtin operation)", name),
    }
}

pub(in super) fn print_values_formatted(state: &mut evaluate::ForthEvaluator, values: &[environment::value::Value]) {
    for (i, value) in values.iter().enumerate() {
        match value {
            environment::value::Value::Number(number) => state.output_stream.writeln(&format!("{:#10x} | {}", i, number)),
            environment::value::Value::ExecutionToken(xt) => state.output_stream.writeln(&format!("{:#10x} | {}", i, stringify_execution_token(state, *xt)))
        }
    }
}

pub(in super) fn view_stack(debugger_state: &mut evaluate::ForthEvaluator, debug_target: debugger::DebugTarget) {
    print_values_formatted(debugger_state, debug_target.stack.debug_only_get_vec())
}

pub(in super) fn view_return_stack(debugger_state: &mut evaluate::ForthEvaluator, debug_target: debugger::DebugTarget) {
    print_values_formatted(debugger_state, debug_target.return_stack.debug_only_get_vec())
}

pub(in super) fn view_memory(debugger_state: &mut evaluate::ForthEvaluator, debug_target: debugger::DebugTarget) {
    print_values_formatted(debugger_state, debug_target.memory.debug_only_get_vec())
}

pub(in super) type DebugOperation = fn(debugger_state: &mut evaluate::ForthEvaluator, debug_target: debugger::DebugTarget);

pub(in super) const DEBUG_OPERATIONS: &[(&str, DebugOperation)] = &[
    ("STACK", view_stack),
    ("RETURNSTACK", view_return_stack),
    ("MEMORY", view_memory),
];