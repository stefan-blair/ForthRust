use crate::environment::{value, memory, generic_numbers, generic_numbers::GenericNumber, generic_numbers::SignedGenericNumber};
use crate::evaluate;
use crate::io;

mod arithmetic_operations;
mod compiler_control_operations;
mod control_flow_operations;
mod data_operations;
mod memory_operations;
mod print_operations;
mod stack_operations;

// import all of the macros exposed by this module for ease of use by the other operations modules
use crate::get_two_from_stack;
use crate::hard_match_address;
use crate::pop_or_underflow;
use crate::pop_address;
use crate::match_or_error;
use crate::peek_or_underflow;
use crate::get_token;
use crate::postpone;
use crate::maybe;


// a set of macros to help simplify the operations
mod helper_macros {
    #[macro_export]
    macro_rules! get_token {
        ($state:ident) => {
            match $state.input_stream.next() {
                Some(token) => token,
                None => return Result::Err(evaluate::Error::InvalidWord)
            }
        };
    }

    #[macro_export]
    macro_rules! peek_or_underflow {
        ($stack:expr, $type:ty) => {
            match $stack.peek::<$type>() {
                Some(v) => v,
                None => return Result::Err(evaluate::Error::StackUnderflow)
            }
        };
    }

    #[macro_export]
    macro_rules! pop_or_underflow {
        ($stack:expr, $type:ty) => {
            match $stack.pop::<$type>() {
                Some(v) => v,
                None => return Result::Err(evaluate::Error::StackUnderflow)
            }
        }
    }

    #[macro_export]
    macro_rules! pop_address {
        ($memory:expr, $stack:expr) => {
            hard_match_address!($memory, pop_or_underflow!($stack, generic_numbers::Number))
        };
    }

    #[macro_export]
    macro_rules! get_two_from_stack {
        ($stack:expr, $type_1:ty, $type_2:ty) => {
            (pop_or_underflow!($stack, $type_1), pop_or_underflow!($stack, $type_2))
        };
    }

    #[macro_export]
    macro_rules! match_or_error {
        ($obj:expr, $pat:pat, $suc:expr, $err:expr) => {
            match $obj {
                $pat => $suc,
                _ => return Result::Err($err)
            }
        };
    }

    #[macro_export]
    macro_rules! hard_match_address {
        ($memory:expr, $obj:expr) => {
            match_or_error!($memory.address_from($obj), Some(address), address, evaluate::Error::InvalidAddress)
        }
    }

    /**
     * Macro that wraps operations to make them into maybe versions (?), which only operate if the top of the stack is nonzero.
     */
    #[macro_export]
    macro_rules! maybe {
        ($v:expr) => {
            |state: &mut evaluate::ForthEvaluator| match state.stack.peek::<value::Value>().map(|value| value.to_number()) {
                Some(x) if x > 0 => $v(state),
                Some(_) => Result::Ok(()),
                None => Result::Err(evaluate::Error::StackUnderflow),
            }
        };
    }

    /**
     * Macro used to generically absorb any type of comment.
     */
    #[macro_export]
    macro_rules! absorb_comment {
        ($closing_brace:expr) => {
            |state| {
                while let Some(io::tokens::Token::Name(name)) = state.input_stream.next() {
                    if name == $closing_brace {
                        return Result::Ok(());
                    }
                }
            
                Result::Err(evaluate::Error::NoMoreTokens)    
            }        
        };
    }

    /**
     * Macro that implements POSTPONE, instead of executing the execution token, pushing it to memory,
     * "postponing" it to be part of the current definition.
     */
    #[macro_export]
    macro_rules! postpone {
        ($state:expr, $execution_token:expr) => {
            $state.memory.push(evaluate::definition::ExecutionToken::Operation($execution_token).value());
        };
    }
}

mod code_compiler_helpers {
    use super::*;

    pub fn create_branch_false_instruction(destination: memory::Address) -> evaluate::compiled_code::CompiledCode {
        Box::new(move |state| {
            match pop_or_underflow!(state.stack, value::Value) {
                value if value.to_number() > 0 => Result::Ok(()),
                _ => state.jump_to(destination)
            }
        })
    }

    pub fn create_branch_instruction(destination: memory::Address) -> evaluate::compiled_code::CompiledCode {
        Box::new(move |state| state.jump_to(destination))
    }

    pub fn push_value(value: value::Value) -> evaluate::compiled_code::CompiledCode {
        Box::new(move |state| {
            state.stack.push(value);
            Result::Ok(())
        })
    }
}

// built in operators; name, whether its immediate or not, and the function to execute
pub type Operation = fn(&mut evaluate::ForthEvaluator) -> evaluate::ForthResult;

/**
 * This is the main function that this module provides.  It takes all of the operations defined in each submodule,
 * and compiles them into one vector.  
 */
pub fn get_operations() -> Vec<(&'static str, bool, Operation)> {
    vec![
        arithmetic_operations::get_operations(),
        memory_operations::get_operations(),
        stack_operations::get_operations(),
        data_operations::get_operations(),
        control_flow_operations::get_operations(),
        compiler_control_operations::get_operations(),
        print_operations::get_operations(),
    ].into_iter().flatten().collect::<Vec<_>>()
}

/**
 * For the sake of demonstration, some important words, including IF ELSE THEN, are implemented
 * in FORTH instead of hardcoded.  Most of the important words can be implemented from only
 * a select few, but were hardcoded instead, for the sake of readability and maintainability.
 */
pub const UNCOMPILED_OPERATIONS: &[&str] = &[
    // if ... [else] ... then
    ": IF HERE 1 ALLOT ; IMMEDIATE",
    ": ELSE POSTPONE 0 HERE 1 ALLOT SWAP HERE _BNE SWAP ! ; IMMEDIATE",
    ": THEN HERE _BNE SWAP ! ; IMMEDIATE",
    // get current index of do ... loop
    ": I R> R@ SWAP >R ;"
];