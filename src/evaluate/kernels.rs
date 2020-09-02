use super::{ForthState, ForthResult, ForthIO, Error};


/**
 * The reason for implementing this feature in terms of type parameters, is so that a chain of additional kernel
 * can be added to the main Forth evaluate loop without having to iterate through an array of function pointers.
 */
pub trait Kernel {
    type NextKernel: Kernel;
    fn new(state: &mut ForthState) -> Self;
    fn get_next_kernel(&mut self) -> &mut Self::NextKernel;

    fn evaluate(&mut self, _state: &mut ForthState, _io: ForthIO) -> ForthResult { Ok(()) }
    fn evaluate_chain(&mut self, state: &mut ForthState, io: ForthIO) -> ForthResult {
        self.evaluate(state, ForthIO { input_stream: io.input_stream, output_stream: io.output_stream })
            .and_then(|_| self.get_next_kernel().evaluate_chain(state, io))
    }

    fn handle_error(&mut self, _state: &mut ForthState, _io: ForthIO, error: Error) -> ForthResult { Err(error) }
    fn handle_error_chain(&mut self, state: &mut ForthState, io: ForthIO, error: Error) -> ForthResult {
        self.handle_error(state, ForthIO { input_stream: io.input_stream, output_stream: io.output_stream}, error)
            .or_else(|error| self.get_next_kernel().handle_error(state, io, error))
    }
}

pub struct DefaultKernel();
impl Kernel for DefaultKernel {
    type NextKernel = Self;
    fn new(_: &mut ForthState) -> Self { 
        Self() 
    }
    fn get_next_kernel(&mut self) -> &mut Self::NextKernel { self }
    fn evaluate(&mut self, _: &mut ForthState, _: ForthIO) -> ForthResult { Ok(()) }
    fn evaluate_chain(&mut self, _: &mut ForthState, _: ForthIO) -> ForthResult { Ok(()) }
    fn handle_error_chain(&mut self, _: &mut ForthState, _: ForthIO, error: Error) -> ForthResult { Err(error) }
}
