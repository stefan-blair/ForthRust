use crate::evaluate;
use crate::environment;
use super::debugger;
use crate::operations;


/**
 * Helper functions
 */
fn stringify_address(addr: environment::memory::Address) -> String {
    format!("{:#x}", addr.as_raw())
}

pub fn stringify_execution_token(debug_target: &evaluate::ForthState, xt: evaluate::definition::ExecutionToken) -> String {
    let word = match debug_target.definitions.debug_only_get_name(xt) {
        Some(word) => format!("{} ", word),
        None => "".to_string()
    };

    match xt {
        evaluate::definition::ExecutionToken::Number(n) => format!("push {}", n),
        evaluate::definition::ExecutionToken::Definition(addr) => format!("{}(defined call @ {})", word, stringify_address(addr)),
        evaluate::definition::ExecutionToken::CompiledInstruction(_) => format!("[{}]", debug_target.compiled_instructions.get(xt).to_string()),
        evaluate::definition::ExecutionToken::LeafOperation(_) => format!("{}(builtin)", word),
    }
}

fn read_length_string_at(debug_target: &evaluate::ForthState, mut address: environment::memory::Address) -> String {
    let length: environment::generic_numbers::UnsignedByte = debug_target.read(address).unwrap();
    let mut buffer = String::new();
    for _ in 0..length {
        address.increment();
        buffer.push(debug_target.read::<environment::generic_numbers::UnsignedByte>(address).unwrap() as char);
    }

    buffer
}

fn read_null_terminated_string(debug_target: &evaluate::ForthState, mut address: environment::memory::Address) -> String {
    let mut buffer = String::new();
    loop {
        let byte: environment::generic_numbers::UnsignedByte = debug_target.read(address).unwrap();
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
        "I" => stringify_execution_token(&debug_target, debug_target.read(address).unwrap()),
        "A" => format!("--> {}", read_from_address(debug_target, debug_target.read(address).unwrap(), format)),
        "N" => format!("{}", debug_target.read::<environment::generic_numbers::Number>(address).unwrap()),
        "D" => format!("{}", debug_target.read::<environment::generic_numbers::DoubleNumber>(address).unwrap()),
        "B" => format!("{}", debug_target.read::<environment::generic_numbers::Byte>(address).unwrap()),
        "UN" => format!("{}", debug_target.read::<environment::generic_numbers::UnsignedNumber>(address).unwrap()),
        "UD" => format!("{}", debug_target.read::<environment::generic_numbers::UnsignedDoubleNumber>(address).unwrap()),
        "UB" => format!("{}", debug_target.read::<environment::generic_numbers::UnsignedByte>(address).unwrap()),
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

fn get_variables<'b>(debug_target: &'b evaluate::ForthState) -> Vec<(&'b String, environment::memory::Address)> {
    debug_target.definitions.debug_only_get_nametag_map().iter()
        .map(|(word, nametag)| (word, debug_target.definitions.get(*nametag).execution_token))
        .filter_map(|(word, execution_token)| match execution_token { 
            evaluate::definition::ExecutionToken::Number(addr) => Some((word, environment::memory::Address::debug_only_from_offset(addr as environment::memory::Offset))),
            _ => None
        }).collect::<Vec<_>>()
}

fn print_memory_formatted(debug_target: &evaluate::ForthState, range: Option<(usize, usize)>, io: evaluate::ForthIO) {
    let memory = debug_target.heap.debug_only_get_vec();
    let variables = get_variables(debug_target);
    let (start, end) = range.unwrap_or((0, memory.len()));
    for (i, value) in memory.iter().enumerate().skip(start).take(end - start) {
        let current_address = environment::memory::Address::debug_only_from_cell(i as environment::memory::Offset);
        let word = match debug_target.definitions.debug_only_get_name(evaluate::definition::ExecutionToken::Definition(current_address)) {
            Some(word) => format!("definition of {}", word),
            None => match variables.iter().filter_map(|(word, addr)| if *addr == current_address {
                Some(word)
            } else {
                None
            }).next() {
                Some(word) => format!("memory of {}", word),
                None => "".to_string()
            }
        };
        let is_instruction_pointer = if Some(current_address) == debug_target.instruction_pointer {
            " ip -> "
        } else {
            "       "
        };

        match value {
            environment::value::Value::Number(number) => io.output_stream.writeln(&format!("{:<7} | {} {:<30} {}", stringify_address(current_address), is_instruction_pointer, number, word)),
            environment::value::Value::ExecutionToken(xt) => io.output_stream.writeln(&format!("{:<7} | {} {:<30} {}", stringify_address(current_address), is_instruction_pointer, stringify_execution_token(&debug_target, *xt), word))
        }
    }
}

/**
 * Debug operations.
 */
pub(in super) fn view_stack(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) {
    print_stack_formatted(debug_target, debug_target.stack.debug_only_get_vec(), debugger_state.forth.state.get_forth_io())
}

pub(in super) fn view_return_stack(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) {
    print_stack_formatted(debug_target, debug_target.return_stack.debug_only_get_vec(), debugger_state.forth.state.get_forth_io())
}

pub(in super) fn view_memory(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) {
    print_memory_formatted(debug_target, None, debugger_state.forth.state.get_forth_io());
}

pub(in super) fn examine_memory(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) {
    let address = debugger_state.forth.state.stack.pop().unwrap();
    let format = debugger_state.forth.state.input_stream.next_word().unwrap();
    debugger_state.forth.state.output_stream.writeln(&read_from_address(debug_target, address, &format[..]));
}

pub(in super) fn view_state(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) {
    let instruction_pointer = debug_target.instruction_pointer.map(|addr| stringify_address(addr)).unwrap_or("(awaiting input)".to_string());
    let execution_mode = match debug_target.execution_mode {
        evaluate::ExecutionMode::Compile => "compiling",
        evaluate::ExecutionMode::Interpret => "interpreting",
    }.to_string();
    let current_instruction = debug_target.current_instruction.map(|xt| stringify_execution_token(debug_target, xt)).unwrap_or("(awaiting instruction)".to_string());

    if let Some(error) = &debugger_state.current_error {
        debugger_state.forth.state.output_stream.writeln("######################################################");
        debugger_state.forth.state.output_stream.writeln(&format!("ENCOUNTERED ERROR {:?}", error));
        debugger_state.forth.state.output_stream.writeln("######################################################");
        debugger_state.forth.state.output_stream.writeln("");
    }

    debugger_state.forth.state.output_stream.writeln("------------------------------------------------------");
    if debugger_state.breakpoints.len() > 0 {
        debugger_state.forth.state.output_stream.write(&format!("\n{:>20}:", "breakpoints"));
        for address in debugger_state.breakpoints.iter() {
            debugger_state.forth.state.output_stream.write(&format!(" {}", stringify_address(*address)));
        }
        debugger_state.forth.state.output_stream.writeln("");
        debugger_state.forth.state.output_stream.writeln("------------------------------------------------------");
    }

    for (word, value) in &[("execution mode", execution_mode), ("current instruction", current_instruction), ("instruction pointer", instruction_pointer)] {
        debugger_state.forth.state.output_stream.writeln(&format!("{:>20}: {}", word, value));
    }
    
    debugger_state.forth.state.output_stream.writeln("------------------------------------------------------");
    if let Some(instruction_pointer) = debug_target.instruction_pointer {
        debugger_state.forth.state.output_stream.writeln("memory:\n");
        let start = if instruction_pointer.get_cell() < 3 {
            0
        } else {
            instruction_pointer.get_cell() - 3
        };
        let end = instruction_pointer.get_cell() + 4;
        print_memory_formatted(debug_target, Some((start, end)), debugger_state.forth.state.get_forth_io());
        debugger_state.forth.state.output_stream.writeln("------------------------------------------------------");
    }

    let stack_vec = debug_target.stack.debug_only_get_vec();
    debugger_state.forth.state.output_stream.writeln("stack:\n");
    print_stack_formatted(debug_target, &stack_vec[..std::cmp::min(10, stack_vec.len())], debugger_state.forth.state.get_forth_io());
    debugger_state.forth.state.output_stream.writeln("------------------------------------------------------");
}

pub(in super) fn all_commands(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) {
    for (word, nametag) in debug_target.definitions.debug_only_get_nametag_map().iter() {
        let definition = debug_target.definitions.get(*nametag);
        let immediate_string = if definition.immediate {
            "immediate"
        } else {
            ""
        };

        debugger_state.forth.state.output_stream.writeln(&format!("{:<15}: {:<10} {}", word, immediate_string, stringify_execution_token(debug_target, definition.execution_token)))
    }
}

pub(in super) fn add_break(debugger_state: &mut debugger::DebugState, _: &mut evaluate::ForthState) {
    let address = debugger_state.forth.state.stack.pop().unwrap();
    debugger_state.breakpoints.push(address);
    debugger_state.forth.state.output_stream.writeln(&format!("Set breakpoint @ {}", stringify_address(address)));
}

pub(in super) fn step(debugger_state: &mut debugger::DebugState, _: &mut evaluate::ForthState) {
    debugger_state.forth.state.output_stream.writeln("Stepping...");
    debugger_state.stepping = true;
    debugger_state.debugging = false;
    debugger_state.current_error = None;
}

pub(in super) fn do_continue(debugger_state: &mut debugger::DebugState, _: &mut evaluate::ForthState) {
    debugger_state.forth.state.output_stream.writeln("Continuing");
    debugger_state.stepping = false;
    debugger_state.debugging = false;
    debugger_state.current_error = None;
}

pub(in super) fn do_exit(debugger_state: &mut debugger::DebugState, _: &mut evaluate::ForthState) {
    debugger_state.forth.state.output_stream.writeln("Exiting...");
    debugger_state.stepping = false;
    debugger_state.debugging = false;
    debugger_state.current_error = debugger_state.current_error.take().or(Some(evaluate::Error::Halt));
}

pub(in super) fn see(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) {
    let definition = match debug_target.definitions.get_from_token(debugger_state.forth.state.input_stream.next().unwrap()) {
        Ok(definition) => definition,
        _ => return debugger_state.forth.state.output_stream.writeln("No definition found")
    };

    debugger_state.forth.state.output_stream.writeln(&stringify_execution_token(debug_target, definition.execution_token));
    if let evaluate::definition::ExecutionToken::Definition(address) = definition.execution_token {
        let mut end = address;
        while {
            end.increment_cell();
            let break_operation = evaluate::definition::ExecutionToken::LeafOperation(operations::control_flow_operations::control_flow_break);
            let current_operation = debug_target.read::<evaluate::definition::ExecutionToken>(end).unwrap();
            break_operation != current_operation
        } {}
        end.increment_cell();

        print_memory_formatted(debug_target, Some((address.get_cell(), end.get_cell())), debugger_state.forth.state.get_forth_io());
    }
}

pub(in super) type DebugOperation = fn(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState);

pub(in super) const DEBUG_OPERATIONS: &[(&str, DebugOperation)] = &[
    ("STACK", view_stack),
    ("RETURNSTACK", view_return_stack),
    ("MEMORY", view_memory),
    ("X", examine_memory),
    ("STATE", view_state),
    ("ALL_COMMANDS", all_commands),
    ("SET_BREAK", add_break),
    ("STEP", step),
    ("CONTINUE", do_continue),
    ("EXIT", do_exit),
    ("SEE", see),
];