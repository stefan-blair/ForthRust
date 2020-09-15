use super::*;


pub fn dereference<N: value::ValueVariant>(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let address = state.stack.pop()?;
    state.stack.push(state.memory.read::<N>(address)?);
    Ok(())
}

pub fn memory_write<N: value::ValueVariant>(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let (address, value) = (state.stack.pop()?, state.stack.pop::<N>()?);

    state.memory.write(address, value)
}

pub fn pop_write(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    state.memory.push(state.stack.pop::<value::Value>()?);
    Ok(())
}

pub fn to(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let word = state.input_stream.next_word()?;
    let nametag = state.definitions.get_nametag(&word)?;

    instruction_compiler::InstructionCompiler::with_state(state).push(nametag.to_number())?;
    state.memory.push(evaluate::definition::ExecutionToken::LeafOperation(|state| {
        let nametag = evaluate::definition::NameTag::from(state.stack.pop()?);
        let number = state.stack.pop::<generic_numbers::Number>()?;
        state.definitions.set(nametag, evaluate::definition::Definition::new(evaluate::definition::ExecutionToken::Number(number), false));
        Ok(())
    }));

    Ok(())
}

macro_rules! generic_operations {
    ($pre:tt, $type:ty) => {
        vec![
            (concat!($pre, "!") , false, memory_write::<$type> as super::Operation),
            (concat!($pre, "@"), false, dereference::<$type>)
        ]
    };
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    let mut operations: Vec<(&'static str, bool, super::Operation)> = vec![
        ("TO", true, to),
        (",", false, pop_write),
    ];

    operations.append(&mut generic_operations!("", value::Value));
    operations.append(&mut generic_operations!("C", generic_numbers::Byte));
    operations.append(&mut generic_operations!("2", value::DoubleValue));

    operations
}