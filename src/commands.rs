/*
use std::collections::hash_map;
use std::mem;

use super::tokens;
use super::evaluate;
use super::memory;


/**
 * This helper macro allows for the unwrapping of an enum, where the variant of that enum is known for certain
 * beforehand.  It matches the enum and calls the statement after, panicing if the match failed
 */
macro_rules! enum_unwrap {
    ($v:expr, $p:pat, $b:stmt) => {
        if let $p = $v {
            $b
        } else {
            panic!();
        }
    };
}

/**
 * Simple macro that wraps up a given closure into a Command type
 */
macro_rules! new_command {
    ($v:expr) => {
        Box::new($v)
    }
}


pub type Command = Box<dyn Fn(&mut evaluate::ForthRuntime) -> evaluate::ForthResult>;

pub type CommandSequence = Vec<Command>;

fn control_flow_parse(parsed_tokens: Vec<parse::ParsedTokens>) -> Vec<CommandSequence> {
    parsed_tokens.into_iter().filter_map(|x| match x {
        parse::ParsedTokens::Sequence(command_sequence) => Some(command_sequence),
        _ => None
    }).collect::<Vec<_>>()
}

/**
 * Rules:
 * - a command cannot end with what another command starts with
 * - a command cannot start with or end with a sequence
 * - no possibility for adjacent Sequences
 * - an optional or repeat statement cannot start with the same expected token as the one that comes after it
 */
pub const COMMANDS: &[(parse::ExpectedTokens, compile::CompileBranch)] = &[
    // implementation for Integer (1, 2, ...)
    (
        &[parse::ExpectedToken::Integer],
        |_, mut parsed_tokens| enum_unwrap!(parsed_tokens.remove(0), parse::ParsedTokens::Token(tokens::Token::Integer(i)), {
            new_command!(move |state| {
                Result::Ok(state.runtime_state.stack.push(i))
            })
        })
    ),
    // implementation for Name
    (
        &[parse::ExpectedToken::Name],
        |state, mut parsed_tokens| enum_unwrap!(parsed_tokens.remove(0), parse::ParsedTokens::Token(tokens::Token::Name(name)), {
            let opcode = state.opcode_map.get(&name).map(|i| *i);
            match opcode {
                Some(OpCode::Operation(opcode)) => new_command!(move |state| state.compilation_state.operations[opcode](&mut state.runtime_state)),
                Some(OpCode::DefinedOperation(opcode)) => new_command!(move |state| state.execute_command_sequence(&state.compilation_state.definitions[opcode])),
                _ => new_command!(move |state| {
                    let local_definitions = mem::replace(&mut state.runtime_state.local_definitions, hash_map::HashMap::new());
                    match local_definitions.get(&name) {
                        Some(command_sequence) => {
                            let result = state.execute_command_sequence(command_sequence);
                            state.runtime_state.local_definitions = local_definitions;
                            result
                        }
                        None => Result::Err(evaluate::Error::UnknownWord)
                    }
                })
            }
        })),
    // implementation for Definition " : NAME [LOCALS|...|]  ......  ;"
    (
        &[
            parse::ExpectedToken::Identifier(":"), parse::ExpectedToken::Name, 
            // optionally accept locals
            parse::ExpectedToken::Optional(&[parse::ExpectedToken::Identifier("LOCALS|"), parse::ExpectedToken::Repeat(&[parse::ExpectedToken::Name]), parse::ExpectedToken::Identifier("|")]), 
            // this checkpoint allows us to register that name before parsing the sequence, so that recursion will actually work
            parse::ExpectedToken::CheckPoint(|state, parsed_tokens| {
                let name = enum_unwrap!(parsed_tokens.get(1), Some(parse::ParsedTokens::Token(tokens::Token::Name(name))), name);
                state.opcode_map.insert(name.clone(), OpCode::DefinedOperation(state.definitions.len()));
                state.definitions.push(Vec::new());
            }),
            parse::ExpectedToken::Sequence, parse::ExpectedToken::Identifier(";")
        ],
        |state, mut parsed_tokens| {
            let mut sequence = enum_unwrap!(parsed_tokens.remove(parsed_tokens.len() - 2), parse::ParsedTokens::Sequence(sequence), sequence);
            let name = enum_unwrap!(parsed_tokens.remove(1), parse::ParsedTokens::Token(tokens::Token::Name(name)), name);

            // if there are locals, add a preamble, a simple set of commands that will fill in the local_definitions at runtime
            if enum_unwrap!(parsed_tokens.remove(1), parse::ParsedTokens::Token(tokens::Token::Name(name)), name) == "LOCALS|" {
                let len = parsed_tokens.len();
                let locals: Vec<String> = parsed_tokens.into_iter().skip(1).take(len - 3).map(|x| enum_unwrap!(x,parse::ParsedTokens::Token(tokens::Token::Name(name)), name)).collect();
                let mut preamble: CommandSequence = Vec::new();
                for local_name in locals {
                    preamble.push(new_command!(move |state| {
                        state.runtime_state.stack.pop().ok_or(evaluate::Error::StackUnderflow).map(|value| {
                            state.runtime_state.local_definitions.insert(local_name.clone(), vec![new_command!(move |state: &mut evaluate::ForthRuntime| {
                                state.runtime_state.stack.push(value);
                                Result::Ok(())
                            })]);
                        })
                    }))
                }
                preamble.append(&mut sequence);
                sequence = preamble;
            }

            match state.opcode_map.get(&name) {
                Some(OpCode::DefinedOperation(opcode)) => state.definitions[*opcode] = sequence,
                _ => ()
            }

            new_command!(|_| Result::Ok(()))
        }
    ),
    // implementation for create statement " CREATE NAME "
    (
        &[parse::ExpectedToken::Identifier("CREATE"), parse::ExpectedToken::Delayed(&[parse::ExpectedToken::Name])],
        |_, _| new_command!(move |state| {
            state.runtime_state.memory.push();
            let address = state.runtime_state.memory.top();

            let resolve_name = Box::new(move |state: &mut compile::CompilationState, mut parsed_tokens: Vec<parse::ParsedTokens>| {
                let name = enum_unwrap!(parsed_tokens.remove(0), parse::ParsedTokens::Token(tokens::Token::Name(name)), name);
                state.opcode_map.insert(name, OpCode::DefinedOperation(state.definitions.len()));
                state.definitions.push(vec![new_command!(move |state| {
                    state.runtime_state.stack.push(evaluate::Value::Address(address));
                    Result::Ok(())
                })])
            }) as parse::DynamicCompileBranch;
            state.runtime_state.awaiting_input_stream.push((&[parse::ExpectedToken::Name], resolve_name));

            Result::Ok(())
        })
    ),
    // implementation for does> statement " DOES> ... "
    (
        &[parse::ExpectedToken::Identifier("DOES>"), parse::ExpectedToken::Sequence, parse::ExpectedToken::Identifier(";")],
        |state, mut parsed_tokens| {
            let sequence = enum_unwrap!(parsed_tokens.remove(1), parse::ParsedTokens::Sequence(sequence), sequence);
            let does_opcode_index = state.definitions.len();
            state.definitions.push(sequence);
            println!("compiling DOES> statement, sequence at {:?}", does_opcode_index);
            new_command!(move |state| {
                println!("setting up DOES>");
                let (expected_tokens, parse) = match state.runtime_state.awaiting_input_stream.pop() {
                    Some(x) => x,
                    None => {
                        println!("wtf");
                        return Result::Ok(())
                    }
                };

                let wrap_does: parse::DynamicCompileBranch = Box::new(move |state: &mut compile::CompilationState, parsed_tokens: Vec<parse::ParsedTokens>| {
                    parse(state, parsed_tokens);
                    // the last definition was just pushed on by the parse function from CREATE, append the rest of the sequence to it
                    state.definitions.last_mut().map(|s| s.push(new_command!(move |state| state.execute_command_sequence(&state.compilation_state.definitions[does_opcode_index]))));
                });

                println!("wrapped up the parse, added to awaiting_input_stream");
                state.runtime_state.awaiting_input_stream.push((expected_tokens, wrap_does));

                Result::Ok(())
            })
        }
    ),
    // (
    //     &[parse::ExpectedToken::Identifier("'")],
    //     |_, _| {
    //         let resolve_name = Box::new(move |state: &mut compile::CompilationState, mut parsed_tokens: Vec<parse::ParsedTokens>| {
    //             let name = enum_unwrap!(parsed_tokens.remove(0), parse::ParsedTokens::Token(tokens::Token::Name(name)), name);
    //         });
    //     }
    // ),
    // implementation for if then statement " IF ... [ELSE] ... THEN "
    (
        &[parse::ExpectedToken::Identifier("IF"), parse::ExpectedToken::Sequence, parse::ExpectedToken::Optional(&[parse::ExpectedToken::Identifier("ELSE"), parse::ExpectedToken::Sequence]), parse::ExpectedToken::Identifier("THEN")],
        |_, parsed_tokens| {
            let bodies = control_flow_parse(parsed_tokens);        
            new_command!(move |state| {
                match state.runtime_state.stack.pop().map(|value| value.to_number()) {
                    Some(i) if i > 0 => state.execute_command_sequence(&bodies[0]),
                    Some(_) if bodies.len() > 1 => state.execute_command_sequence(&bodies[1]),
                    Some(_) => Result::Ok(()),
                    None => Result::Err(evaluate::Error::StackUnderflow)
                }
            })
        }
    ),
    // implementation for while then statement " WHILE ... DO ... THEN "
    (
        &[parse::ExpectedToken::Identifier("WHILE"), parse::ExpectedToken::Sequence, parse::ExpectedToken::Identifier("DO"), parse::ExpectedToken::Sequence, parse::ExpectedToken::Identifier("THEN")],
        |_, parsed_tokens| {
            let bodies = control_flow_parse(parsed_tokens);
            new_command!(move |state| {
                loop 
                {
                    let result = state.execute_command_sequence(&bodies[0])
                    .and_then(|_| state.runtime_state.stack.peek().map(|value| value.to_number()).ok_or(evaluate::Error::StackUnderflow))
                    .and_then(|i| if i > 0 { state.execute_command_sequence(&bodies[1]).map(|_| true) } else { Result::Ok(false) });
                    
                    match result {
                        Result::Ok(true) => continue,
                        error @ _ => return error.map(|_| ())
                    }
                }
            })
        }
    ),
    // implementation for switch then statement " SWITCH CASE _ ... CASE _ ... CASE _ ... ... THEN "
    (
        &[parse::ExpectedToken::Identifier("SWITCH"), 
            parse::ExpectedToken::Repeat(&[parse::ExpectedToken::Identifier("CASE"), parse::ExpectedToken::Integer, parse::ExpectedToken::Sequence]), 
            parse::ExpectedToken::Optional(&[parse::ExpectedToken::Identifier("DEFAULT"), parse::ExpectedToken::Sequence]), 
            parse::ExpectedToken::Identifier("THEN")],
        |_, parsed_tokens| {
            let mut cases = hash_map::HashMap::new();
            let mut default = None;

            let mut i = parsed_tokens.into_iter();
            i.next();
            while let Some(parse::ParsedTokens::Token(tokens::Token::Name(name))) = i.next() {
                match &name[..] {
                    "CASE" => {
                        let predicate = enum_unwrap!(i.next().unwrap(), parse::ParsedTokens::Token(tokens::Token::Integer(evaluate::Value::Number(i))), i);
                        let sequence = enum_unwrap!(i.next().unwrap(), parse::ParsedTokens::Sequence(sequence), sequence);
                        cases.insert(predicate, sequence);
                    }
                    "DEFAULT" => default = enum_unwrap!(i.next().unwrap(), parse::ParsedTokens::Sequence(sequence), Some(sequence)),
                    _ => break
                }
            };

            new_command!(move |state| {
                let value = match state.runtime_state.stack.pop() {
                    Some(evaluate::Value::Number(i)) => i,
                    Some(_) => return Result::Err(evaluate::Error::InvalidAddress),
                    None => return Result::Err(evaluate::Error::StackUnderflow)
                };
                cases.get(&value).or(default.as_ref()).map_or(Result::Ok(()), |sequence| state.execute_command_sequence(sequence))
            })
        }
    ),
];
*/