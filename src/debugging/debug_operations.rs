use crate::evaluate;
use crate::environment::{memory::{self, MemorySegment}, generic_numbers, value};
use super::debugger;
use crate::operations;


/**
 * Helper functions
 */
fn stringify_address(addr: memory::Address) -> String {
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

fn read_length_string_at(debug_target: &evaluate::ForthState, mut address: memory::Address) -> Result<String, evaluate::Error> {
    let length: generic_numbers::UnsignedByte = debug_target.read(address)?;
    let mut buffer = String::new();
    for _ in 0..length {
        address.increment();
        buffer.push(debug_target.read::<generic_numbers::UnsignedByte>(address)? as char);
    }

    Ok(buffer)
}

fn read_null_terminated_string(debug_target: &evaluate::ForthState, mut address: memory::Address) -> Result<String, evaluate::Error> {
    let mut buffer = String::new();
    loop {
        let byte: generic_numbers::UnsignedByte = debug_target.read(address)?;
        if byte == 0 {
            return Ok(buffer);
        } else {
            buffer.push(byte as char);
        }

        address.increment();
    }
}

fn read_from_address(debug_target: &evaluate::ForthState, address: memory::Address, format: &str) -> Result<String, evaluate::Error> {    
    Ok(match format {
        "I" => stringify_execution_token(&debug_target, debug_target.read(address)?),
        "A" => format!("--> {}", read_from_address(debug_target, debug_target.read(address)?, format)?),
        "N" => format!("{}", debug_target.read::<generic_numbers::Number>(address)?),
        "D" => format!("{}", debug_target.read::<generic_numbers::DoubleNumber>(address)?),
        "B" => format!("{}", debug_target.read::<generic_numbers::Byte>(address)?),
        "UN" => format!("{}", debug_target.read::<generic_numbers::UnsignedNumber>(address)?),
        "UD" => format!("{}", debug_target.read::<generic_numbers::UnsignedDoubleNumber>(address)?),
        "UB" => format!("{}", debug_target.read::<generic_numbers::UnsignedByte>(address)?),
        "LS" => read_length_string_at(debug_target, address)?,
        "S" => read_null_terminated_string(debug_target, address)?,
        _ => "Unknown format specifier".to_string()
    })
}

fn get_variables<'b>(debug_target: &'b evaluate::ForthState) -> Vec<(&'b String, memory::Address)> {
    debug_target.definitions.debug_only_get_nametag_map().iter()
        .map(|(word, nametag)| (word, debug_target.definitions.get(*nametag).execution_token))
        .filter_map(|(word, execution_token)| match execution_token { 
            evaluate::definition::ExecutionToken::Number(addr) => Some((word, memory::Address::debug_only_from_offset(addr as usize))),
            _ => None
        }).collect::<Vec<_>>()
}

fn print_address(debug_target: &evaluate::ForthState, address: memory::Address) -> String {
    let value = if let Ok(value) = debug_target.read::<value::Value>(address) {
        value
    } else {
        return "[Invalid Address]".to_string()
    };

    let variables = get_variables(debug_target);

    let is_instruction_pointer = if Some(address) == debug_target.instruction_pointer() {
        " [ ip ] "
    } else {
        "        "
    };

    let is_word_definition = match debug_target.definitions.debug_only_get_name(evaluate::definition::ExecutionToken::Definition(address)) {
        Some(word) => format!("definition of {}", word),
        None => match variables.iter().filter_map(|(word, addr)| if *addr == address {
            Some(word)
        } else {
            None
        }).next() {
            Some(word) => format!("memory of {}", word),
            None => "".to_string()
        }
    };

    format!("{}{:<30}{:<20}", is_instruction_pointer, print_value_helper(debug_target, value, 0, 4), is_word_definition)
}

fn print_value_helper(debug_target: &evaluate::ForthState, value: value::Value, depth: usize, max: usize) -> String {
    match value {
        value::Value::Number(number) => if let Ok(value) = debug_target.read::<value::Value>(memory::Address::from_raw(number as usize)) {
            let address = memory::Address::from_raw(number as usize);
            if depth <= max {
                format!("{} -> {}", stringify_address(address), print_value_helper(debug_target, value, depth + 1, max))
            } else {
                stringify_address(address)
            }
        } else {
            number.to_string()
        }
        value::Value::ExecutionToken(xt) => stringify_execution_token(&debug_target, xt)
    }       
}

fn print_memory_formatted(debug_target: &evaluate::ForthState, address: memory::Address, optional_max: Option<memory::Address>, io: evaluate::ForthIO) {
    let mut current_address = address;
    while let Ok(_) = debug_target.check_address(current_address) {
        if optional_max.map(|max| !current_address.less_than(max)).unwrap_or(false) {
            break
        }

        io.output_stream.writeln(&format!("{:<7} | {}", stringify_address(current_address), print_address(debug_target, current_address)));
        current_address.increment_cell();
    }
}

/**
 * Debug operations.
 */
pub(in super) fn view_memory_region(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let label = debugger_state.forth.state.input_stream.next_word()?;
    for mapping in debug_target.memory_map().get_entries().iter() {
        if label == mapping.label.to_uppercase() {
            let io = debugger_state.forth.state.get_forth_io();
            io.output_stream.writeln(&format!("{}:", label.to_lowercase()));
            return Ok(print_memory_formatted(debug_target, mapping.base, None, io));
        }
    }

    Err(evaluate::Error::UnknownWord(label))
}

pub(in super) fn view_memory_map(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) -> evaluate::ForthResult {
    debugger_state.forth.state.output_stream.writeln("memory mapping:");
    for mapping in debug_target.memory_map().get_entries().iter() {
        debugger_state.forth.state.output_stream.writeln(&format!("{}   {}  | {}", stringify_address(mapping.base), mapping.permissions.to_string(), mapping.label));
    }

    Ok(())
}

pub(in super) fn examine_memory(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let address = debugger_state.forth.state.stack.pop()?;
    let format = debugger_state.forth.state.input_stream.next_word()?;
    debugger_state.forth.state.output_stream.writeln(&read_from_address(debug_target, address, &format[..])?);

    Ok(())
}

pub(in super) fn view_state(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let instruction_pointer = debug_target.instruction_pointer().map(|addr| stringify_address(addr)).unwrap_or("(awaiting input)".to_string());
    let execution_mode = match debug_target.execution_mode() {
        evaluate::ExecutionMode::Compile => "compiling",
        evaluate::ExecutionMode::Interpret => "interpreting",
    }.to_string();
    let current_instruction = debug_target.current_instruction().map(|xt| stringify_execution_token(debug_target, xt)).unwrap_or("(awaiting instruction)".to_string());

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
    if let Some(instruction_pointer) = debug_target.instruction_pointer() {
        debugger_state.forth.state.output_stream.writeln("memory:\n");
        let start = if debug_target.read::<value::Value>(instruction_pointer.minus_cell(3)).is_ok() {
            instruction_pointer.minus_cell(3)
        } else {
            instruction_pointer
        };
        print_memory_formatted(debug_target, start, Some(instruction_pointer.plus_cell(4)), debugger_state.forth.state.get_forth_io());
        debugger_state.forth.state.output_stream.writeln("------------------------------------------------------");
    }

    debugger_state.forth.state.output_stream.writeln("stack:\n");
    print_memory_formatted(debug_target, debug_target.stack.get_base(), None, debugger_state.forth.state.get_forth_io());
    debugger_state.forth.state.output_stream.writeln("------------------------------------------------------");

    Ok(())
}

pub(in super) fn all_commands(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) -> evaluate::ForthResult {
    for (word, nametag) in debug_target.definitions.debug_only_get_nametag_map().iter() {
        let definition = debug_target.definitions.get(*nametag);
        let immediate_string = if definition.immediate {
            "immediate"
        } else {
            ""
        };

        debugger_state.forth.state.output_stream.writeln(&format!("{:<15}: {:<10} {}", word, immediate_string, stringify_execution_token(debug_target, definition.execution_token)))
    }

    Ok(())
}

pub(in super) fn add_break(debugger_state: &mut debugger::DebugState, _: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let address = debugger_state.forth.state.stack.pop()?;
    debugger_state.breakpoints.push(address);
    debugger_state.forth.state.output_stream.writeln(&format!("Set breakpoint @ {}", stringify_address(address)));
    
    Ok(())
}

pub(in super) fn step(debugger_state: &mut debugger::DebugState, _: &mut evaluate::ForthState) -> evaluate::ForthResult {
    debugger_state.forth.state.output_stream.writeln("Stepping...");
    debugger_state.stepping = true;
    debugger_state.debugging = false;
    debugger_state.current_error = None;
    
    Ok(())
}

pub(in super) fn do_continue(debugger_state: &mut debugger::DebugState, _: &mut evaluate::ForthState) -> evaluate::ForthResult {
    debugger_state.forth.state.output_stream.writeln("Continuing");
    debugger_state.stepping = false;
    debugger_state.debugging = false;
    debugger_state.current_error = None;
    
    Ok(())
}

pub(in super) fn do_exit(debugger_state: &mut debugger::DebugState, _: &mut evaluate::ForthState) -> evaluate::ForthResult {
    debugger_state.forth.state.output_stream.writeln("Exiting...");
    debugger_state.stepping = false;
    debugger_state.debugging = false;
    debugger_state.current_error = debugger_state.current_error.take().or(Some(evaluate::Error::Halt));
    
    Ok(())
}

pub(in super) fn see(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let definition = debug_target.definitions.get_from_token(debugger_state.forth.state.input_stream.next()?)?;

    debugger_state.forth.state.output_stream.writeln(&stringify_execution_token(debug_target, definition.execution_token));
    if let evaluate::definition::ExecutionToken::Definition(address) = definition.execution_token {
        let mut end = address;
        while {
            end.increment_cell();
            let break_operation = evaluate::definition::ExecutionToken::LeafOperation(operations::control_flow_operations::control_flow_break);
            let current_operation = debug_target.read::<evaluate::definition::ExecutionToken>(end)?;
            break_operation != current_operation
        } {}
        end.increment_cell();

        print_memory_formatted(debug_target, address, Some(end), debugger_state.forth.state.get_forth_io());
    }
    
    Ok(())
}

pub(in super) type DebugOperation = fn(debugger_state: &mut debugger::DebugState, debug_target: &mut evaluate::ForthState) -> evaluate::ForthResult;

pub(in super) const DEBUG_OPERATIONS: &[(&str, DebugOperation)] = &[
    ("XVIEW", view_memory_region),
    ("VMMAP", view_memory_map),
    ("X", examine_memory),
    ("STATE", view_state),
    ("ALL_COMMANDS", all_commands),
    ("SET_BREAK", add_break),
    ("STEP", step),
    ("CONTINUE", do_continue),
    ("EXIT", do_exit),
    ("SEE", see),
];