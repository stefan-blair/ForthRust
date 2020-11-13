use super::*;


// return stack commands
pub fn stack_to_return_stack<T: value::ValueVariant>(state: &mut ForthState) -> ForthResult { 
    state.return_stack.push(state.stack.pop::<T>()?); 
    Ok(()) 
}

pub fn return_stack_to_stack<T: value::ValueVariant>(state: &mut ForthState) -> ForthResult { 
    state.stack.push(state.return_stack.pop::<T>()?); 
    Ok(()) 
}

pub fn copy_from_return_stack<T: value::ValueVariant>(state: &mut ForthState) -> ForthResult { 
    state.stack.push(state.return_stack.peek::<T>()?); 
    Ok(()) 
}

pub fn push_stack_frame(state: &mut ForthState) -> ForthResult {
    state.return_stack.push_frame();
    Ok(())
}

pub fn pop_stack_frame(state: &mut ForthState) -> ForthResult {
    state.return_stack.pop_frame()?;
    Ok(())
}

// argument stack commands
pub fn dup<T: value::ValueVariant>(state: &mut ForthState) -> ForthResult { 
    let value = state.stack.peek::<T>()?;
    state.stack.push(value); 
    Ok(()) 
}
pub fn drop<T: value::ValueVariant>(state: &mut ForthState) -> ForthResult { state.stack.pop::<T>()?; Ok(()) }
pub fn swap<T: value::ValueVariant>(state: &mut ForthState) -> ForthResult { 
    let (a, b): (T, T) = (state.stack.pop()?, state.stack.pop()?);
    state.stack.push(a);
    state.stack.push(b);
    Ok(())
}
pub fn over<T: value::ValueVariant>(state: &mut ForthState) -> ForthResult { 
    let (a, b): (T, T) = (state.stack.pop()?, state.stack.pop()?);
    state.stack.push(b);
    state.stack.push(a);
    state.stack.push(b);
    Ok(())
}
pub fn rot<T: value::ValueVariant>(state: &mut ForthState) -> ForthResult { 
    let (a, b, c): (T, T, T) = (state.stack.pop()?, state.stack.pop()?, state.stack.pop()?);
    state.stack.push(b);
    state.stack.push(a);
    state.stack.push(c);
    Ok(())
}
pub fn nrot<T: value::ValueVariant>(state: &mut ForthState) -> ForthResult {
    let (a, b, c): (T, T, T) = (state.stack.pop()?, state.stack.pop()?, state.stack.pop()?);
    state.stack.push(c);
    state.stack.push(a);
    state.stack.push(b);
    Ok(())
}
pub fn nip<T: value::ValueVariant>(state: &mut ForthState) -> ForthResult {
    let (a, _): (T, T) = (state.stack.pop()?, state.stack.pop()?);
    state.stack.push(a);
    Ok(())
}
pub fn tuck<T: value::ValueVariant>(state: &mut ForthState) -> ForthResult {
    let (a, b): (T, T) = (state.stack.pop()?, state.stack.pop()?);
    state.stack.push(a);
    state.stack.push(b);
    state.stack.push(a);
    Ok(())
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
    operations.append(&mut vec![
        ("PUSH_FRAME", false, push_stack_frame),
        ("POP_FRAME", false, pop_stack_frame)
    ]);

    operations
}