use super::*;


pub fn get_char(state: &mut ForthState) -> ForthResult {
    let c = state.input_stream.next_char()?;

    state.stack.push(c as generic_numbers::Byte);
    Ok(())
}

pub fn read_string_to_memory(state: &mut ForthState, delimiter: char) -> ForthResult {
    let length_address = state.heap.top();
    let mut string_address = length_address.plus(1);
    let mut length: generic_numbers::UnsignedByte = 0;
    loop {
        let next_char = state.input_stream.next_char()?;
        if next_char == delimiter {
            break;
        } else {
            if !string_address.less_than(state.heap.top()) {
                state.heap.push_none::<value::Value>();
            }
            state.write(string_address, next_char as generic_numbers::UnsignedByte)?;
            string_address.increment();
            length += 1;
        }
    }
    state.write(length_address, length)?;

    Ok(())
}

pub fn get_word(state: &mut ForthState) -> ForthResult {
    let delimiter = state.stack.pop::<generic_numbers::UnsignedByte>()? as char;
    let address = state.heap.top();
    read_string_to_memory(state, delimiter).map(|_| state.stack.push(address))
}

pub fn trailing(state: &mut ForthState) -> ForthResult {
    let count: generic_numbers::UnsignedNumber = state.stack.pop()?;
    let address: memory::Address = state.stack.pop()?;

    let mut new_count = 0;
    for i in 0..count {
        let current_char = state.read::<generic_numbers::UnsignedByte>(address.plus(i as usize))? as char;
        if current_char.is_ascii() && !current_char.is_whitespace() {
            new_count = i + 1;
        }
    }

    state.stack.push(address);
    state.stack.push(new_count);

    Ok(())
}

pub fn cmove(state: &mut ForthState) -> ForthResult {
    let count: generic_numbers::UnsignedNumber = state.stack.pop()?;
    let destination: memory::Address = state.stack.pop()?;
    let source: memory::Address = state.stack.pop()?;

    for i in 0..count {
        let current_byte = state.read::<generic_numbers::UnsignedByte>(source.plus(i as usize))?;
        state.write(destination.plus(i as usize), current_byte)?;
    }

    Ok(())
}

pub fn cmove_backwards(state: &mut ForthState) -> ForthResult {
    let count: generic_numbers::UnsignedNumber = state.stack.pop()?;
    let destination: memory::Address = state.stack.pop()?;
    let source: memory::Address = state.stack.pop()?;

    for i in (count - 1)..0 {
        let current_byte = state.read::<generic_numbers::UnsignedByte>(source.plus(i as usize))?;
        state.write(destination.plus(i as usize), current_byte)?;
    }

    Ok(())
}

pub fn move_noclobber(state: &mut ForthState) -> ForthResult {
    let count: generic_numbers::UnsignedNumber = state.stack.pop()?;
    let destination: memory::Address = state.stack.pop()?;
    let source: memory::Address = state.stack.pop()?;
    
    let mut bytes = Vec::new();
    for i in 0..count {
        bytes.push(state.read::<generic_numbers::UnsignedByte>(source.plus(i as usize))?);
    }

    for (i, byte) in bytes.into_iter().enumerate() {
        state.write(destination.plus(i as usize), byte)?;
    }

    Ok(())
}

pub fn accept(state: &mut ForthState) -> ForthResult {
    let count: generic_numbers::UnsignedNumber = state.stack.pop()?;
    let address: memory::Address = state.stack.pop()?;

    let mut copied_characters = 0;
    while copied_characters < count {
        let current_char = state.input_stream.next_char()?;
        
        if current_char == '\n' {
            break;
        }

        state.write(address.plus(copied_characters as usize), current_char as generic_numbers::UnsignedByte)?;
        copied_characters += 1;
    }

    Ok(())
}

pub fn count(state: &mut ForthState) -> ForthResult {
    let address: memory::Address = state.stack.pop()?;
    
    let length = state.read::<generic_numbers::UnsignedByte>(address)?;
    state.stack.push(address.plus(1));
    state.stack.push(length);

    Ok(())
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    vec![
        ("CHAR", false, get_char),
        ("KEY", false, get_char),
        ("WORD", false, get_word),
        ("-TRAILING", false, trailing),
        ("CMOVE", false, cmove),
        ("CMOVE>", false, cmove_backwards),
        ("MOVE", false, move_noclobber),
        ("ACCEPT", false, accept),
        ("COUNT", false, count)
    ]
}