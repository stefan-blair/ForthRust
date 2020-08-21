use super::*;


pub fn pop_and_print<N: GenericNumber>(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    state.output_stream.write(&format!("{:?} ", pop_or_underflow!(state.stack, N)));
    Result::Ok(())
}

pub fn print_newline(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    state.output_stream.writeln("");
    Result::Ok(())
}

pub fn print_string(state: &mut evaluate::ForthEvaluator) -> evaluate::ForthResult {
    // state.memory.push(state.compiled_code(Box::new(x: T)))
    Result::Ok(())
}

pub fn get_operations() -> Vec<(&'static str, bool, super::Operation)> {
    vec![
        (".", false, pop_and_print::<generic_numbers::Number>),
        ("D.", false, pop_and_print::<generic_numbers::DoubleNumber>),
        ("C.", false, pop_and_print::<generic_numbers::Byte>),
        ("U.", false, pop_and_print::<generic_numbers::UnsignedNumber>),
        (".\"", true, print_string),
        ("CR", false, print_newline)    
    ]
}