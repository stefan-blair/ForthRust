use super::*;

pub fn control_flow_break(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { state.return_from() }

pub fn do_init_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    // make room for an instruction to be patched in later that will push the end address of the loop onto the stack, for use by leave instructions 
    state.memory.push_none::<value::Value>();
    // add an instruction to move the address from the stack to the return stack
    postpone!(state, stack_operations::stack_to_return_stack::<value::Value>);
    // add instructions to the current definition that initialize the loop by moving the bounds onto the return stack
    postpone!(state, super::stack_operations::stack_to_return_stack::<value::DoubleValue>);

    // put the the address of the loop prologue so the end can patch it     TODO add this to all of the loop beginnings
    state.stack.push(state.memory.top().to_number());

    Result::Ok(())
}

pub fn loop_plus_compiletime(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    // push the loop runtime
    state.memory.push(evaluate::definition::ExecutionToken::LeafOperation(|state| {
        // pop off the step from the stack, and the range from the return stack
        let (step, start, end): (generic_numbers::Number, generic_numbers::Number, generic_numbers::Number) = (state.stack.pop()?, state.return_stack.pop()?, state.return_stack.pop()?);

        let new_start = start + step;
        // we use a "branch false" instruction, so we want to check for falsehood
        state.stack.push((new_start >= end) as generic_numbers::Number);
        state.return_stack.push(end);
        state.return_stack.push(new_start);
        Result::Ok(())
    }).value());

    // get the address of the top of the loop, and patch the conditional branch at the end of the loop
    let loop_address = state.stack.pop()?;
    compiled_instructions::InstructionCompiler::with_state(state).branch_false(loop_address)?;

    // add an epilogue to pop the state off of the return stack
    state.memory.push(evaluate::definition::ExecutionToken::LeafOperation(|state| {
        // pop the start and end values
        state.return_stack.pop::<value::DoubleValue>()?;
        // pop the leave address
        state.return_stack.pop::<value::Value>()?;
        Result::Ok(())
    }).value());
    

    // fill in the blank space at the beginning of the loop with the address of the end of the loop so that it gets pushed onto the stack for leave instructions
    state.memory.write(loop_address.minus_cell(3), evaluate::definition::ExecutionToken::Number(state.memory.top().to_number()))
}

pub fn loop_compiletime(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    // postpone pushing 1 onto the stack, which is the expected step value on the stack (+LOOP has an explicit step)
    state.memory.push(evaluate::definition::ExecutionToken::Number(1).value());
    loop_plus_compiletime(state)
}

pub fn begin_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    // leave room for the leave instruction
    state.memory.push_none::<value::Value>();
    postpone!(state, stack_operations::stack_to_return_stack::<value::Value>);

    state.stack.push(state.memory.top().to_number());
    Result::Ok(())
}

pub fn until_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let loop_address = state.stack.pop()?;
    compiled_instructions::InstructionCompiler::with_state(state).branch_false(loop_address)?;

    // fill in the blank space at the beginning of the loop with the address of the end of the loop so that it gets pushed onto the stack for leave instructions
    state.memory.write(loop_address.minus_cell(2), evaluate::definition::ExecutionToken::Number(state.memory.top().to_number()))
}

pub fn again_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let loop_address = state.stack.pop()?;
    compiled_instructions::InstructionCompiler::with_state(state).branch(loop_address)?;

    // fill in the blank space at the beginning of the loop with the address of the end of the loop so that it gets pushed onto the stack for leave instructions
    state.memory.write(loop_address.minus_cell(2), evaluate::definition::ExecutionToken::Number(state.memory.top().to_number()))
}

pub fn while_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    state.stack.push(state.memory.top().to_number());
    state.memory.push_none::<value::Value>();
    Result::Ok(())
}

pub fn repeat_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let branch_address = state.stack.pop()?;

    // add a branch instruction to the beginning of the loop unconditionally
    let loop_start_address = state.stack.pop()?;
    compiled_instructions::InstructionCompiler::with_state(state).branch(loop_start_address)?;

    // back patch the conditional branch in the middle of the loop
    let loop_middle_address = state.memory.top();
    compiled_instructions::InstructionCompiler::with_state(state).with_address(branch_address).branch_false(loop_middle_address)?;

    // fill in the blank space at the beginning of the loop with the address of the end of the loop so that it gets pushed onto the stack for leave instructions
    state.memory.write(loop_start_address.minus_cell(2), evaluate::definition::ExecutionToken::Number(state.memory.top().to_number()))
}

pub fn leave(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    state.return_stack.pop::<value::DoubleValue>()?;
    let end_of_loop_address = state.return_stack.pop()?;
    state.jump_to(end_of_loop_address)
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    vec![
        ("BREAK", false, control_flow_break),
        ("DO", true, do_init_loop),
        ("+LOOP", true, loop_plus_compiletime),
        ("LOOP", true, loop_compiletime),
        ("BEGIN", true, begin_loop),
        ("UNTIL", true, until_loop),
        ("AGAIN", true, again_loop),
        ("WHILE", true, while_loop),
        ("REPEAT", true, repeat_loop),
        ("LEAVE", false, leave),        
    ]
}