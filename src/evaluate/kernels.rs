use super::{ForthState, ForthResult, ForthIO};


/**
 * The reason for implementing this feature in terms of type parameters, is so that a chain of additional kernel
 * can be added to the main Forth evaluate loop without having to iterate through an array of function pointers.
 */
pub trait Kernel {
    type NextKernel: Kernel;
    fn new() -> Self;
    fn get_next_kernel(&mut self) -> &mut Self::NextKernel;

    fn evaluate(&mut self, state: &mut ForthState, io: ForthIO) -> ForthResult;
    fn evaluate_chain(&mut self, state: &mut ForthState, io: ForthIO) -> ForthResult {
        self.evaluate(state, ForthIO { input_stream: io.input_stream, output_stream: io.output_stream })
            .and_then(|_| self.get_next_kernel().evaluate_chain(state, io))
    }
}

pub struct DefaultKernel();
impl Kernel for DefaultKernel {
    type NextKernel = Self;
    fn new() -> Self { 
        Self() 
    }
    fn get_next_kernel(&mut self) -> &mut Self::NextKernel { self }
    fn evaluate(&mut self, _: &mut ForthState, _: ForthIO) -> ForthResult { Ok(()) }
    fn evaluate_chain(&mut self, _: &mut ForthState, _: ForthIO) -> ForthResult { Ok(()) }
}
