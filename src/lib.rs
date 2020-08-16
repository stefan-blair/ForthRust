mod operations;
mod evaluate;
mod generic_numbers;
mod memory;
mod stack;
mod io;

pub use evaluate::Error;
pub use generic_numbers::Number;

use io::tokens;

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

    pub fn consume_output(&mut self) -> String {
        self.state.output_stream.consume()
    }

    pub fn eval(&mut self, input: &str) -> evaluate::ForthResult {
        let token_stream = tokens::TokenStream::from_string(input);
        self.state.evaluate(token_stream)
    }
}
