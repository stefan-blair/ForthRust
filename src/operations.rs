use super::memory;
use super::generic_numbers;
use super::generic_numbers::{GenericNumber, SignedGenericNumber, AsValue};
use super::evaluate;
use super::tokens;


/**
 * These macros provide
 */
macro_rules! get_token {
    ($state:ident) => {
        match $state.input_stream.next() {
            Some(token) => token,
            None => return Result::Err(evaluate::Error::InvalidWord)
        }
    };
}

macro_rules! peek_or_underflow {
    ($stack:expr) => {
        match $stack.peek() {
            Some(v) => v,
            None => return Result::Err(evaluate::Error::StackUnderflow)
        }
    };
}

macro_rules! pop_or_underflow {
    ($stack:expr) => {
        match $stack.pop() {
            Some(v) => v,
            None => return Result::Err(evaluate::Error::StackUnderflow)
        }
    };
    ($stack:expr, $type:ty) => {
        match $stack.pop_number::<$type>() {
            Some(v) => v,
            None => return Result::Err(evaluate::Error::StackUnderflow)
        }
    }
}

macro_rules! get_two_from_stack {
    ($stack:expr) => {
        (pop_or_underflow!($stack), pop_or_underflow!($stack))
    };
}

macro_rules! match_or_error {
    ($obj:expr, $pat:pat, $suc:expr, $err:expr) => {
        match $obj {
            $pat => $suc,
            _ => return Result::Err($err)
        }
    };
}

macro_rules! hard_match_number {
    ($obj:expr) => {
        match_or_error!($obj, memory::Value::Number(number), number, evaluate::Error::InvalidNumber)
    }
}

macro_rules! hard_match_address {
    ($memory:expr, $obj:expr) => {
        match_or_error!($memory.address_from(hard_match_number!($obj)), Some(address), address, evaluate::Error::InvalidAddress)
    }
}

/**
 * Macro that wraps operations to make them into maybe versions (?), which only operate if the top of the stack is nonzero.
 */
macro_rules! maybe {
    ($v:expr) => {
        |state: &mut evaluate::ForthEvaluator| match state.stack.peek().map(|value| value.to_number()) {
            Some(x) if x > 0 => $v(state),
            Some(_) => CONTINUE_RESULT,
            None => Result::Err(evaluate::Error::StackUnderflow),
        }
    };
}

/**
 * Macro used to generically absorb any type of comment.
 */
macro_rules! absorb_comment {
    ($closing_brace:expr) => {
        |state| {
            while let Some(tokens::Token::Name(name)) = state.input_stream.next() {
                if name == $closing_brace {
                    return CONTINUE_RESULT;
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
macro_rules! postpone {
    ($state:expr, $execution_token:expr) => {
        $state.memory.push(memory::ExecutionToken::Operation($execution_token).value());
    };
}

const CONTINUE_RESULT: evaluate::CodeResult = Result::Ok(evaluate::ControlFlowState::Continue);

mod arithmetic_operations {
    use std::cmp;
    use super::*;
    use generic_numbers::{ConvertOperations};

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
            stack: &mut memory::Stack,
            arg_count: usize
        ) -> Result<Vec<G::Output>, evaluate::Error> {
            let mut args: Vec<G::Output> = Vec::new();
            for _ in 0..arg_count {
                match stack.pop_number::<<G as Glue>::Input>() {
                    Some(number) => args.push(G::glue(number)),
                    None => return Result::Err(evaluate::Error::StackUnderflow)
                }
            }
            Result::Ok(args)
        }

        fn operation_result_handler<N: GenericNumber>(
            stack: &mut memory::Stack,
            result: Result<N, evaluate::Error>
        ) -> evaluate::CodeResult {
            result.map(|x| stack.push_number(x)).map(|_| evaluate::ControlFlowState::Continue)
        }

        pub fn tertiary_operation<G: Glue>(
            stack: &mut memory::Stack,
            f: fn(G::Output, G::Output, G::Output) -> Result<G::Output, evaluate::Error>
        ) -> evaluate::CodeResult {
            operation_args::<G>(stack, 3).map(|args| dispatcher!(3, f, args)).and_then(|result| operation_result_handler::<G::Output>(stack, result))
        }
            
        pub fn mono_operation<G: Glue>(stack: &mut memory::Stack, f: fn(G::Output) -> G::Output) -> evaluate::CodeResult {
            operation_args::<G>(stack, 1).map(|args| dispatcher!(1, f, args)).and_then(|result| operation_result_handler::<G::Output>(stack, Result::Ok(result)))
        }

        pub fn binary_operation<G: Glue>(
            stack: &mut memory::Stack,
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
                (concat!($pre, "+"), false, arithmetic_operations::add::<$type> as super::Operation),
                (concat!($pre, "-"), false, arithmetic_operations::sub::<$type>),
                (concat!($pre, "*"), false, arithmetic_operations::multiply::<$type>),
                (concat!($pre, "2*"), false, arithmetic_operations::leftshift::<$type>),
                (concat!($pre, "2/"), false, arithmetic_operations::rightshift::<$type>),
            ]
        };
    }
    
    // M UM
    macro_rules! growing_operations {
        ($pre:tt, $type:ty) => {
            vec![
                (concat!($pre, "*/"), false, arithmetic_operations::multiply_divide::<$type>),
            ]
        };
    }

    // _ D U UD
    macro_rules! both_signs_operations {
        ($pre:tt, $type:ty) => {
            vec![
                (concat!($pre, "<"), false, arithmetic_operations::less_than::<$type> as super::Operation),
                (concat!($pre, ">"), false, arithmetic_operations::greater_than::<$type>),
                (concat!($pre, "<="), false, arithmetic_operations::less_than_or_equal::<$type>),
                (concat!($pre, ">="), false, arithmetic_operations::greater_than_or_equals::<$type>),
                (concat!($pre, "0<"), false, arithmetic_operations::less_than_zero::<$type>),
                (concat!($pre, "0>"), false, arithmetic_operations::greater_than_zero::<$type>),
                (concat!($pre, "/"), false, arithmetic_operations::divide::<$type>),
                (concat!($pre, "MIN"), false, arithmetic_operations::min::<$type>),
                (concat!($pre, "MAX"), false, arithmetic_operations::max::<$type>),
                ]
        };
    }

    // _ D
    macro_rules! number_operations {
        ($pre:tt, $type:ty) => {
            vec![
                (concat!($pre, "ABS"), false, arithmetic_operations::abs::<$type> as super::Operation),
                (concat!($pre, "NEGATE"), false, arithmetic_operations::negate::<$type>),        
                (concat!($pre, "MOD"), false, arithmetic_operations::modulo::<$type>),
                (concat!($pre, "="), false, arithmetic_operations::equals::<$type>),
                (concat!($pre, "<>"), false, arithmetic_operations::not_equals::<$type>),
                (concat!($pre, "AND"), false, arithmetic_operations::bitwise_and::<$type>),
                (concat!($pre, "OR"), false, arithmetic_operations::bitwise_or::<$type>),
                (concat!($pre, "0="), false, arithmetic_operations::equals_zero::<$type>),
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
}

mod memory_operations {
    use super::*;

    pub fn dereference(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
        state.stack.push(state.memory.read(address));
        CONTINUE_RESULT
    }
    
    pub fn memory_write(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let (address, value) = (hard_match_address!(state.memory, pop_or_underflow!(state.stack)), pop_or_underflow!(state.stack));
        state.memory.write(address, value);
        CONTINUE_RESULT
    }

    pub fn pop_write(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        state.memory.push(pop_or_underflow!(state.stack));
        CONTINUE_RESULT
    }

    pub fn number_dereference<N: GenericNumber>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
        state.stack.push_number::<N>(state.memory.read_number::<N>(address));    
        CONTINUE_RESULT
    }

    pub fn number_write<N: GenericNumber>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let number = match state.stack.pop_number::<N>() {
            Some(x) => x,
            None => return Result::Err(evaluate::Error::StackUnderflow)
        };
        let address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
        state.memory.write_number::<N>(address, number);
        CONTINUE_RESULT
    }
        
    pub fn to(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let name = match get_token!(state) {
            tokens::Token::Name(name) => name,
            _ => return Result::Err(evaluate::Error::InvalidWord)
        };
        let nametag = match state.definitions.get_nametag(&name) {
            Some(nametag) => nametag,
            None => return Result::Err(evaluate::Error::UnknownWord)
        };
    
        state.memory.push(state.compiled_code.add_compiled_code(Box::new(move |state| {
            let number = hard_match_number!(pop_or_underflow!(state.stack));
            state.definitions.set(nametag, evaluate::Definition::new(memory::ExecutionToken::Number(number), false));
            CONTINUE_RESULT
        })).value());
    
        CONTINUE_RESULT
    }

    macro_rules! generic_operations {
        ($pre:tt, $type:ty) => {
            vec![
                (concat!($pre, "!") , false, memory_operations::number_write::<$type> as super::Operation),
                (concat!($pre, "@"), false, memory_operations::number_dereference::<$type>)
            ]
        };
    }

    pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
        let mut operations: Vec<(&'static str, bool, super::Operation)> = vec![
            ("!", false, memory_write),
            ("@", false, dereference),
            ("TO", true, to),
            (",", false, pop_write),
        ];

        operations.append(&mut generic_operations!("C", generic_numbers::Byte));
        operations.append(&mut generic_operations!("2", generic_numbers::DoubleNumber));

        operations
    }
}

mod stack_operations {
    use super::*;

    // return stack commands
    pub fn stack_to_return_stack(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.return_stack.push(pop_or_underflow!(state.stack)); CONTINUE_RESULT }
    pub fn twice_stack_to_return_stack(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { stack_to_return_stack(state).and_then(|_| stack_to_return_stack(state)) }
    pub fn return_stack_to_stack(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.stack.push(pop_or_underflow!(state.return_stack)); CONTINUE_RESULT }
    pub fn copy_from_return_stack(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.stack.push(peek_or_underflow!(state.return_stack)); CONTINUE_RESULT }
    
    // argument stack commands
    pub fn dup(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.stack.push(peek_or_underflow!(state.stack)); CONTINUE_RESULT }
    pub fn drop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { pop_or_underflow!(state.stack); CONTINUE_RESULT }
    pub fn rdrop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { pop_or_underflow!(state.return_stack); CONTINUE_RESULT }
    pub fn swap(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { 
        let (a, b) = get_two_from_stack!(&mut state.stack);
        state.stack.push(a);
        state.stack.push(b);
        CONTINUE_RESULT
    }
    pub fn over(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { 
        let (a, b) = get_two_from_stack!(&mut state.stack);
        state.stack.push(b);
        state.stack.push(a);
        state.stack.push(b);
        CONTINUE_RESULT
    }
    pub fn rot(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { 
        let (a, b, c) = (pop_or_underflow!(state.stack), pop_or_underflow!(state.stack), pop_or_underflow!(state.stack));
        state.stack.push(b);
        state.stack.push(a);
        state.stack.push(c);
        CONTINUE_RESULT
    }

    pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
        vec![
            (">R", false, stack_to_return_stack),
            ("2>R", false, twice_stack_to_return_stack),
            ("R>", false, return_stack_to_stack),
            ("R@", false, copy_from_return_stack),
            ("DUP", false, dup),
            ("?DUP", false, maybe!(dup)),
            ("DROP", false, drop),
            ("SWAP", false, swap),
            ("OVER", false, over),
            ("ROT", false, rot),    
        ]
    }
}

mod data_operations {
    use super::*;

    pub fn here(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.stack.push(state.memory.top().to_number().value()); CONTINUE_RESULT }
    pub fn allot(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.memory.expand(hard_match_number!(pop_or_underflow!(state.stack)) as memory::Offset); CONTINUE_RESULT }

    pub fn create(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let name = match get_token!(state) {
            tokens::Token::Name(name) => name,
            _ => return Result::Err(evaluate::Error::InvalidWord)
        };
    
        let address = state.memory.top();
        state.memory.push_none();
        let xt = state.compiled_code.add_compiled_code(Box::new(move |state| { state.stack.push(address.to_number().value()); CONTINUE_RESULT } ));
        state.definitions.add(name, evaluate::Definition::new(xt, false));
    
        CONTINUE_RESULT
    }
    
    pub fn does(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
    
        // execution token that will execute the remainder of the function, add 2 to bypass added ending
        let address = state.memory.top().plus_cell(2);
        let xt = state.compiled_code.add_compiled_code(Box::new(move |state| {
            state.execute_at(address).map(|_| evaluate::ControlFlowState::Continue)
        }));
    
        // this will wrap the most recent definition with the remainder of the code
        let wrapper_xt = state.compiled_code.add_compiled_code(Box::new(move |state| {
            let old_definition = state.definitions.get(state.definitions.get_most_recent());
            let wrapped_xt = state.compiled_code.add_compiled_code(Box::new(move |state| {
                state.execute(old_definition.execution_token).and_then(|_|state.execute(xt))
            }));
            state.definitions.set(state.definitions.get_most_recent(), evaluate::Definition::new(wrapped_xt, false));
            CONTINUE_RESULT
        }));
    
        state.memory.push(wrapper_xt.value());
    
        // add a manual break, so that normal calls to the function wont execute the rest of the code, only created objects
        postpone!(state, super::control_flow_operations::control_flow_break);
    
        CONTINUE_RESULT
    }
    
    pub fn value(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let name = match get_token!(state) {
            tokens::Token::Name(name) => name,
            _ => return Result::Err(evaluate::Error::InvalidWord)
        };

        let number = hard_match_number!(pop_or_underflow!(state.stack));
    
        state.definitions.add(name, evaluate::Definition::new(memory::ExecutionToken::Number(number), false));
        CONTINUE_RESULT
    }

    pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
        vec![
            ("HERE", false, here),
            ("ALLOT", false, allot),
            ("CREATE", false, create),
            ("DOES>", true, does),
            ("VALUE", false, value),
        ]
    }
}

mod control_flow_operations {
    use super::*;

    pub fn control_flow_break(_: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { Result::Ok(evaluate::ControlFlowState::Break) }

    pub fn do_init_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        postpone!(state, super::stack_operations::twice_stack_to_return_stack);
        state.stack.push(state.memory.top().to_number().value());
        state.return_stack.push(memory::Value::Number(0));
        CONTINUE_RESULT
    }
    
    fn loop_runtime(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        // pop off the step from the stack, and the range from the return stack
        let (step, end, start) = match (pop_or_underflow!(state.stack), get_two_from_stack!(state.return_stack)) {
            (memory::Value::Number(step), (memory::Value::Number(end), memory::Value::Number(start))) => (step, end, start),
            _ => return Result::Err(evaluate::Error::InvalidNumber)
        };

        let new_start = start + step;
        // we use a "branch false" instruction, so we want to check for falsehood
        state.stack.push(memory::Value::Number((new_start >= end) as generic_numbers::Number));
        state.return_stack.push(memory::Value::Number(new_start));
        state.return_stack.push(memory::Value::Number(end));
        CONTINUE_RESULT
    }
    
    pub fn loop_plus_compiletime(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        postpone!(state, loop_runtime);

        // get the address of the top of the loop, and patch the conditional branch at the end of the loop
        let loop_address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
        let branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_false_instruction(loop_address)).value();
        state.memory.push(branch_xt);

        patch_leave_instructions(state).map(|c| {
            // pop from the return stack
            postpone!(state, super::stack_operations::rdrop);
            postpone!(state, super::stack_operations::rdrop);
            c
        })
    }

    pub fn loop_compiletime(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        // postpone pushing 1 onto the stack, which is the expected step value on the stack (+LOOP has an explicit step)
        state.memory.push(memory::ExecutionToken::Number(1).value());
        loop_plus_compiletime(state)
    }

    pub fn begin_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        state.stack.push(state.memory.top().to_number().value());
        state.return_stack.push(memory::Value::Number(0));
        CONTINUE_RESULT
    }

    pub fn until_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let loop_address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
        let branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_false_instruction(loop_address)).value();
        state.memory.push(branch_xt);

        patch_leave_instructions(state)
    }

    pub fn again_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let loop_address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
        let branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_instruction(loop_address)).value();
        state.memory.push(branch_xt);

        patch_leave_instructions(state)
    }

    pub fn while_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        state.stack.push(state.memory.top().to_number().value());
        state.memory.push_none();
        CONTINUE_RESULT
    }

    pub fn repeat_loop(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let branch_address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));

        // add a branch instruction to the beginning of the loop unconditionally
        let loop_address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
        let branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_instruction(loop_address)).value();
        state.memory.push(branch_xt);

        // back patch the conditional branch in the middle of the loop
        let conditional_branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_false_instruction(state.memory.top())).value();
        state.memory.write(branch_address, conditional_branch_xt);

        patch_leave_instructions(state)
    }

    pub fn patch_leave_instructions(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let leave_address_count = hard_match_number!(pop_or_underflow!(state.return_stack));
        // if there were any leave instructions, iterate through them patch them to jump to the end of the loop
        if leave_address_count > 0 {
            let leave_branch_xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_instruction(state.memory.top())).value();

            for _ in 0..leave_address_count {
                let leave_address = hard_match_address!(state.memory, pop_or_underflow!(state.return_stack));
                state.memory.write(leave_address, leave_branch_xt);
            }
        }
        CONTINUE_RESULT
    }

    pub fn leave(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let leave_address_count = hard_match_number!(pop_or_underflow!(state.return_stack)) + 1;
        state.return_stack.push(state.memory.top().to_number().value());
        state.return_stack.push(memory::Value::Number(leave_address_count));
        state.memory.push_none();
        CONTINUE_RESULT
    }

    pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
        vec![
            ("BREAK", false, control_flow_break),
            ("DO", true, do_init_loop),
            ("+LOOP", true, loop_plus_compiletime),
            ("LOOP", true, loop_compiletime),
            ("BEGIN", true, begin_loop),
            ("UNTIL", true, until_loop),
            ("AGAIN", true, again_loop),
            ("WHILE", true, while_loop),
            ("REPEAT", true, repeat_loop),
            ("LEAVE", true, leave),        
        ]
    }
}

mod compiler_control_operations {
    use super::*;

    pub fn immedate(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { state.definitions.make_immediate(state.definitions.get_most_recent()); CONTINUE_RESULT }
    pub fn set_interpret(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { *state.execution_mode = evaluate::ExecutionMode::Interpret; CONTINUE_RESULT }
    pub fn set_compile(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult { *state.execution_mode = evaluate::ExecutionMode::Compile; CONTINUE_RESULT }
    
    pub fn start_word_compilation(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let name = match get_token!(state) {
            tokens::Token::Name(name) => name,
            _ => return Result::Err(evaluate::Error::InvalidWord)
        };
    
        let address = state.memory.top();
        let execution_token = state.compiled_code.add_compiled_code(Box::new(move |state| {
            state.execute_at(address).map(|_| evaluate::ControlFlowState::Continue)
        }));
    
        // we will edit the definition to be immediate if the IMMEDIATE keyword is found
        state.definitions.add(name, evaluate::Definition::new(execution_token, false));
    
        set_compile(state)
    }
    
    pub fn end_word_compilation(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        postpone!(state, super::control_flow_operations::control_flow_break);
        set_interpret(state)
    }
    
    pub fn postpone(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let token = get_token!(state);
        let definition = match state.definitions.get_from_token(token) {
            Some(definition) => definition,
            None => return Result::Err(evaluate::Error::UnknownWord)
        };
    
        let xt = if definition.immediate {
            definition.execution_token
        } else {
            state.compiled_code.add_compiled_code(Box::new(move |state| {
                state.memory.push(definition.execution_token.value());
                CONTINUE_RESULT
            }))
        };
    
        state.memory.push(xt.value());
    
        CONTINUE_RESULT
    }

    pub fn literal(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::push_value(pop_or_underflow!(state.stack)));
        state.memory.push(xt.value());
        CONTINUE_RESULT
    }

    pub fn execute(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let execution_token = match pop_or_underflow!(state.stack) {
            memory::Value::ExecutionToken(execution_token) => execution_token,
            _ => return Result::Err(evaluate::Error::InvalidExecutionToken)
        };
        state.execute(execution_token)
    }

    // read the next token from the input stream
    pub fn read_execution_token(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        state.input_stream
            .next().ok_or(evaluate::Error::NoMoreTokens)
            .and_then(|token| state.definitions.get_from_token(token).ok_or(evaluate::Error::UnknownWord))
            .map(|definition| {
                state.stack.push(memory::Value::ExecutionToken(definition.execution_token));
                evaluate::ControlFlowState::Continue
            })       
    }

    pub fn get_execution_token(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        state.input_stream
            .next().ok_or(evaluate::Error::NoMoreTokens)
            .and_then(|token| state.definitions.get_from_token(token).ok_or(evaluate::Error::UnknownWord))
            .map(|definition| {
                let xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::push_value(definition.execution_token.value()));
                state.memory.push(xt.value());
                evaluate::ControlFlowState::Continue
            })        
    }

    /**
     * Generates a bne instruction.  Pops an address off of the stack to be the destination for the branch.
     * Pushes the execution token of this branch instruction onto the stack.
     */
    pub fn push_branch_false_instruction(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
        let xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_false_instruction(address));
        state.stack.push(xt.value());
        CONTINUE_RESULT            
    }

    pub fn push_branch_instruction(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        let address = hard_match_address!(state.memory, pop_or_underflow!(state.stack));
        let xt = state.compiled_code.add_compiled_code(super::code_compiler_helpers::create_branch_instruction(address));
        state.stack.push(xt.value());
        CONTINUE_RESULT            
    }

    pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
        vec![
            ("IMMEDIATE", false, immedate),
            ("[", true, set_interpret),
            ("]", true, set_compile),
            (":", false, start_word_compilation),
            (";", true, end_word_compilation),
            ("POSTPONE", true, postpone),
            ("LITERAL", true, literal),
            ("EXECUTE", false, execute),
            ("`", false, read_execution_token),
            ("[`]", true, get_execution_token),
            ("(", true, absorb_comment!(")")),
            // branch generators
            ("_BNE", false, push_branch_false_instruction),
            ("_B", false, push_branch_instruction),
        ]
    }
}

mod print_operations {
    use super::*;

    pub fn pop_and_print<N: GenericNumber>(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        state.output_stream.write(&format!("{:?} ", pop_or_underflow!(state.stack, N)));
        CONTINUE_RESULT
    }

    pub fn print_newline(state: &mut evaluate::ForthEvaluator) -> evaluate::CodeResult {
        state.output_stream.writeln("");
        CONTINUE_RESULT
    }

    pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
        vec![
            (".", false, pop_and_print::<generic_numbers::Number>),
            ("D.", false, pop_and_print::<generic_numbers::DoubleNumber>),
            ("C.", false, pop_and_print::<generic_numbers::Byte>),
            ("U.", false, pop_and_print::<generic_numbers::UnsignedNumber>),
            ("CR", false, print_newline)    
        ]
    }
}

mod code_compiler_helpers {
    use super::*;

    pub fn create_branch_false_instruction(destination: memory::Address) -> evaluate::CompiledCode {
        Box::new(move |state| {
            match pop_or_underflow!(state.stack) {
                value if value.to_raw_number() > 0 => CONTINUE_RESULT,
                _ => Result::Ok(evaluate::ControlFlowState::Jump(destination)),
            }
        })
    }

    pub fn create_branch_instruction(destination: memory::Address) -> evaluate::CompiledCode {
        Box::new(move |_| Result::Ok(evaluate::ControlFlowState::Jump(destination)))
    }

    pub fn push_value(value: memory::Value) -> evaluate::CompiledCode {
        Box::new(move |state| {
            state.stack.push(value);
            CONTINUE_RESULT
        })
    }
}

// built in operators; name, whether its immediate or not, and the function to execute
pub type Operation = fn(&mut evaluate::ForthEvaluator) -> evaluate::CodeResult;

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