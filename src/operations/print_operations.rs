use super::*;


pub fn pop_and_print<N: GenericNumber>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    state.output_stream.write(&format!("{:?} ", pop_or_underflow!(state.stack, N)));
    Result::Ok(())
}

pub fn print_newline(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    state.output_stream.writeln("");
    Result::Ok(())
}

pub fn print_string(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    // this will probably be ripped out
    state.memory.push(evaluate::definition::ExecutionToken::Operation(|state| {

        println!("{:?}", state.stack.to_vec().iter().map(|x| x.to_number()).collect::<Vec<_>>());

        // there must be an instruction pointer if its literally executing this
        let mut string_address = state.instruction_pointer.unwrap();
        let length: generic_numbers::UnsignedByte = read_or_error!(state.memory.read(string_address));
        for _ in 0..length {
            // increment the string address and read the next character
            string_address.increment();
            let c: generic_numbers::UnsignedByte = read_or_error!(state.memory.read(string_address));
            // print the byte as a character
            state.output_stream.write(&format!("{}", c as char));
        }
        
        println!("{:?}", state.stack.to_vec().iter().map(|x| x.to_number()).collect::<Vec<_>>());

        // now jump to the next instruction
        state.jump_to(string_address.nearest_cell())

    }).value());

    // TODO: THERE SEEMS TO BE A BUG HERE where it pushes 32 onto the stack somewhere for some reason ....

    let length_address = state.memory.top();
    let mut string_address = length_address.plus(1);
    let mut length: generic_numbers::UnsignedByte = 0;
    loop {
        match state.input_stream.next_char() {
            Some('\"') => break,
            Some(next_char) => {
                if !string_address.less_than(state.memory.top()) {
                    state.memory.push_none();
                }
                write_or_error!(state.memory, string_address, next_char as generic_numbers::UnsignedByte);
                string_address.increment();
                length += 1;
            },
            None => return Result::Err(evaluate::Error::NoMoreTokens)
        }
    }
    write_or_error!(state.memory, length_address, length);

    
    Result::Ok(())
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    vec![
        (".", false, pop_and_print::<generic_numbers::Number>),
        ("D.", false, pop_and_print::<generic_numbers::DoubleNumber>),
        ("C.", false, pop_and_print::<generic_numbers::Byte>),
        ("U.", false, pop_and_print::<generic_numbers::UnsignedNumber>),
        (".\"", true, print_string),
        ("CR", false, print_newline)    
    ]
}