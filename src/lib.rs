mod operations;
mod evaluate;
mod environment;
mod io;

pub use evaluate::Error;
pub use environment::generic_numbers::Number;
pub use io::output_stream;

use io::tokens;

pub struct Forth<OUT> {
    state: evaluate::ForthState,
    pub output_stream: OUT
}

impl<'a, OUT> Forth<OUT> 
where OUT: io::output_stream::OutputStream + 'a {
    pub fn new(output_stream: OUT) -> Self {
        Forth {
            state: evaluate::ForthState::new(),
            output_stream: output_stream
        }
    }

    pub fn stack(&self) -> Vec<Number> {
        self.state.stack.to_vec().iter().map(|x| x.to_number()).collect::<Vec<_>>()
    }

    pub fn return_stack(&self) -> Vec<Number> {
        self.state.return_stack.to_vec().iter().map(|x| x.to_number()).collect::<Vec<_>>()
    }

    pub fn eval(&mut self, input: &str) -> evaluate::ForthResult {
        self.eval_stream(input.chars())
    }

    pub fn eval_stream<I: Iterator<Item = char> + 'a>(&mut self, stream: I) -> evaluate::ForthResult {
        self.state.evaluate(tokens::TokenStream::new(stream), &mut self.output_stream)
    }
}
