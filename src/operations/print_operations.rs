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