use super::*;

use crate::absorb_comment;


pub fn immedate(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { state.definitions.make_immediate(state.definitions.get_most_recent()); Result::Ok(()) }
pub fn set_interpret(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { state.set_interpretmode() }
pub fn set_compile(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { state.set_compilemode() }

pub fn start_word_compilation(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let name = match get_token!(state) {
        io::tokens::Token::Name(name) => name,
        _ => return Result::Err(evaluate::Error::InvalidWord)
    };

    let address = state.memory.top();
    let execution_token = evaluate::definition::ExecutionToken::DefinedOperation(address);

    // the IMMEDIATE keyword will edit the definition to be immediate
    state.definitions.add(name, evaluate::definition::Definition::new(execution_token, false));

    set_compile(state)
}

pub fn end_word_compilation(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    postpone!(state, super::control_flow_operations::control_flow_break);
    set_interpret(state)
}

pub fn postpone(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
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
            Result::Ok(())
        }))
    };

    state.memory.push(xt.value());

    Result::Ok(())
}

pub fn literal(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::push_value(pop_or_underflow!(state.stack, value::Value)));
    state.memory.push(xt.value());
    Result::Ok(())
}

pub fn execute(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let execution_token = pop_or_underflow!(state.stack, evaluate::definition::ExecutionToken);
    state.execute(execution_token)
}

// read the next token from the input stream
pub fn read_execution_token(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    state.input_stream
        .next().ok_or(evaluate::Error::NoMoreTokens)
        .and_then(|token| state.definitions.get_from_token(token).ok_or(evaluate::Error::UnknownWord))
        .map(|definition| state.stack.push(definition.execution_token))       
}

pub fn get_execution_token(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    state.input_stream
        .next().ok_or(evaluate::Error::NoMoreTokens)
        .and_then(|token| state.definitions.get_from_token(token).ok_or(evaluate::Error::UnknownWord))
        .map(|definition| {
            let xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::push_value(definition.execution_token.value()));
            state.memory.push(xt.value());
        })
}

/**
 * Generates a bne instruction.  Pops an address off of the stack to be the destination for the branch.
 * Pushes the execution token of this branch instruction onto the stack.
 */
pub fn push_branch_false_instruction(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let address = pop_address!(state.memory, state.stack);
    let xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_false_instruction(address));
    state.stack.push(xt);
    Result::Ok(())            
}

pub fn push_branch_instruction(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let address = pop_address!(state.memory, state.stack);
    let xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_instruction(address));
    state.stack.push(xt);
    Result::Ok(())            
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