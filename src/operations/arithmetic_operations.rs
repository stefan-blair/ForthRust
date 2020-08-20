use std::cmp;

use super::*;
use crate::environment::stack;
use generic_numbers::{ConvertOperations};


// these are submodules of this module, only used here
use glue::Glue;
use helper_functions::{mono_operation, binary_operation, tertiary_operation};

/**
 * The arithmetic operations use a notion of "Glue" to achieve generic implementations.  A Glue trait
 * must specify an Input type and an Output type.  The Input type specifies what type is popped off of
 * the stack, as input for the operation.  For example, a DoubleNumber would pop two numbers and join
 * them.  The Output type specifies what type this input type should be converted to and treated as 
 * before passing it to the operation.  Additionally, a "glue" function must be specified to convert
 * between the two types.  If Input == Output, no real implementation is necessary.
 * For example, an operation could having Input = Number, so as to pop off a single number, but
 * Output = DoubleNumber, to avoid overflowing the input values during the operation.
 */
mod glue {
    use super::*;

    pub trait Glue {
        type Input: GenericNumber;
        type Output: GenericNumber;
        fn glue(input: Self::Input) -> Self::Output;
    }

    macro_rules! create_glue {
        ($name: ident, $input:ty, $output:ty, $body:expr) => {
            pub struct $name;
            impl Glue for $name {
                type Input = $input;
                type Output = $output;
                fn glue(input: Self::Input) -> Self::Output {
                    $body(input)
                }
            }                
        };
    }

    pub struct PassThroughGlue<N>(N);
    impl<N: GenericNumber> Glue for PassThroughGlue<N> {
        type Input = N;
        type Output = N;
        fn glue(input: Self::Input) -> Self::Output {
            input
        }
    }

    create_glue!(SingleToDoubleGlue, generic_numbers::Number, generic_numbers::DoubleNumber, |input| generic_numbers::DoubleNumber::from_chunk(input));
    create_glue!(UnsignedSingleToDoubleGlue, generic_numbers::Number, generic_numbers::UnsignedDoubleNumber, |input| generic_numbers::DoubleNumber::from_chunk(input).to_unsigned());
}

/**
 * A modules of some helpful functions used to simplify the writing of arithmetic operations:
 * mono_operation
 * binary_operation
 * tertiary_operation
 */
mod helper_functions {
    use super::*;
    use super::glue::Glue;

    /**
     * Simple macro used to call functions using an array of arguments
     */
    macro_rules! dispatcher {
        (1, $fn:expr, $args:expr) => { $fn($args[0]) };
        (2, $fn:expr, $args:expr) => { $fn($args[0], $args[1]) };
        (3, $fn:expr, $args:expr) => { $fn($args[0], $args[1], $args[2]) };
    }

    /**
     * Retrieves the given number of args from the stack as inputs, puts it through the glue
     * to convert it to the output type, and returns the arguments
     */
    fn operation_args<G: Glue>(
        stack: &mut stack::Stack,
        arg_count: usize
    ) -> Result<Vec<G::Output>, evaluate::Error> {
        let mut args: Vec<G::Output> = Vec::new();
        for _ in 0..arg_count {
            match stack.pop::<<G as Glue>::Input>() {
                Some(number) => args.push(G::glue(number)),
                None => return Result::Err(evaluate::Error::StackUnderflow)
            }
        }
        Result::Ok(args)
    }

    fn operation_result_handler<N: GenericNumber>(
        stack: &mut stack::Stack,
        result: Result<N, evaluate::Error>
    ) -> evaluate::CodeResult {
        result.map(|x| stack.push(x)).map(|_| evaluate::ControlFlowState::Continue)
    }

    pub fn tertiary_operation<G: Glue>(
        stack: &mut stack::Stack,
        f: fn(G::Output, G::Output, G::Output) -> Result<G::Output, evaluate::Error>
    ) -> evaluate::CodeResult {
        operation_args::<G>(stack, 3).map(|args| dispatcher!(3, f, args)).and_then(|result| operation_result_handler::<G::Output>(stack, result))
    }
        
    pub fn mono_operation<G: Glue>(stack: &mut stack::Stack, f: fn(G::Output) -> G::Output) -> evaluate::CodeResult {
        operation_args::<G>(stack, 1).map(|args| dispatcher!(1, f, args)).and_then(|result| operation_result_handler::<G::Output>(stack, Result::Ok(result)))
    }

    pub fn binary_operation<G: Glue>(
        stack: &mut stack::Stack,
        f: fn(G::Output, G::Output) -> Result<G::Output, evaluate::Error>,
    ) -> evaluate::CodeResult {
        operation_args::<G>(stack, 2).map(|args| dispatcher!(2, f, args)).and_then(|result| operation_result_handler::<G::Output>(stack, result))
    }
}

/**
 * The general pipeline of an operation, where A is the number of arguments, G is the specified Glue, O is the operation function to perform (i.e. addition):
 * 
 * pop A values from the stack of type G::Input -> 
 * call G::glue to convert each argument to G::Output -> 
 * call the operation O on the base numeric arguments ->
 * push the output onto the stack
 */

// binary operations
pub fn add<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(a + b)) }
pub fn sub<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(b - a)) }
pub fn multiply<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(a * b)) }
pub fn divide<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| if a == G::Output::as_zero() { Result::Err(evaluate::Error::DivisionByZero) } else { Result::Ok(b / a) }) }
pub fn modulo<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(b % a)) }
pub fn min<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(cmp::min(a, b))) }
pub fn max<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(cmp::max(a, b))) }
// mono operations
pub fn negate<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { mono_operation::<G>(&mut state.stack, |a| a.neg()) }
pub fn abs<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { mono_operation::<G>(&mut state.stack, |a| a.abs()) }
// // tertiary operators
pub fn multiply_divide<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { tertiary_operation::<G>(&mut state.stack, |a, b, c| if c == G::Output::as_zero() { Result::Err(evaluate::Error::DivisionByZero) } else { Result::Ok((a * b) / c)}) }
// // boolean operators
pub fn equals<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(G::Output::from(a == b))) }
pub fn not_equals<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(G::Output::from(a != b))) }
pub fn less_than<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(G::Output::from(b < a))) }
pub fn greater_than<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(G::Output::from(b > a))) }
pub fn less_than_or_equal<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(G::Output::from(b <= a))) }
pub fn greater_than_or_equals<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(G::Output::from(b >= a))) }
pub fn equals_zero<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { mono_operation::<G>(&mut state.stack, |a| G::Output::from(a == G::Output::as_zero())) }
pub fn less_than_zero<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { mono_operation::<G>(&mut state.stack, |a| G::Output::from(a < G::Output::as_zero())) }
pub fn greater_than_zero<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { mono_operation::<G>(&mut state.stack, |a| G::Output::from(a > G::Output::as_zero())) }
// bitwise operations
pub fn bitwise_and<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(b & a)) }
pub fn bitwise_or<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { binary_operation::<G>(&mut state.stack, |a, b| Result::Ok(b | a)) }
pub fn leftshift<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { mono_operation::<G>(&mut state.stack, |a| a << G::Output::as_one()) }
pub fn rightshift<G: Glue>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { mono_operation::<G>(&mut state.stack, |a| a >> G::Output::as_one()) }

// all
macro_rules! overflowable_operations {
    ($pre:tt, $type:ty) => {
        vec![
            (concat!($pre, "+"), false, add::<$type> as super::Operation),
            (concat!($pre, "-"), false, sub::<$type>),
            (concat!($pre, "*"), false, multiply::<$type>),
            (concat!($pre, "2*"), false, leftshift::<$type>),
            (concat!($pre, "2/"), false, rightshift::<$type>),
        ]
    };
}

// M UM
macro_rules! growing_operations {
    ($pre:tt, $type:ty) => {
        vec![
            (concat!($pre, "*/"), false, multiply_divide::<$type>),
        ]
    };
}

// _ D U UD
macro_rules! both_signs_operations {
    ($pre:tt, $type:ty) => {
        vec![
            (concat!($pre, "<"), false, less_than::<$type> as super::Operation),
            (concat!($pre, ">"), false, greater_than::<$type>),
            (concat!($pre, "<="), false, less_than_or_equal::<$type>),
            (concat!($pre, ">="), false, greater_than_or_equals::<$type>),
            (concat!($pre, "0<"), false, less_than_zero::<$type>),
            (concat!($pre, "0>"), false, greater_than_zero::<$type>),
            (concat!($pre, "/"), false, divide::<$type>),
            (concat!($pre, "MIN"), false, min::<$type>),
            (concat!($pre, "MAX"), false, max::<$type>),
            ]
    };
}

// _ D
macro_rules! number_operations {
    ($pre:tt, $type:ty) => {
        vec![
            (concat!($pre, "ABS"), false, abs::<$type> as super::Operation),
            (concat!($pre, "NEGATE"), false, negate::<$type>),        
            (concat!($pre, "MOD"), false, modulo::<$type>),
            (concat!($pre, "="), false, equals::<$type>),
            (concat!($pre, "<>"), false, not_equals::<$type>),
            (concat!($pre, "AND"), false, bitwise_and::<$type>),
            (concat!($pre, "OR"), false, bitwise_or::<$type>),
            (concat!($pre, "0="), false, equals_zero::<$type>),
        ]
    };
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    let mut operations = Vec::new();

    operations.append(&mut overflowable_operations!("", glue::PassThroughGlue<generic_numbers::Number>));
    operations.append(&mut overflowable_operations!("D", glue::PassThroughGlue<generic_numbers::DoubleNumber>));
    operations.append(&mut overflowable_operations!("U", glue::PassThroughGlue<generic_numbers::UnsignedNumber>));
    operations.append(&mut overflowable_operations!("UD", glue::PassThroughGlue<generic_numbers::UnsignedDoubleNumber>));
    operations.append(&mut overflowable_operations!("M", glue::SingleToDoubleGlue));
    operations.append(&mut overflowable_operations!("UM", glue::UnsignedSingleToDoubleGlue));

    operations.append(&mut growing_operations!("", glue::SingleToDoubleGlue));
    operations.append(&mut growing_operations!("U", glue::UnsignedSingleToDoubleGlue));

    operations.append(&mut both_signs_operations!("", glue::PassThroughGlue<generic_numbers::Number>));
    operations.append(&mut both_signs_operations!("D", glue::PassThroughGlue<generic_numbers::DoubleNumber>));
    operations.append(&mut both_signs_operations!("U", glue::PassThroughGlue<generic_numbers::UnsignedNumber>));
    operations.append(&mut both_signs_operations!("UD", glue::PassThroughGlue<generic_numbers::UnsignedDoubleNumber>));

    operations.append(&mut number_operations!("", glue::PassThroughGlue<generic_numbers::Number>));
    operations.append(&mut number_operations!("D", glue::PassThroughGlue<generic_numbers::DoubleNumber>));

    operations
}