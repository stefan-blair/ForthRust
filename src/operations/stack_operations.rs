use super::*;


// return stack commands
pub fn stack_to_return_stack(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.return_stack.push(pop_or_underflow!(state.stack)); CONTINUE_RESULT }
pub fn twice_stack_to_return_stack(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { stack_to_return_stack(state).and_then(|_| stack_to_return_stack(state)) }
pub fn return_stack_to_stack(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.stack.push(pop_or_underflow!(state.return_stack)); CONTINUE_RESULT }
pub fn copy_from_return_stack(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.stack.push(peek_or_underflow!(state.return_stack)); CONTINUE_RESULT }

// argument stack commands
pub fn dup(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.stack.push(peek_or_underflow!(state.stack)); CONTINUE_RESULT }
pub fn drop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { pop_or_underflow!(state.stack); CONTINUE_RESULT }
pub fn rdrop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { pop_or_underflow!(state.return_stack); CONTINUE_RESULT }
pub fn swap(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { 
    let (a, b) = get_two_from_stack!(&mut state.stack);
    state.stack.push(a);
    state.stack.push(b);
    CONTINUE_RESULT
}
pub fn over(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { 
    let (a, b) = get_two_from_stack!(&mut state.stack);
    state.stack.push(b);
    state.stack.push(a);
    state.stack.push(b);
    CONTINUE_RESULT
}
pub fn rot(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { 
    let (a, b, c) = (pop_or_underflow!(state.stack), pop_or_underflow!(state.stack), pop_or_underflow!(state.stack));
    state.stack.push(b);
    state.stack.push(a);
    state.stack.push(c);
    CONTINUE_RESULT
}
pub fn nrot(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let (a, b, c) = (pop_or_underflow!(state.stack), pop_or_underflow!(state.stack), pop_or_underflow!(state.stack));
    state.stack.push(c);
    state.stack.push(a);
    state.stack.push(b);
    CONTINUE_RESULT
}
pub fn nip(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let (a, _) = get_two_from_stack!(&mut state.stack);
    state.stack.push(a);
    CONTINUE_RESULT
}
pub fn tuck(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    let (a, b) = get_two_from_stack!(&mut state.stack);
    state.stack.push(a);
    state.stack.push(b);
    state.stack.push(a);
    CONTINUE_RESULT
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    vec![
        (">R", false, stack_to_return_stack),
        ("2>R", false, twice_stack_to_return_stack),
        ("R>", false, return_stack_to_stack),
        ("R@", false, copy_from_return_stack),
        ("DUP", false, dup),
        ("?DUP", false, maybe!(dup)),
        ("DROP", false, drop),
        ("SWAP", false, swap),
        ("OVER", false, over),
        ("ROT", false, rot),
        ("-ROT", false, nrot),
        ("NIP", false, nip),
        ("TUCK", false, tuck)
    ]
}