mod operations;
mod evaluate;
mod environment;
mod io;

pub use evaluate::Error;
pub use environment::generic_numbers::Number;

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
        self.state.stack.to_vec().iter().map(|x| x.to_number()).collect::<Vec<_>>()
    }

    pub fn return_stack(&self) -> Vec<Number> {
        self.state.return_stack.to_vec().iter().map(|x| x.to_number()).collect::<Vec<_>>()
    }

    pub fn consume_output(&mut self) -> String {
        self.state.output_stream.consume()
    }

    pub fn eval(&mut self, input: &str) -> evaluate::ForthResult {
        let token_stream = tokens::TokenStream::from_string(input);
        self.state.evaluate(token_stream)
    }
}
