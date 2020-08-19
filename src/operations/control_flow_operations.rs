use super::*;

pub fn control_flow_break(_: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { Result::Ok(evaluate::ControlFlowState::Break) }

pub fn do_init_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    postpone!(state, super::stack_operations::twice_stack_to_return_stack);
    state.stack.push(state.memory.top().to_number().value());
    state.return_stack.push(memory::Value::Number(0));
    CONTINUE_RESULT
}

fn loop_runtime(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    // pop off the step from the stack, and the range from the return stack
    let (step, end, start) = match (pop_or_underflow!(state.stack), get_two_from_stack!(state.return_stack)) {
        (memory::Value::Number(step), (memory::Value::Number(end), memory::Value::Number(start))) => (step, end, start),
        _ => return Result::Err(evaluate::Error::InvalidNumber)
    };

    let new_start = start + step;
    // we use a "branch false" instruction, so we want to check for falsehood
    state.stack.push(memory::Value::Number((new_start >= end) as generic_numbers::Number));
    state.return_stack.push(memory::Value::Number(new_start));
    state.return_stack.push(memory::Value::Number(end));
    CONTINUE_RESULT
}

pub fn loop_plus_compiletime(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    postpone!(state, loop_runtime);

    // get the address of the top of the loop, and patch the conditional branch at the end of the loop
    let loop_address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
    let branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_false_instruction(loop_address)).value();
    state.memory.push(branch_xt);

    patch_leave_instructions(state).map(|c| {
        // pop from the return stack
        postpone!(state, super::stack_operations::rdrop);
        postpone!(state, super::stack_operations::rdrop);
        c
    })
}

pub fn loop_compiletime(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    // postpone pushing 1 onto the stack, which is the expected step value on the stack (+LOOP has an explicit step)
    state.memory.push(evaluate::definition::ExecutionToken::Number(1).value());
    loop_plus_compiletime(state)
}

pub fn begin_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    state.stack.push(state.memory.top().to_number().value());
    state.return_stack.push(memory::Value::Number(0));
    CONTINUE_RESULT
}

pub fn until_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let loop_address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
    let branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_false_instruction(loop_address)).value();
    state.memory.push(branch_xt);

    patch_leave_instructions(state)
}

pub fn again_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let loop_address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
    let branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_instruction(loop_address)).value();
    state.memory.push(branch_xt);

    patch_leave_instructions(state)
}

pub fn while_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    state.stack.push(state.memory.top().to_number().value());
    state.memory.push_none();
    CONTINUE_RESULT
}

pub fn repeat_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let branch_address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));

    // add a branch instruction to the beginning of the loop unconditionally
    let loop_address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
    let branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_instruction(loop_address)).value();
    state.memory.push(branch_xt);

    // back patch the conditional branch in the middle of the loop
    let conditional_branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_false_instruction(state.memory.top())).value();
    state.memory.write(branch_address, conditional_branch_xt);

    patch_leave_instructions(state)
}

pub fn patch_leave_instructions(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let leave_address_count = hard_match_number!(pop_or_underflow!(state.return_stack));
    // if there were any leave instructions, iterate through them patch them to jump to the end of the loop
    if leave_address_count > 0 {
        let leave_branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_instruction(state.memory.top())).value();

        for _ in 0..leave_address_count {
            let leave_address = hard_match_address!(state.memory, pop_or_underflow!(state.return_stack));
            state.memory.write(leave_address, leave_branch_xt);
        }
    }
    CONTINUE_RESULT
}

pub fn leave(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let leave_address_count = hard_match_number!(pop_or_underflow!(state.return_stack)) + 1;
    state.return_stack.push(state.memory.top().to_number().value());
    state.return_stack.push(memory::Value::Number(leave_address_count));
    state.memory.push_none();
    CONTINUE_RESULT
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
        ("LEAVE", true, leave),        
    ]
}