use crate::environment::{value, memory, generic_numbers, generic_numbers::GenericNumber, generic_numbers::SignedGenericNumber, units::{Bytes, Cells}};
use crate::environment::memory::MemorySegment;
use crate::evaluate::{self, ForthResult, ForthState};

pub mod control_flow_operations;
mod arithmetic_operations;
mod compiler_control_operations;
mod data_operations;
mod memory_operations;
mod print_operations;
mod stack_operations;
mod string_operations;

// import all of the macros exposed by this module for ease of use by the other operations modules
use crate::postpone;
use crate::maybe;


// a set of macros to help simplify the operations
mod helper_macros {
    /**
     * Macro that wraps operations to make them into maybe versions (?), which only operate if the top of the stack is nonzero.
     */
    #[macro_export]
    macro_rules! maybe {
        ($v:expr) => {
            |state: &mut evaluate::ForthState| if state.stack.peek::<value::Value>()?.to_number() > 0 {
                $v(state)
            } else {
                Ok(())                
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
            $state.data_space.push(evaluate::definition::ExecutionToken::LeafOperation($execution_token).value());
        };
    }
}

/**
 * Helper mod for tokens whose operations require them to scan ahead for some closing token.  For example, the comment
 * ( .... ), requires to scan ahead to the next ')' character.  This mod contains structs that allow those end characters
 * to be specified at compile time via traits.
 */
mod closing_tokens {
    
    pub trait ClosingToken {
        const CLOSING_TOKEN: char;
    }

    macro_rules! closing_token {
        ($name:tt, $closing_token:expr) => {
            pub struct $name;

            impl ClosingToken for $name {
                const CLOSING_TOKEN: char = $closing_token;
            }
        }
    }

    closing_token!(CurlyBracket, '}');
    closing_token!(NewLine, '\n');
    closing_token!(Parenthesis, ')');
    closing_token!(Pipe, '|');
}

// built in operators; name, whether its immediate or not, and the function to execute
pub type Operation = fn(&mut evaluate::ForthState) -> evaluate::ForthResult;
pub type OperationTable = Vec<(&'static str, bool, Operation)>;

/**
 * This is the main function that this module provides.  It takes all of the operations defined in each submodule,
 * and compiles them into one vector.  
 */
pub fn get_operations() -> OperationTable {
    vec![
        arithmetic_operations::get_operations(),
        memory_operations::get_operations(),
        stack_operations::get_operations(),
        data_operations::get_operations(),
        control_flow_operations::get_operations(),
        compiler_control_operations::get_operations(),
        print_operations::get_operations(),
        string_operations::get_operations(),
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
    ": ELSE POSTPONE 0 HERE 1 ALLOT SWAP HERE _BNE ; IMMEDIATE",
    ": THEN HERE _BNE ; IMMEDIATE",
    // get current index of do ... loop
    ": I R> R> R@ SWAP >R SWAP >R ;",

    // get next character
    ": [CHAR] CHAR POSTPONE LITERAL ; IMMEDIATE",
    // some increment instructions
    ": CELL+ 1 CELLS + ;",
    ": 1+ 1 + ;"
];