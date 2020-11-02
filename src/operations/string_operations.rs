use super::*;


pub fn get_char(state: &mut ForthState) -> ForthResult {
    let c = state.input_stream.next_char()?;

    state.stack.push(c as generic_numbers::Byte);
    Ok(())
}

pub fn read_string_to_memory(state: &mut ForthState, delimiter: char) -> ForthResult {
    let length_address = state.data_space.top();
    let mut string_address = length_address.plus(Bytes::one());
    let mut length: generic_numbers::UnsignedByte = 0;
    loop {
        let next_char = state.input_stream.next_char()?;
        if next_char == delimiter {
            break;
        } else {
            if !string_address.less_than(state.data_space.top()) {
                state.data_space.push_none::<value::Value>();
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
    let address = state.data_space.top();
    read_string_to_memory(state, delimiter).map(|_| state.stack.push(address))
}

pub fn trailing(state: &mut ForthState) -> ForthResult {
    let count: usize = state.stack.pop()?;
    let address: memory::Address = state.stack.pop()?;

    let mut new_count = Bytes::zero();
    for i in (0..count).map(|i| Bytes::from(i)) {
        let current_char = state.read::<generic_numbers::UnsignedByte>(address.plus(i))? as char;
        if current_char.is_ascii() && !current_char.is_whitespace() {
            new_count = i + Bytes::one();
        }
    }

    state.stack.push(address);
    state.stack.push(new_count);

    Ok(())
}

pub fn cmove(state: &mut ForthState) -> ForthResult {
    let count: usize = state.stack.pop()?;
    let destination: memory::Address = state.stack.pop()?;
    let source: memory::Address = state.stack.pop()?;

    for i in (0..count).map(|i| Bytes::from(i)) {
        let current_byte = state.read::<Bytes>(source.plus(i))?;
        state.write(destination.plus(i), current_byte)?;
    }

    Ok(())
}

pub fn cmove_backwards(state: &mut ForthState) -> ForthResult {
    let count: usize = state.stack.pop()?;
    let destination: memory::Address = state.stack.pop()?;
    let source: memory::Address = state.stack.pop()?;

    for i in ((count - 1)..0).map(|i| Bytes::from(i)) {
        let current_byte = state.read::<generic_numbers::UnsignedByte>(source.plus(i))?;
        state.write(destination.plus(i), current_byte)?;
    }

    Ok(())
}

pub fn move_noclobber(state: &mut ForthState) -> ForthResult {
    let count: usize = state.stack.pop()?;
    let destination: memory::Address = state.stack.pop()?;
    let source: memory::Address = state.stack.pop()?;
    
    let mut bytes = Vec::new();
    for i in (0..count).map(|i| Bytes::from(i)) {
        bytes.push(state.read::<generic_numbers::UnsignedByte>(source.plus(i))?);
    }

    for (i, byte) in bytes.into_iter().enumerate() {
        state.write(destination.plus(Bytes::from(i)), byte)?;
    }

    Ok(())
}

pub fn accept(state: &mut ForthState) -> ForthResult {
    let count: Bytes = state.stack.pop()?;
    let address: memory::Address = state.stack.pop()?;

    let mut copied_characters = Bytes::zero();
    while copied_characters < count {
        let current_char = state.input_stream.next_char()?;
        
        if current_char == '\n' {
            break;
        }

        state.write(address.plus(copied_characters), current_char as generic_numbers::UnsignedByte)?;
        copied_characters += Bytes::one();
    }

    Ok(())
}

pub fn count(state: &mut ForthState) -> ForthResult {
    let address: memory::Address = state.stack.pop()?;
    
    let length = state.read::<generic_numbers::UnsignedByte>(address)?;
    state.stack.push(address.plus(Bytes::one()));
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