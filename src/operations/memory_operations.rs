use super::*;


pub fn dereference<N: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let address = state.stack.pop()?;
    state.stack.push(state.memory.read::<value::Value>(address)?);
    Result::Ok(())
}

pub fn memory_write<N: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let (address, value) = (state.stack.pop()?, state.stack.pop::<N>()?);

    state.memory.write(address, value)?;
    Result::Ok(())
}

pub fn pop_write(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    state.memory.push(state.stack.pop::<value::Value>()?);
    Result::Ok(())
}

pub fn to(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let name = state.input_stream.next_word()?;
    let nametag = state.definitions.get_nametag(&name)?;

    state.memory.push(state.compiled_code.add_compiled_code(Box::new(move |state| {
        let number = state.stack.pop::<generic_numbers::Number>()?;
        state.definitions.set(nametag, evaluate::definition::Definition::new(evaluate::definition::ExecutionToken::Number(number), false));
        Result::Ok(())
    })).value());

    Result::Ok(())
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