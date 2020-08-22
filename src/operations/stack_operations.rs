use super::*;


// return stack commands
pub fn stack_to_return_stack<T: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { state.return_stack.push(pop_or_underflow!(state.stack, T)); Result::Ok(()) }
pub fn return_stack_to_stack<T: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { state.stack.push(pop_or_underflow!(state.return_stack, T)); Result::Ok(()) }
pub fn copy_from_return_stack<T: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { state.stack.push(peek_or_underflow!(state.return_stack, T)); Result::Ok(()) }

// argument stack commands
pub fn dup<T: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { 
    let value = peek_or_underflow!(state.stack, T);
    state.stack.push(value); 
    Result::Ok(()) 
}
pub fn drop<T: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { pop_or_underflow!(state.stack, T); Result::Ok(()) }
pub fn swap<T: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { 
    let (a, b) = get_two_from_stack!(&mut state.stack, T, T);
    state.stack.push(a);
    state.stack.push(b);
    Result::Ok(())
}
pub fn over<T: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { 
    let (a, b) = get_two_from_stack!(&mut state.stack, T, T);
    state.stack.push(b);
    state.stack.push(a);
    state.stack.push(b);
    Result::Ok(())
}
pub fn rot<T: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult { 
    let (a, b, c) = (pop_or_underflow!(state.stack, T), pop_or_underflow!(state.stack, T), pop_or_underflow!(state.stack, T));
    state.stack.push(b);
    state.stack.push(a);
    state.stack.push(c);
    Result::Ok(())
}
pub fn nrot<T: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let (a, b, c) = (pop_or_underflow!(state.stack, T), pop_or_underflow!(state.stack, T), pop_or_underflow!(state.stack, T));
    state.stack.push(c);
    state.stack.push(a);
    state.stack.push(b);
    Result::Ok(())
}
pub fn nip<T: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let (a, _) = get_two_from_stack!(&mut state.stack, T, T);
    state.stack.push(a);
    Result::Ok(())
}
pub fn tuck<T: value::ValueVariant>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    let (a, b) = get_two_from_stack!(&mut state.stack, T, T);
    state.stack.push(a);
    state.stack.push(b);
    state.stack.push(a);
    Result::Ok(())
}

macro_rules! stack_operations {
    ($pre:tt, $type:ty) => {
        vec![
            (concat!($pre,">R"), false, stack_to_return_stack::<$type> as super::Operation),
            (concat!($pre,"R>"), false, return_stack_to_stack::<$type>),
            (concat!($pre,"R@"), false, copy_from_return_stack::<$type>),
            (concat!($pre,"DUP"), false, dup::<$type>),
            (concat!($pre,"?DUP"), false, maybe!(dup::<$type>)),
            (concat!($pre,"DROP"), false, drop::<$type>),
            (concat!($pre,"SWAP"), false, swap::<$type>),
            (concat!($pre,"OVER"), false, over::<$type>),
            (concat!($pre,"ROT"), false, rot::<$type>),
            (concat!($pre,"-ROT"), false, nrot::<$type>),
            (concat!($pre,"NIP"), false, nip::<$type>),
            (concat!($pre,"TUCK"), false, tuck::<$type>),
        ]
    };
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    let mut operations = stack_operations!("", value::Value);
    operations.append(&mut stack_operations!("2", value::DoubleValue));

    operations
}