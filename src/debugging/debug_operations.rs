use crate::io;
use crate::evaluate;
use crate::environment;
use super::debugger;


/**
 * Helper functions
 */
fn stringify_address(addr: environment::memory::Address) -> String {
    format!("{:#x}", addr.to_offset())
}

fn stringify_execution_token(debug_target: &debugger::DebugTarget, xt: evaluate::definition::ExecutionToken) -> String {
    let name = match debug_target.definitions.debug_only_get_name(xt) {
        Some(name) => format!("{} ", name),
        None => "".to_string()
    };

    match xt {
        evaluate::definition::ExecutionToken::Number(n) => format!("push {}", n),
        evaluate::definition::ExecutionToken::DefinedOperation(addr) => format!("{}(defined call @ {})", name, stringify_address(addr)),
        evaluate::definition::ExecutionToken::CompiledOperation(_) => format!("{}(call compiled operation)", name),
        evaluate::definition::ExecutionToken::Operation(_) => format!("{}(builtin)", name),
    }
}

fn read_length_string_at(debug_target: debugger::DebugTarget, mut address: environment::memory::Address) -> String {
    let length: environment::generic_numbers::UnsignedByte = debug_target.memory.read(address);
    let mut buffer = String::new();
    for _ in 0..length {
        address.increment();
        buffer.push(debug_target.memory.read::<environment::generic_numbers::UnsignedByte>(address) as char);
    }

    buffer
}

fn read_null_terminated_string(debug_target: debugger::DebugTarget, mut address: environment::memory::Address) -> String {
    let mut buffer = String::new();
    loop {
        let byte: environment::generic_numbers::UnsignedByte = debug_target.memory.read(address);
        if byte == 0 {
            return buffer;
        } else {
            buffer.push(byte as char);
        }

        address.increment();
    }
}

fn read_from_address(debug_target: debugger::DebugTarget, address: environment::memory::Address, format: &str) -> String {
    // check if starts with H and then add the format in
    match format {
        "I" => stringify_execution_token(&debug_target, debug_target.memory.read(address)),
        "N" => format!("{}", debug_target.memory.read::<environment::generic_numbers::Number>(address)),
        "D" => format!("{}", debug_target.memory.read::<environment::generic_numbers::DoubleNumber>(address)),
        "B" => format!("{}", debug_target.memory.read::<environment::generic_numbers::Byte>(address)),
        "UN" => format!("{}", debug_target.memory.read::<environment::generic_numbers::UnsignedNumber>(address)),
        "UD" => format!("{}", debug_target.memory.read::<environment::generic_numbers::UnsignedDoubleNumber>(address)),
        "UB" => format!("{}", debug_target.memory.read::<environment::generic_numbers::UnsignedByte>(address)),
        "LS" => read_length_string_at(debug_target, address),
        "S" => read_null_terminated_string(debug_target, address),
        _ => "Unknown format specifier".to_string()
    }
}

fn print_stack_formatted(state: &mut evaluate::ForthEvaluator, debug_target: debugger::DebugTarget, values: &[environment::value::Value]) {
    for (i, value) in values.iter().enumerate() {
        match value {
            environment::value::Value::Number(number) => state.output_stream.writeln(&format!("{:#10x} | {}", i, number)),
            environment::value::Value::ExecutionToken(xt) => state.output_stream.writeln(&format!("{:#10x} | {}", i, stringify_execution_token(&debug_target, *xt)))
        }
    }
}

/**
 * Debug operations.
 */
pub(in super) fn view_stack(debugger_state: &mut evaluate::ForthEvaluator, debug_target: debugger::DebugTarget) {
    print_stack_formatted(debugger_state, debug_target, debug_target.stack.debug_only_get_vec())
}

pub(in super) fn view_return_stack(debugger_state: &mut evaluate::ForthEvaluator, debug_target: debugger::DebugTarget) {
    print_stack_formatted(debugger_state, debug_target, debug_target.return_stack.debug_only_get_vec())
}

pub(in super) fn view_memory(debugger_state: &mut evaluate::ForthEvaluator, debug_target: debugger::DebugTarget) {
    for (i, value) in debug_target.memory.debug_only_get_vec().iter().enumerate() {
        let name = match debug_target.definitions.debug_only_get_name(evaluate::definition::ExecutionToken::DefinedOperation(debug_target.memory.address_from_cell(i as environment::generic_numbers::Number).unwrap())) {
            Some(name) => format!("\t\t\t: definition of {}", name),
            None => "".to_string()
        };
        match value {
            environment::value::Value::Number(number) => debugger_state.output_stream.writeln(&format!("{:#10x} | {} {}", i, number, name)),
            environment::value::Value::ExecutionToken(xt) => debugger_state.output_stream.writeln(&format!("{:#10x} | {} {}", i, stringify_execution_token(&debug_target, *xt), name))
        }
    }
}

pub(in super) fn examine_memory(debugger_state: &mut evaluate::ForthEvaluator, debug_target: debugger::DebugTarget) {
    let address = debug_target.memory.address_from(debugger_state.stack.pop().unwrap()).unwrap();
    let format = match debugger_state.input_stream.next() {
        Some(io::tokens::Token::Name(format)) => format,
        _ => return
    };
    debugger_state.output_stream.writeln(&read_from_address(debug_target, address, &format[..]));
}

pub(in super) type DebugOperation = fn(debugger_state: &mut evaluate::ForthEvaluator, debug_target: debugger::DebugTarget);

pub(in super) const DEBUG_OPERATIONS: &[(&str, DebugOperation)] = &[
    ("STACK", view_stack),
    ("RETURNSTACK", view_return_stack),
    ("MEMORY", view_memory),
    ("X", examine_memory),
];