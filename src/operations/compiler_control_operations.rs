use super::*;

use crate::absorb_comment;


pub fn immedate(state: &mut evaluate::ForthState) -> evaluate::ForthResult { state.definitions.make_immediate(state.definitions.get_most_recent_nametag()); Result::Ok(()) }
pub fn set_interpret(state: &mut evaluate::ForthState) -> evaluate::ForthResult { state.set_interpretmode() }
pub fn set_compile(state: &mut evaluate::ForthState) -> evaluate::ForthResult { state.set_compilemode() }

pub fn start_word_compilation(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let word = state.input_stream.next_word()?;
    let address = state.heap.top();
    let execution_token = evaluate::definition::ExecutionToken::Definition(address);

    // the IMMEDIATE keyword will edit the definition to be immediate
    state.definitions.add(word, evaluate::definition::Definition::new(execution_token, false));

    set_compile(state)
}

pub fn end_word_compilation(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    postpone!(state, super::control_flow_operations::control_flow_break);
    set_interpret(state)
}

pub fn postpone(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let definition = state.definitions.get_from_token(state.input_stream.next()?)?;

    if definition.immediate {
        Ok(state.heap.push(definition.execution_token))
    } else {
        instruction_compiler::InstructionCompiler::with_state(state).mem_push(definition.execution_token.value())
    }
}

pub fn literal<N: value::ValueVariant + 'static>(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let value = state.stack.pop::<N>()?;
    instruction_compiler::InstructionCompiler::with_state(state).push(value)
}

pub fn execute(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let execution_token = state.stack.pop::<evaluate::definition::ExecutionToken>()?;
    state.execute(execution_token)
}

// read the next token from the input stream
pub fn read_execution_token(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    state.input_stream.next()
        .and_then(|token| state.definitions.get_from_token(token))
        .map(|definition| state.stack.push(definition.execution_token))       
}

pub fn get_execution_token(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    state.input_stream.next()
        .and_then(|token| state.definitions.get_from_token(token))
        .and_then(|definition| instruction_compiler::InstructionCompiler::with_state(state).push(definition.execution_token.value()))
}

/**
 * Generates a bne instruction.  Pops an address off of the stack to be the destination for the branch.
 * Pushes the execution token of this branch instruction onto the stack.
 */
pub fn write_branch_false(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let branch_target = state.stack.pop()?;
    let destination = state.stack.pop()?;
    instruction_compiler::InstructionCompiler::with_state(state).with_address(destination).branch_false(branch_target)
}

pub fn write_branch(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let branch_target = state.stack.pop()?;
    let destination = state.stack.pop()?;
    instruction_compiler::InstructionCompiler::with_state(state).with_address(destination).branch(branch_target)
}

pub fn body(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let xt = state.stack.pop()?;
    match xt {
        evaluate::definition::ExecutionToken::Definition(address) => state.stack.push(address),
        evaluate::definition::ExecutionToken::Number(i) => state.stack.push(i),
        _ => state.stack.push(xt)
    };

    Ok(())
}

pub fn execution_mode_address(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    Ok(state.stack.push(state.internal_state_memory().execution_mode.address))
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    vec![
        ("IMMEDIATE", false, immedate),
        ("[", true, set_interpret),
        ("]", true, set_compile),
        (":", false, start_word_compilation),
        (";", true, end_word_compilation),
        ("POSTPONE", true, postpone),
        ("LITERAL", true, literal::<value::Value>),
        ("EXECUTE", false, execute),
        ("'", false, read_execution_token),
        (">BODY", false, body),
        ("[']", true, get_execution_token),
        ("(", true, absorb_comment!(')')),
        ("\\", true, absorb_comment!('\n')),
        ("STATE", false, execution_mode_address),
        // branch generators
        ("_BNE", false, write_branch_false),
        ("_B", false, write_branch),
    ]
}