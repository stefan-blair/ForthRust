use super::*;

use crate::postpone;
use evaluate::definition;


pub fn here(state: &mut evaluate::ForthState) -> evaluate::ForthResult { state.stack.push(state.memory.top().to_number()); Ok(()) }

pub fn allot(state: &mut evaluate::ForthState) -> evaluate::ForthResult { 
    let total_memory = state.stack.pop::<generic_numbers::UnsignedNumber>()? as memory::Offset;
    let cells = (total_memory + memory::CELL_SIZE - 1) / memory::CELL_SIZE;
    state.memory.expand(cells as memory::Offset); 
    Ok(()) 
}

pub fn create(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let word = state.input_stream.next_word()?;

    let address = state.memory.top().plus_cell(3);
    let xt = definition::ExecutionToken::Definition(state.memory.top());
    state.memory.push(definition::ExecutionToken::Number(address.to_number()));
    postpone!(state, super::control_flow_operations::control_flow_break);
    postpone!(state, super::control_flow_operations::control_flow_break);
    state.definitions.add(word, definition::Definition::new(xt, false));

    Ok(())
}

pub fn variable<N: value::ValueVariant>(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    create(state).and_then(|_| Ok(state.memory.push_none::<N>()))
}

pub fn constant<N: value::ValueVariant>(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let word = state.input_stream.next_word()?;

    let address = state.memory.top();
    state.memory.push(definition::ExecutionToken::LeafOperation(|state| {
        let value: N = state.memory.read(state.instruction_pointer.unwrap())?;
        state.stack.push(value);
        state.return_from()
    }));
    state.memory.push(state.stack.pop::<N>()?);
    let xt = definition::ExecutionToken::Definition(address);
    state.definitions.add(word, definition::Definition::new(xt, false));

    Ok(())
}

pub fn does(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let object_address = if let definition::ExecutionToken::Definition(address) = state.definitions.get(state.definitions.get_most_recent_nametag()).execution_token {
        address
    } else {
        return Ok(())
    };

    state.memory.write(object_address.plus_cell(1), definition::ExecutionToken::Definition(state.instruction_pointer.unwrap()))?;

    // add a manual break, so that normal calls to the function wont execute the rest of the code, only created objects
    state.return_from()
}

pub fn value(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let word = state.input_stream.next_word()?;

    let number = state.stack.pop::<generic_numbers::Number>()?;

    state.definitions.add(word, definition::Definition::new(definition::ExecutionToken::Number(number), false));
    Ok(())
}

pub fn cells(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let number = state.stack.pop::<generic_numbers::UnsignedNumber>()? as memory::Offset;
    state.stack.push((number * memory::CELL_SIZE) as generic_numbers::UnsignedNumber);
    Ok(())
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    vec![
        ("HERE", false, here),
        ("ALLOT", false, allot),
        ("CREATE", false, create),
        ("DOES>", false, does),
        ("VALUE", false, value),
        ("CELLS", false, cells),

        ("VARIABLE" , false, variable::<value::Value>),
        ("CONSTANT", false, constant::<value::Value>),
        ("2VARIABLE" , false, variable::<value::DoubleValue>),
        ("2CONSTANT", false, constant::<value::DoubleValue>)
    ]
}
