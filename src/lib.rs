mod operations;
mod evaluate;
mod tokens;
mod commands;
mod generic_numbers;
mod memory;
mod compiler;

// the tests need these two to be exposed
pub use evaluate::Error;
pub use generic_numbers::Number;

pub struct Forth {
    state: evaluate::ForthState
}

impl Forth {
    pub fn new() -> Self {
        Forth {
            state: evaluate::ForthState::new()
        }
    }

    pub fn stack(&self) -> Vec<Number> {
        self.state.stack.to_vec().iter().map(|x| x.to_raw_number()).collect::<Vec<_>>()
    }

    /**
     * Leaves out all values except numbers, useful for comparing results without having to worry
     * about unpredictable addresses, etc.
     */
    pub fn stack_numbers(&self) -> Vec<Number> {
        self.state.stack.to_vec().into_iter().filter_map(|x| match x {
            memory::Value::Number(x) => Some(x),
            _ => None
        }).collect::<Vec<_>>()
    }

    pub fn consume_output(&mut self) -> String {
        self.state.output_stream.consume()
    }

    pub fn eval(&mut self, input: &str) -> evaluate::ForthResult {
        let token_stream = tokens::TokenStream::from_string(input);
        self.state.evaluate(token_stream)
    }
}
