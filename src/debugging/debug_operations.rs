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
        let current_address = debug_target.memory.address_from_cell(i as environment::generic_numbers::Number).unwrap();
        let name = match debug_target.definitions.debug_only_get_name(evaluate::definition::ExecutionToken::DefinedOperation(current_address)) {
            Some(name) => format!("definition of {}", name),
            None => "".to_string()
        };
        let is_instruction_pointer = if Some(current_address) == debug_target.instruction_pointer {
            " ip -> "
        } else {
            "       "
        };

        match value {
            environment::value::Value::Number(number) => io.output_stream.writeln(&format!("{} | {} {:<30} {}", stringify_address(current_address), is_instruction_pointer, number, name)),
            environment::value::Value::ExecutionToken(xt) => io.output_stream.writeln(&format!("{} | {} {:<30} {}", stringify_address(current_address), is_instruction_pointer, stringify_execution_token(&debug_target, *xt), name))
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

pub(in super) fn view_state(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState, io: evaluate::ForthIO) {
    let instruction_pointer = debug_target.instruction_pointer.map(|addr| stringify_address(addr)).unwrap_or("(awaiting input)".to_string());
    let execution_mode = match debug_target.execution_mode {
        evaluate::ExecutionMode::Compile => "compiling",
        evaluate::ExecutionMode::Interpret => "interpreting",
    }.to_string();
    let current_instruction = debug_target.current_instruction.map(|xt| stringify_execution_token(debug_target, xt)).unwrap_or("(awaiting instruction)".to_string());

    if let Some(error) = &debugger_state.current_error {
        io.output_stream.writeln(&format!("ENCOUNTERED ERROR {:?}", error));
        io.output_stream.writeln("");
    }

    for (name, value) in &[("instruction pointer", instruction_pointer), ("execution mode", execution_mode), ("current instruction", current_instruction)] {
        io.output_stream.writeln(&format!("{:>10}: {}", name, value));
    }

    if debugger_state.breakpoints.len() > 0 {
        io.output_stream.write(&format!("\nbreakpoints:"));
        for address in debugger_state.breakpoints.iter() {
            io.output_stream.write(&format!(" {}", stringify_address(*address)));
        }
        io.output_stream.writeln("");
    }
}

pub(in super) fn add_break(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState, _: evaluate::ForthIO) {
    let address = debug_target.memory.address_from(debugger_state.forth.state.stack.pop().unwrap()).unwrap();
    debugger_state.breakpoints.push(address);
}

pub(in super) fn step(debugger_state: &mut debugger::DebugState, _: &mut evaluate::ForthState, _: evaluate::ForthIO) {
    debugger_state.stepping = true;
    debugger_state.debugging = false;
}

pub(in super) fn do_continue(debugger_state: &mut debugger::DebugState, _: &mut evaluate::ForthState, _: evaluate::ForthIO) {
    debugger_state.stepping = false;
    debugger_state.debugging = false;
}

pub(in super) type DebugOperation = fn(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState, io: evaluate::ForthIO);

pub(in super) const DEBUG_OPERATIONS: &[(&str, DebugOperation)] = &[
    ("STACK", view_stack),
    ("RETURNSTACK", view_return_stack),
    ("MEMORY", view_memory),
    ("X", examine_memory),
    ("STATE", view_state),
    ("SET_BREAK", add_break),
    ("STEP", step),
    ("CONTINUE", do_continue),
];