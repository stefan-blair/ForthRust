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

fn stringify_execution_token(debug_target: &evaluate::ForthState, xt: evaluate::definition::ExecutionToken) -> String {
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

fn read_length_string_at(debug_target: &evaluate::ForthState, mut address: environment::memory::Address) -> String {
    let length: environment::generic_numbers::UnsignedByte = debug_target.memory.read(address);
    let mut buffer = String::new();
    for _ in 0..length {
        address.increment();
        buffer.push(debug_target.memory.read::<environment::generic_numbers::UnsignedByte>(address) as char);
    }

    buffer
}

fn read_null_terminated_string(debug_target: &evaluate::ForthState, mut address: environment::memory::Address) -> String {
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

fn read_from_address(debug_target: &evaluate::ForthState, address: environment::memory::Address, format: &str) -> String {    
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

fn print_stack_formatted(debug_target: &evaluate::ForthState, values: &[environment::value::Value], io: evaluate::ForthIO) {
    for (i, value) in values.iter().enumerate() {
        match value {
            environment::value::Value::Number(number) => io.output_stream.writeln(&format!("{:#10x} | {}", i, number)),
            environment::value::Value::ExecutionToken(xt) => io.output_stream.writeln(&format!("{:#10x} | {}", i, stringify_execution_token(&debug_target, *xt)))
        }
    }
}

/**
 * Debug operations.
 */
pub(in super) fn view_stack(_: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState, io: evaluate::ForthIO) {
    print_stack_formatted(debug_target, debug_target.stack.debug_only_get_vec(), io)
}

pub(in super) fn view_return_stack(_: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState, io: evaluate::ForthIO) {
    print_stack_formatted(debug_target, debug_target.return_stack.debug_only_get_vec(), io)
}

pub(in super) fn view_memory(_: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState, io: evaluate::ForthIO) {
    for (i, value) in debug_target.memory.debug_only_get_vec().iter().enumerate() {
        let name = match debug_target.definitions.debug_only_get_name(evaluate::definition::ExecutionToken::DefinedOperation(debug_target.memory.address_from_cell(i as environment::generic_numbers::Number).unwrap())) {
            Some(name) => format!("\t\t\t: definition of {}", name),
            None => "".to_string()
        };
        match value {
            environment::value::Value::Number(number) => io.output_stream.writeln(&format!("{:#10x} | {} {}", i, number, name)),
            environment::value::Value::ExecutionToken(xt) => io.output_stream.writeln(&format!("{:#10x} | {} {}", i, stringify_execution_token(&debug_target, *xt), name))
        }
    }
}

pub(in super) fn examine_memory(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState, io: evaluate::ForthIO) {
    let address = debug_target.memory.address_from(debugger_state.forth.state.stack.pop().unwrap()).unwrap();
    let format = match io.input_stream.next() {
        Some(io::tokens::Token::Name(format)) => format,
        _ => return
    };
    io.output_stream.writeln(&read_from_address(debug_target, address, &format[..]));
}

pub(in super) type DebugOperation = fn(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState, io: evaluate::ForthIO);

pub(in super) const DEBUG_OPERATIONS: &[(&str, DebugOperation)] = &[
    ("STACK", view_stack),
    ("RETURNSTACK", view_return_stack),
    ("MEMORY", view_memory),
    ("X", examine_memory),
];