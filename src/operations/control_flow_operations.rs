use super::*;

pub fn control_flow_break(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { state.return_from() }

pub fn do_init_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    postpone!(state, super::stack_operations::stack_to_return_stack::<value::DoubleValue>);
    state.stack.push(state.memory.top().to_number());
    state.return_stack.push(0 as generic_numbers::Number);
    Result::Ok(())
}

fn loop_runtime(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    // pop off the step from the stack, and the range from the return stack
    let (step, (start, end)) = (pop_or_underflow!(state.stack, generic_numbers::Number), get_two_from_stack!(state.return_stack, generic_numbers::Number, generic_numbers::Number));

    let new_start = start + step;
    // we use a "branch false" instruction, so we want to check for falsehood
    state.stack.push((new_start >= end) as generic_numbers::Number);
    state.return_stack.push(end);
    state.return_stack.push(new_start);
    Result::Ok(())
}

pub fn loop_plus_compiletime(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    postpone!(state, loop_runtime);

    // get the address of the top of the loop, and patch the conditional branch at the end of the loop
    let loop_address = pop_address!(state.memory, state.stack);
    let branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_false_instruction(loop_address)).value();
    state.memory.push(branch_xt);

    patch_leave_instructions(state).map(|c| {
        // pop from the return stack
        postpone!(state, super::stack_operations::rdrop::<value::DoubleValue>);
        c
    })
}

pub fn loop_compiletime(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    // postpone pushing 1 onto the stack, which is the expected step value on the stack (+LOOP has an explicit step)
    state.memory.push(evaluate::definition::ExecutionToken::Number(1).value());
    loop_plus_compiletime(state)
}

pub fn begin_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    state.stack.push(state.memory.top().to_number());
    state.return_stack.push(0 as generic_numbers::Number);
    Result::Ok(())
}

pub fn until_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let loop_address = pop_address!(state.memory, state.stack);
    let branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_false_instruction(loop_address)).value();
    state.memory.push(branch_xt);

    patch_leave_instructions(state)
}

pub fn again_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let loop_address = pop_address!(state.memory, state.stack);
    let branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_instruction(loop_address)).value();
    state.memory.push(branch_xt);

    patch_leave_instructions(state)
}

pub fn while_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    state.stack.push(state.memory.top().to_number());
    state.memory.push_none();
    Result::Ok(())
}

pub fn repeat_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let branch_address = pop_address!(state.memory, state.stack);

    // add a branch instruction to the beginning of the loop unconditionally
    let loop_address = pop_address!(state.memory, state.stack);
    let branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_instruction(loop_address)).value();
    state.memory.push(branch_xt);

    // back patch the conditional branch in the middle of the loop
    let conditional_branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_false_instruction(state.memory.top())).value();
    state.memory.write(branch_address, conditional_branch_xt);

    patch_leave_instructions(state)
}

pub fn patch_leave_instructions(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let leave_address_count = pop_or_underflow!(state.return_stack, generic_numbers::Number);
    // if there were any leave instructions, iterate through them patch them to jump to the end of the loop
    if leave_address_count > 0 {
        let leave_branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_instruction(state.memory.top())).value();

        for _ in 0..leave_address_count {
            let leave_address = hard_match_address!(state.memory, pop_or_underflow!(state.return_stack, generic_numbers::Number));
            state.memory.write(leave_address, leave_branch_xt);
        }
    }
    Result::Ok(())
}

pub fn leave(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let leave_address_count = pop_or_underflow!(state.return_stack, generic_numbers::Number) + 1;
    state.return_stack.push(state.memory.top().to_number());
    state.return_stack.push(leave_address_count);
    state.memory.push_none();
    Result::Ok(())
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