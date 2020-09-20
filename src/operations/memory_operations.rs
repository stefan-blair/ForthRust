use super::*;


pub fn dereference<N: value::ValueVariant>(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let address = state.stack.pop()?;
    state.stack.push(state.read::<N>(address)?);
    Ok(())
}

pub fn memory_write<N: value::ValueVariant>(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let (address, value) = (state.stack.pop()?, state.stack.pop::<N>()?);

    state.write(address, value)
}

pub fn pop_write(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    state.heap.push(state.stack.pop::<value::Value>()?);
    Ok(())
}

macro_rules! generic_operations {
    ($pre:tt, $type:ty) => {
        vec![
            (concat!($pre, "!") , false, memory_write::<$type> as super::Operation),
            (concat!($pre, "@"), false, dereference::<$type>)
        ]
    };
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    let mut operations: Vec<(&'static str, bool, super::Operation)> = vec![
        (",", false, pop_write),
    ];

    operations.append(&mut generic_operations!("", value::Value));
    operations.append(&mut generic_operations!("C", generic_numbers::Byte));
    operations.append(&mut generic_operations!("2", value::DoubleValue));

    operations
}