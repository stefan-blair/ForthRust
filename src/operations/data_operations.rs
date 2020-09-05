use super::*;

use crate::pop_or_underflow;
use crate::get_token;
use crate::postpone;


pub fn here(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { state.stack.push(state.memory.top().to_number()); Result::Ok(()) }

pub fn allot(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { 
    let total_memory = pop_or_underflow!(state.stack, generic_numbers::UnsignedNumber) as memory::Offset;
    let cells = (total_memory + memory::CELL_SIZE - 1) / memory::CELL_SIZE;
    state.memory.expand(cells as memory::Offset); 
    Result::Ok(()) 
}

pub fn create(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let name = match get_token!(state) {
        io::tokens::Token::Name(name) => name,
        _ => return Result::Err(evaluate::Error::InvalidWord)
    };

    let address = state.memory.top();
    state.memory.push_none();
    let xt = evaluate::definition::ExecutionToken::Number(address.to_number());
    state.definitions.add(name, evaluate::definition::Definition::new(xt, false));

    Result::Ok(())
}

pub fn does(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {

    // execution token that will execute the remainder of the function, add 2 to bypass added ending
    let address = state.memory.top().plus_cell(2);
    let xt = evaluate::definition::ExecutionToken::DefinedOperation(address);

    // this will wrap the most recent definition with the remainder of the code
    let wrapper_xt = state.compiled_code.add_compiled_code(Box::new(move |state| {
        let old_definition = state.definitions.get(state.definitions.get_most_recent());
        let wrapped_xt = state.compiled_code.add_compiled_code(Box::new(move |state| {
            state.execute(old_definition.execution_token).and_then(|_|state.execute(xt))
        }));
        state.definitions.set(state.definitions.get_most_recent(), evaluate::definition::Definition::new(wrapped_xt, false));
        Result::Ok(())
    }));

    state.memory.push(wrapper_xt.value());

    // add a manual break, so that normal calls to the function wont execute the rest of the code, only created objects
    postpone!(state, super::control_flow_operations::control_flow_break);

    Result::Ok(())
}

pub fn value(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let name = match get_token!(state) {
        io::tokens::Token::Name(name) => name,
        _ => return Result::Err(evaluate::Error::InvalidWord)
    };

    let number = pop_or_underflow!(state.stack, generic_numbers::Number);

    state.definitions.add(name, evaluate::definition::Definition::new(evaluate::definition::ExecutionToken::Number(number), false));
    Result::Ok(())
}

pub fn cells(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let number = pop_or_underflow!(state.stack, generic_numbers::UnsignedNumber) as memory::Offset;
    state.stack.push((number * memory::CELL_SIZE) as generic_numbers::UnsignedNumber);
    Ok(())
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    vec![
        ("HERE", false, here),
        ("ALLOT", false, allot),
        ("CREATE", false, create),
        ("DOES>", true, does),
        ("VALUE", false, value),
        ("CELLS", false, cells),
    ]
}
