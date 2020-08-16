use super::*;

use crate::absorb_comment;


pub fn immedate(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.definitions.make_immediate(state.definitions.get_most_recent()); CONTINUE_RESULT }
pub fn set_interpret(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { *state.execution_mode = evaluate::ExecutionMode::Interpret; CONTINUE_RESULT }
pub fn set_compile(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { *state.execution_mode = evaluate::ExecutionMode::Compile; CONTINUE_RESULT }

pub fn start_word_compilation(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let name = match get_token!(state) {
        io::tokens::Token::Name(name) => name,
        _ => return Result::Err(evaluate::Error::InvalidWord)
    };

    let address = state.memory.top();
    let execution_token = state.compiled_code.add_compiled_code(Box::new(move |state| {
        state.execute_at(address).map(|_| evaluate::ControlFlowState::Continue)
    }));

    // we will edit the definition to be immediate if the IMMEDIATE keyword is found
    state.definitions.add(name, evaluate::Definition::new(execution_token, false));

    set_compile(state)
}

pub fn end_word_compilation(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    postpone!(state, super::control_flow_operations::control_flow_break);
    set_interpret(state)
}

pub fn postpone(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let token = get_token!(state);
    let definition = match state.definitions.get_from_token(token) {
        Some(definition) => definition,
        None => return Result::Err(evaluate::Error::UnknownWord)
    };

    let xt = if definition.immediate {
        definition.execution_token
    } else {
        state.compiled_code.add_compiled_code(Box::new(move |state| {
            state.memory.push(definition.execution_token.value());
            CONTINUE_RESULT
        }))
    };

    state.memory.push(xt.value());

    CONTINUE_RESULT
}

pub fn literal(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::push_value(pop_or_underflow!(state.stack)));
    state.memory.push(xt.value());
    CONTINUE_RESULT
}

pub fn execute(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let execution_token = match pop_or_underflow!(state.stack) {
        memory::Value::ExecutionToken(execution_token) => execution_token,
        _ => return Result::Err(evaluate::Error::InvalidExecutionToken)
    };
    state.execute(execution_token)
}

// read the next token from the input stream
pub fn read_execution_token(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    state.input_stream
        .next().ok_or(evaluate::Error::NoMoreTokens)
        .and_then(|token| state.definitions.get_from_token(token).ok_or(evaluate::Error::UnknownWord))
        .map(|definition| {
            state.stack.push(memory::Value::ExecutionToken(definition.execution_token));
            evaluate::ControlFlowState::Continue
        })       
}

pub fn get_execution_token(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    state.input_stream
        .next().ok_or(evaluate::Error::NoMoreTokens)
        .and_then(|token| state.definitions.get_from_token(token).ok_or(evaluate::Error::UnknownWord))
        .map(|definition| {
            let xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::push_value(definition.execution_token.value()));
            state.memory.push(xt.value());
            evaluate::ControlFlowState::Continue
        })        
}

/**
 * Generates a bne instruction.  Pops an address off of the stack to be the destination for the branch.
 * Pushes the execution token of this branch instruction onto the stack.
 */
pub fn push_branch_false_instruction(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
    let xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_false_instruction(address));
    state.stack.push(xt.value());
    CONTINUE_RESULT            
}

pub fn push_branch_instruction(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
    let xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_instruction(address));
    state.stack.push(xt.value());
    CONTINUE_RESULT            
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    vec![
        ("IMMEDIATE", false, immedate),
        ("[", true, set_interpret),
        ("]", true, set_compile),
        (":", false, start_word_compilation),
        (";", true, end_word_compilation),
        ("POSTPONE", true, postpone),
        ("LITERAL", true, literal),
        ("EXECUTE", false, execute),
        ("`", false, read_execution_token),
        ("[`]", true, get_execution_token),
        ("(", true, absorb_comment!(")")),
        // branch generators
        ("_BNE", false, push_branch_false_instruction),
        ("_B", false, push_branch_instruction),
    ]
}