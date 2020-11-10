use super::*;
use crate::io::tokens;
use crate::evaluate::definition;


pub fn immedate(state: &mut ForthState) -> ForthResult { state.definitions.make_most_recent_immediate(); Result::Ok(()) }
pub fn set_interpret(state: &mut ForthState) -> ForthResult { state.set_interpretmode() }
pub fn set_compile(state: &mut ForthState) -> ForthResult { state.set_compilemode() }

pub fn start_word_compilation(state: &mut ForthState) -> ForthResult {
    let word = state.input_stream.next_word()?;
    let execution_token = evaluate::definition::ExecutionToken::Definition(state.data_space.top());

    // the IMMEDIATE keyword will edit the definition to be immediate
    state.definitions.add(word, evaluate::definition::Definition::new(execution_token, false));

    set_compile(state)
}

pub fn end_word_compilation(state: &mut ForthState) -> ForthResult {
    postpone!(state, super::control_flow_operations::control_flow_break);
    // clear any declared temp values
    state.definitions.clear_temp();
    set_interpret(state)
}

pub fn postpone(state: &mut ForthState) -> ForthResult {
    let definition = state.definitions.get_from_token(state.input_stream.next()?)?;

    state.data_space.push(
        if definition.immediate {
            definition.execution_token
        } else {
            state.compiled_instructions.compiler().mem_push(definition.execution_token.value())
        }
    );

    Ok(())
}

pub fn literal<N: value::ValueVariant + 'static>(state: &mut ForthState) -> ForthResult {
    let value = state.stack.pop::<N>()?;
    state.data_space.push(state.compiled_instructions.compiler().push(value));

    Ok(())
}

pub fn execute(state: &mut ForthState) -> ForthResult {
    let execution_token = state.stack.pop::<evaluate::definition::ExecutionToken>()?;
    state.execute(execution_token)
}

// read the next token from the input stream
pub fn read_execution_token(state: &mut ForthState) -> ForthResult {
    state.input_stream.next()
        .and_then(|token| state.definitions.get_from_token(token))
        .map(|definition| state.stack.push(definition.execution_token))       
}

pub fn get_execution_token(state: &mut ForthState) -> ForthResult {
    state.input_stream.next()
        .and_then(|token| state.definitions.get_from_token(token))
        .map(|definition| state.data_space.push(state.compiled_instructions.compiler().push(definition.execution_token.value())))
    }

/**
 * Generates a bne instruction.  Pops an address off of the stack to be the destination for the branch.
 * Pushes the execution token of this branch instruction onto the stack.
 */
pub fn write_branch_false(state: &mut ForthState) -> ForthResult {
    let branch_target = state.stack.pop()?;
    let destination = state.stack.pop()?;
    let xt = state.compiled_instructions.compiler().branch_false(branch_target);
    state.write(destination, xt)
}

pub fn write_branch(state: &mut ForthState) -> ForthResult {
    let branch_target = state.stack.pop()?;
    let destination = state.stack.pop()?;
    let xt = state.compiled_instructions.compiler().branch(branch_target);
    state.write(destination, xt)
}

pub fn body(state: &mut ForthState) -> ForthResult {
    let xt = state.stack.pop()?;
    match xt {
        evaluate::definition::ExecutionToken::Definition(address) => state.stack.push(address),
        evaluate::definition::ExecutionToken::Number(i) => state.stack.push(i),
        _ => state.stack.push(xt)
    };

    Ok(())
}

pub fn absorb_comment<T: closing_tokens::ClosingToken>(state: &mut ForthState) -> ForthResult {
    while let Ok(c) = state.input_stream.next_char() {
        if c == T::CLOSING_TOKEN {
            return Result::Ok(());
        }
    }

    Err(evaluate::Error::NoMoreTokens)
}

pub fn locals<T: closing_tokens::ClosingToken>(state: &mut ForthState) -> ForthResult {
    // read in all of the locals
    let mut local_names = Vec::new();
    loop {
        let name = match state.input_stream.next()? {
            tokens::Token::Word(name) => name,
            _ => return Err(evaluate::Error::InvalidWord)
        };

        // reached the end of the list of locals
        if name.len() == 1 && name.chars().next().unwrap() == T::CLOSING_TOKEN {
            break
        }
        
        local_names.push(name);
    }

    // add a leaf operation to transfer stack values to the return stack
    state.data_space.push(definition::ExecutionToken::Number(local_names.len() as generic_numbers::Number));
    state.data_space.push(definition::ExecutionToken::LeafOperation(|state| {
        // pop the number of locals, and move that number of values from the stack to the return stack
        let num_locals = state.stack.pop::<generic_numbers::Number>()?;
        for _ in 0..num_locals {
            stack_operations::stack_to_return_stack::<value::Value>(state)?;
        }

        Ok(())
    }));

    // add a jump over the stubs, so they aren't accidentally executed
    let jmp_addr = state.data_space.top();
    state.data_space.push_none::<definition::ExecutionToken>();

    // add the stubs to resolve locals
    for (offset, name) in local_names.into_iter().enumerate() {
        // add the stub to the temporary definitions, so that during the parsing of this function, the locals are correctly resolved
        let stub_address = state.data_space.top();
        state.definitions.add_temp(name, definition::Definition::new(definition::ExecutionToken::Definition(stub_address), false));
        // define the stub, starting with the offset into the return stack frame
        state.data_space.push(definition::ExecutionToken::Number(offset as generic_numbers::Number));
        state.data_space.push(definition::ExecutionToken::LeafOperation(|state| {
            state.return_from()?;

            // sort of cheating here, but because we just returned, we can operate on the previous stack frame, which is the one we care about
            let offset = state.stack.pop()?;
            let value = state.return_stack.read_from_frame::<value::Value>(offset)?;
            state.stack.push(value);
            Ok(())
        }));
    }

    let jmp_destination = state.data_space.top();
    state.data_space.write(jmp_addr, state.compiled_instructions.compiler().relative_branch(jmp_addr, jmp_destination))
}

pub fn execution_mode_address(state: &mut ForthState) -> ForthResult {
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
        ("(", true, absorb_comment::<closing_tokens::Parenthesis>),
        ("\\", true, absorb_comment::<closing_tokens::NewLine>),
        ("STATE", false, execution_mode_address),
        // branch generators
        ("_BNE", false, write_branch_false),
        ("_B", false, write_branch),
        // locals
        ("LOCALS|", true, locals::<closing_tokens::Pipe>),
        ("{", true, locals::<closing_tokens::CurlyBracket>)
    ]
}

mod test {
    #[cfg(test)]
    use crate::{Forth, Number, stack};


    #[cfg(test)]
    fn stack_to_vec(stack: &stack::Stack) -> Vec<Number> {
        stack.to_vec().iter().map(|x| x.to_number()).collect::<Vec<_>>()
    }
    
    #[test]
    fn locals_test() {
        let mut f = Forth::default();
        assert!(f.evaluate_string(": testing locals| a b c | b c a ;").is_ok());
        assert!(f.evaluate_string("1 2 3 testing 6 5 4 testing 10 1 34 testing").is_ok());
        assert_eq!(vec![2, 1, 3, 5, 6, 4, 1, 10, 34], stack_to_vec(&mut f.state.stack));    
    }

    #[test]
    fn set_locals_test() {
        let mut f = Forth::default();
        assert!(f.evaluate_string(": testing locals| r y | r y 10 to r r ;").is_ok());
        assert!(f.evaluate_string("5 6 testing").is_ok());
        assert_eq!(vec![6, 5, 10], stack_to_vec(&mut f.state.stack));    
    }
}