// use super::evaluate;
// use super::tokens;
// use super::get_token;


// pub fn compile(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
//     let name = match get_token!(state) {
//         tokens::Token::Name(name) => name,
//         _ => return Result::Err(evaluate::Error::InvalidWord)
//     };

//     let address = state.memory.top();
//     let execution_token = state.compiled_code.add_compiled_code(Box::new(move |state| {
//         state.execute_at(address).map(|_| evaluate::ControlFlowState::Continue)
//     }));

//     // we will edit the definition to be immediate if the IMMEDIATE keyword is found
//     state.definitions.add(name, evaluate::Definition::new(execution_token, false));

//     loop {
//         let token = get_token!(state);
//         let definition = match state.definitions.get_from_token(token) {
//             Some(definition) => definition,
//             None => return Result::Err(evaluate::Error::UnknownWord)
//         };

//         if definition.immediate {
//             match state.execute(definition.execution_token) {
//                 Result::Ok(evaluate::ControlFlowState::Continue) => (),
//                 Result::Ok(_) => break Result::Ok(evaluate::ControlFlowState::Continue),
//                 Result::Err(error) => return Result::Err(error)
//             }
//         } else {
//             state.memory.push(definition.execution_token.value());
//         }
//     }
// }

// fn compile_locals(state: &mut evaluate::ForthEvaluator) -> HashMap<String, evaluate::Definition> {
//     let mut locals = HashMap::new();
//     loop {
//         let name = match get_token!(state) {
//             tokens::Token::Name(name) if name == "|" => return locals,
//             tokens::Token::Name(name) => name,
//             _ => return Result::Err(evaluate::Error::InvalidWord)
//         };

//         state.memory.push(state.get_execution_token_from_name(">R").value());

//         let pop_local_xt = state.compiled_code.add_compiled_code(Box::new(move |state| {
//             state.stack.push(state.return_stack.get_from_frame(locals.len()));
//             Result::Ok(evaluate::ControlFlowState::Continue)
//         }));

//         locals.insert(name, evaluate::Definition::new(pop_local_xt, false));
//     }
// }
