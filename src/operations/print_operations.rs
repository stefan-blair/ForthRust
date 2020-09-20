use super::*;


pub fn pop_and_print<N: GenericNumber>(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    state.output_stream.write(&format!("{:?} ", state.stack.pop::<N>()?));
    Result::Ok(())
}

pub fn print_newline(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    state.output_stream.writeln("");
    Result::Ok(())
}

pub fn print_string(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    state.heap.push(evaluate::definition::ExecutionToken::LeafOperation(|state| {
        // there must be an instruction pointer if its literally executing this
        let mut string_address = state.instruction_pointer.unwrap();
        let length: generic_numbers::UnsignedByte = state.read(string_address)?;
        for _ in 0..length {
            // increment the string address and read the next character
            string_address.increment();
            let c: generic_numbers::UnsignedByte = state.read(string_address)?;
            // print the byte as a character
            state.output_stream.write(&format!("{}", c as char));
        }

        // now jump to the next instruction
        state.jump_to(string_address.nearest_cell())

    }).value());

    // TODO: THERE SEEMS TO BE A BUG HERE where it pushes 32 onto the stack somewhere for some reason ....
    string_operations::read_string_to_memory(state, '"')
}

pub fn type_string(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let count: generic_numbers::UnsignedNumber = state.stack.pop()?;
    let address: memory::Address = state.stack.pop()?;

    for i in 0..count {
        let c: generic_numbers::UnsignedByte = state.read(address.plus(i as usize))?;
        state.output_stream.write(&format!("{}", c as char));
    }

    Ok(())
}

pub fn emit(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let ascii_char = state.stack.pop::<generic_numbers::UnsignedByte>()? as char;
    state.output_stream.write(&format!("{}", ascii_char));
    Ok(())
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    vec![
        (".", false, pop_and_print::<generic_numbers::Number>),
        ("D.", false, pop_and_print::<generic_numbers::DoubleNumber>),
        ("C.", false, pop_and_print::<generic_numbers::Byte>),
        ("U.", false, pop_and_print::<generic_numbers::UnsignedNumber>),
        (".\"", true, print_string),
        ("CR", false, print_newline),
        ("TYPE", false, type_string),
        ("EMIT", false, emit),
    ]
}