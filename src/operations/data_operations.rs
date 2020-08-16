use super::*;

use crate::hard_match_number;
use crate::pop_or_underflow;
use crate::get_token;
use crate::postpone;


pub fn here(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.stack.push(state.memory.top().to_number().value()); CONTINUE_RESULT }
pub fn allot(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.memory.expand(hard_match_number!(pop_or_underflow!(state.stack)) as memory::Offset); CONTINUE_RESULT }

pub fn create(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let name = match get_token!(state) {
        io::tokens::Token::Name(name) => name,
        _ => return Result::Err(evaluate::Error::InvalidWord)
    };

    let address = state.memory.top();
    state.memory.push_none();
    let xt = state.compiled_code.add_compiled_code(Box::new(move |state| { state.stack.push(address.to_number().value()); CONTINUE_RESULT } ));
    state.definitions.add(name, evaluate::Definition::new(xt, false));

    CONTINUE_RESULT
}

pub fn does(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {

    // execution token that will execute the remainder of the function, add 2 to bypass added ending
    let address = state.memory.top().plus_cell(2);
    let xt = state.compiled_code.add_compiled_code(Box::new(move |state| {
        state.execute_at(address).map(|_| evaluate::ControlFlowState::Continue)
    }));

    // this will wrap the most recent definition with the remainder of the code
    let wrapper_xt = state.compiled_code.add_compiled_code(Box::new(move |state| {
        let old_definition = state.definitions.get(state.definitions.get_most_recent());
        let wrapped_xt = state.compiled_code.add_compiled_code(Box::new(move |state| {
            state.execute(old_definition.execution_token).and_then(|_|state.execute(xt))
        }));
        state.definitions.set(state.definitions.get_most_recent(), evaluate::Definition::new(wrapped_xt, false));
        CONTINUE_RESULT
    }));

    state.memory.push(wrapper_xt.value());

    // add a manual break, so that normal calls to the function wont execute the rest of the code, only created objects
    postpone!(state, super::control_flow_operations::control_flow_break);

    CONTINUE_RESULT
}

pub fn value(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let name = match get_token!(state) {
        io::tokens::Token::Name(name) => name,
        _ => return Result::Err(evaluate::Error::InvalidWord)
    };

    let number = hard_match_number!(pop_or_underflow!(state.stack));

    state.definitions.add(name, evaluate::Definition::new(memory::ExecutionToken::Number(number), false));
    CONTINUE_RESULT
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    vec![
        ("HERE", false, here),
        ("ALLOT", false, allot),
        ("CREATE", false, create),
        ("DOES>", true, does),
        ("VALUE", false, value),
    ]
}
