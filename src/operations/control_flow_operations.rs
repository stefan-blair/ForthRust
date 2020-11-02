use super::*;

pub fn control_flow_break(state: &mut evaluate::ForthState) -> evaluate::ForthResult { state.return_from() }

pub fn do_init_loop(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    // make room for an instruction to be patched in later that will push the end address of the loop onto the stack, for use by leave instructions 
    state.data_space.push_none::<value::Value>();
    // add an instruction to move the address from the stack to the return stack
    postpone!(state, stack_operations::stack_to_return_stack::<value::Value>);
    // add instructions to the current definition that initialize the loop by moving the bounds onto the return stack
    postpone!(state, stack_operations::stack_to_return_stack::<value::DoubleValue>);

    state.stack.push(state.data_space.top().to_number());

    Result::Ok(())
}

pub fn loop_plus_compiletime(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    // push the loop runtime
    state.data_space.push(evaluate::definition::ExecutionToken::LeafOperation(|state| {
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
    state.data_space.push(state.compiled_instructions.compiler().branch_false(loop_address));

    // add an epilogue to pop the state off of the return stack
    state.data_space.push(evaluate::definition::ExecutionToken::LeafOperation(|state| {
        // pop the start and end values
        state.return_stack.pop::<value::DoubleValue>()?;
        // pop the leave address
        state.return_stack.pop::<value::Value>()?;
        Result::Ok(())
    }).value());
    

    // fill in the blank space at the beginning of the loop with the address of the end of the loop so that it gets pushed onto the stack for leave instructions
    state.write(loop_address.minus_cell(Cells::cells(3)), evaluate::definition::ExecutionToken::Number(state.data_space.top().to_number()))
}

pub fn loop_compiletime(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    // postpone pushing 1 onto the stack, which is the expected step value on the stack (+LOOP has an explicit step)
    state.data_space.push(evaluate::definition::ExecutionToken::Number(1).value());
    loop_plus_compiletime(state)
}

pub fn begin_loop(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    // leave room for the leave instruction
    state.data_space.push_none::<value::Value>();
    postpone!(state, stack_operations::stack_to_return_stack::<value::Value>);

    state.stack.push(state.data_space.top().to_number());
    Result::Ok(())
}

pub fn until_loop(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let loop_address = state.stack.pop()?;
    state.data_space.push(state.compiled_instructions.compiler().branch_false(loop_address));
    // fill in the blank space at the beginning of the loop with the address of the end of the loop so that it gets pushed onto the stack for leave instructions
    state.write(loop_address.minus_cell(Cells::cells(2)), evaluate::definition::ExecutionToken::Number(state.data_space.top().to_number()))
}

pub fn again_loop(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let loop_address = state.stack.pop()?;
    state.data_space.push(state.compiled_instructions.compiler().branch(loop_address));

    // fill in the blank space at the beginning of the loop with the address of the end of the loop so that it gets pushed onto the stack for leave instructions
    state.write(loop_address.minus_cell(Cells::cells(2)), evaluate::definition::ExecutionToken::Number(state.data_space.top().to_number()))
}

pub fn while_loop(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    state.stack.push(state.data_space.top().to_number());
    state.data_space.push_none::<value::Value>();
    Result::Ok(())
}

pub fn repeat_loop(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let branch_address = state.stack.pop()?;

    // add a branch instruction to the beginning of the loop unconditionally
    let loop_start_address = state.stack.pop()?;
    state.data_space.push(state.compiled_instructions.compiler().branch(loop_start_address));

    // back patch the conditional branch in the middle of the loop
    let loop_middle_address = state.data_space.top();
    state.data_space.write(branch_address, state.compiled_instructions.compiler().branch_false(loop_middle_address))?;

    // fill in the blank space at the beginning of the loop with the address of the end of the loop so that it gets pushed onto the stack for leave instructions
    state.write(loop_start_address.minus_cell(Cells::cells(2)), evaluate::definition::ExecutionToken::Number(state.data_space.top().to_number()))
}

pub fn leave(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    state.return_stack.pop::<value::DoubleValue>()?;
    let end_of_loop_address = state.return_stack.pop()?;
    state.jump_to(end_of_loop_address)
}

pub fn exit(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    state.return_from()
}

pub fn throw(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let error_code = state.stack.pop::<generic_numbers::UnsignedNumber>()?;
    Err(evaluate::Error::Exception(error_code))
}

pub fn evaluate_string(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let length: usize = state.stack.pop()?;
    let address: memory::Address = state.stack.pop()?;

    // read the characters into a vector
    let mut copied_string = Vec::new();
    for i in (0..length).map(|i| Bytes::from(i)) {
        copied_string.push(state.read::<generic_numbers::UnsignedByte>(address.plus(i))? as char);
    }
    // convert that vector into an into_iter, which takes ownership of it, and prepend it to the current input stream
    state.input_stream.prepend_stream(copied_string.into_iter());

    Ok(())
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
        ("EXIT", false, exit),
        ("THROW", false, throw),
        ("EVALUATE", false, evaluate_string),
    ]
}