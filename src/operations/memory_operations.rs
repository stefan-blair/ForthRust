use super::*;

use crate::get_token;
use crate::hard_match_address;
use crate::pop_or_underflow;


pub fn dereference<N: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let address = pop_address!(state.memory, state.stack);
    state.stack.push(state.memory.read::<value::Value>(address));
    Result::Ok(())
}

pub fn memory_write<N: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let (address, value) = (pop_address!(state.memory, state.stack), pop_or_underflow!(state.stack, N));
    state.memory.write(address, value);
    Result::Ok(())
}

pub fn pop_write(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    state.memory.push(pop_or_underflow!(state.stack, value::Value));
    Result::Ok(())
}

pub fn to(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let name = match get_token!(state) {
        io::tokens::Token::Name(name) => name,
        _ => return Result::Err(evaluate::Error::InvalidWord)
    };
    let nametag = match state.definitions.get_nametag(&name) {
        Some(nametag) => nametag,
        None => return Result::Err(evaluate::Error::UnknownWord)
    };

    state.memory.push(state.compiled_code.add_compiled_code(Box::new(move |state| {
        let number = pop_or_underflow!(state.stack, generic_numbers::Number);
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