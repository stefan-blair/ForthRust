use super::*;

use crate::postpone;
use evaluate::definition;


pub fn here(state: &mut evaluate::ForthState) -> evaluate::ForthResult { state.stack.push(state.data_space.top().to_number()); Ok(()) }

pub fn allot(state: &mut evaluate::ForthState) -> evaluate::ForthResult { 
    state.data_space.expand(state.stack.pop::<Bytes>()?.to_cells()); 
    Ok(()) 
}

pub fn create(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let word = state.input_stream.next_word()?;

    let address = state.data_space.top().plus_cell(Cells::cells(3));
    let xt = definition::ExecutionToken::Definition(state.data_space.top());
    state.data_space.push(definition::ExecutionToken::Number(address.to_number()));
    postpone!(state, super::control_flow_operations::control_flow_break);
    postpone!(state, super::control_flow_operations::control_flow_break);
    state.definitions.add(word, definition::Definition::new(xt, false));

    Ok(())
}

pub fn variable<N: value::ValueVariant>(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    create(state).and_then(|_| Ok(state.data_space.push_none::<N>()))
}

pub fn constant<N: value::ValueVariant>(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let word = state.input_stream.next_word()?;

    let address = state.data_space.top();
    state.data_space.push(definition::ExecutionToken::LeafOperation(|state| {
        let value: N = state.read(state.instruction_pointer().unwrap())?;
        state.stack.push(value);
        state.return_from()
    }));
    state.data_space.push(state.stack.pop::<N>()?);
    let xt = definition::ExecutionToken::Definition(address);
    state.definitions.add(word, definition::Definition::new(xt, false));

    Ok(())
}

pub fn does(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let object_address = if let definition::ExecutionToken::Definition(address) = state.definitions.most_recent_definition().execution_token {
        address
    } else {
        return Ok(())
    };

    state.write(object_address.plus_cell(Cells::one()), definition::ExecutionToken::Definition(state.instruction_pointer().unwrap()))?;

    // add a manual break, so that normal calls to the function wont execute the rest of the code, only created objects
    state.return_from()
}

pub fn value(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let word = state.input_stream.next_word()?;

    let v = state.stack.pop::<value::Value>()?;
    let xt = state.compiled_instructions.compiler().push(v);
    state.definitions.add(word, definition::Definition::new(xt, false));

    Ok(())
}

pub fn to(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let word = state.input_stream.next_word()?;
    let nametag = state.definitions.get_nametag(&word)?;

    match nametag {
        evaluate::definition::NameTag::Definition(n) => {
            // push the definition's index
            state.data_space.push(evaluate::definition::ExecutionToken::Number(n as generic_numbers::Number));
            // push the operating code
            state.data_space.push(evaluate::definition::ExecutionToken::LeafOperation(|state| {
                let index = state.stack.pop()?;
                let v = state.stack.pop::<value::Value>()?;
                let xt = state.compiled_instructions.compiler().push(v);

                state.definitions.set_by_index(index, evaluate::definition::Definition::new(xt, false))
            }));        
        },
        evaluate::definition::NameTag::TempDefinition(n) => {
            if let definition::ExecutionToken::Definition(address) = state.definitions.get_temp_by_index(n)?.execution_token {
                if let definition::ExecutionToken::Number(offset) = state.data_space.read(address)? {
                    state.data_space.push(evaluate::definition::ExecutionToken::Number(offset));
                    state.data_space.push(evaluate::definition::ExecutionToken::LeafOperation(|state| {
                        let offset = state.stack.pop()?;
                        let v = state.stack.pop::<value::Value>()?;
                        
                        state.return_stack.write_to_frame(offset, v)?;
                        Ok(())
                    }))
                }                
            }
        }
    }

    Ok(())
}

pub fn cells(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let cells = state.stack.pop::<Cells>()?;
    state.stack.push(cells.to_bytes());
    Ok(())
}

pub fn map_anonymous(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let num_pages = state.stack.pop()?;
    let address = state.create_anonymous_mapping(num_pages)?;
    state.stack.push(address);
    Ok(())
}

pub fn allocate(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    match state.heap.allocate(state.stack.pop::<Bytes>()?) {
        Ok(address) => {
            state.stack.push(address);
            state.stack.push(0 as generic_numbers::Number);
        },
        Err(_) => {
            state.stack.push(0 as generic_numbers::Number);
            state.stack.push(-1 as generic_numbers::Number);
        }
    }

    Ok(())
}

pub fn free(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let address: memory::Address = state.stack.pop()?;
    match state.heap.free(address) {
        Ok(_) => state.stack.push(0 as generic_numbers::Number),
        Err(_) => state.stack.push(-1 as generic_numbers::Number),
    }

    Ok(())
}

pub fn resize(state: &mut evaluate::ForthState) -> evaluate::ForthResult {
    let new_size = state.stack.pop::<Bytes>()?;
    let address: memory::Address = state.stack.pop()?;
    match state.heap.resize(address, new_size) {
        Ok(new_address) => {
            state.stack.push(new_address);
            state.stack.push(0 as generic_numbers::Number);
        },
        Err(_) => {
            state.stack.push(0 as generic_numbers::Number);
            state.stack.push(-1 as generic_numbers::Number);
        }
    }

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
        ("TO", true, to),
        ("MAP", false, map_anonymous), 

        // heap instructions
        ("ALLOCATE", false, allocate),
        ("FREE", false, free),
        ("RESIZE", false, resize),        

        ("VARIABLE" , false, variable::<value::Value>),
        ("CONSTANT", false, constant::<value::Value>),
        ("2VARIABLE" , false, variable::<value::DoubleValue>),
        ("2CONSTANT", false, constant::<value::DoubleValue>)
    ]
}
