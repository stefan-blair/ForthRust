use super::*;

use crate::get_token;
use crate::hard_match_address;
use crate::pop_or_underflow;
use crate::hard_match_number;


pub fn dereference(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
    state.stack.push(state.memory.read(address));
    CONTINUE_RESULT
}

pub fn memory_write(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let (address, value) = (hard_match_address!(state.memory, pop_or_underflow!(state.stack)), pop_or_underflow!(state.stack));
    state.memory.write(address, value);
    CONTINUE_RESULT
}

pub fn pop_write(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    state.memory.push(pop_or_underflow!(state.stack));
    CONTINUE_RESULT
}

pub fn number_dereference<N: GenericNumber>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
    state.stack.push_number::<N>(state.memory.read_number::<N>(address));    
    CONTINUE_RESULT
}

pub fn number_write<N: GenericNumber>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let number = match state.stack.pop_number::<N>() {
        Some(x) => x,
        None => return Result::Err(evaluate::Error::StackUnderflow)
    };
    let address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
    state.memory.write_number::<N>(address, number);
    CONTINUE_RESULT
}
    
pub fn to(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let name = match get_token!(state) {
        io::tokens::Token::Name(name) => name,
        _ => return Result::Err(evaluate::Error::InvalidWord)
    };
    let nametag = match state.definitions.get_nametag(&name) {
        Some(nametag) => nametag,
        None => return Result::Err(evaluate::Error::UnknownWord)
    };

    state.memory.push(state.compiled_code.add_compiled_code(Box::new(move |state| {
        let number = hard_match_number!(pop_or_underflow!(state.stack));
        state.definitions.set(nametag, evaluate::definition::Definition::new(evaluate::definition::ExecutionToken::Number(number), false));
        CONTINUE_RESULT
    })).value());

    CONTINUE_RESULT
}

macro_rules! generic_operations {
    ($pre:tt, $type:ty) => {
        vec![
            (concat!($pre, "!") , false, number_write::<$type> as super::Operation),
            (concat!($pre, "@"), false, number_dereference::<$type>)
        ]
    };
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    let mut operations: Vec<(&'static str, bool, super::Operation)> = vec![
        ("!", false, memory_write),
        ("@", false, dereference),
        ("TO", true, to),
        (",", false, pop_write),
    ];

    operations.append(&mut generic_operations!("C", generic_numbers::Byte));
    operations.append(&mut generic_operations!("2", generic_numbers::DoubleNumber));

    operations
}