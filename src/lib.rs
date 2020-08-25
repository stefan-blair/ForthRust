mod operations;
mod evaluate;
mod environment;
mod io;
mod debugging;

pub use evaluate::Error;
pub use environment::generic_numbers::Number;
pub use io::output_stream;
pub use debugging::debugger;

use io::tokens;

pub struct Forth<'a, OUT> {
    state: evaluate::ForthState<'a>,
    pub output_stream: OUT
}

impl<'a, OUT> Forth<'a, OUT> 
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

    pub fn eval<'b>(&mut self, input: &'b str) -> evaluate::ForthResult {
        self.eval_stream(input.chars())
    }

    pub fn eval_stream<'b, I: Iterator<Item = char> + 'b>(&mut self, stream: I) -> evaluate::ForthResult {
        self.state.evaluate(&mut tokens::TokenStream::new(stream), &mut self.output_stream)
    }

    pub fn add_operations(&mut self, operations: operations::OperationTable) {
        self.state.add_operations(operations);
    }
}
