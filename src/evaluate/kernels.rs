use super::{ForthState, ForthResult, Error};


/**
 * The reason for implementing this feature in terms of type parameters, is so that a chain of additional kernel
 * can be added to the main Forth evaluate loop without having to iterate through an array of function pointers.
 */
pub trait Kernel {
    type NextKernel: Kernel;
    fn new(state: &mut ForthState) -> Self;
    fn get_next_kernel(&mut self) -> &mut Self::NextKernel;

    fn evaluate(&mut self, _state: &mut ForthState) -> ForthResult { Ok(()) }
    fn evaluate_chain(&mut self, state: &mut ForthState) -> ForthResult {
        self.evaluate(state)
            .and_then(|_| self.get_next_kernel().evaluate_chain(state))
    }

    fn handle_error(&mut self, _state: &mut ForthState, error: Error) -> ForthResult { Err(error) }
    fn handle_error_chain(&mut self, state: &mut ForthState, error: Error) -> ForthResult {
        self.handle_error(state, error)
            .or_else(|error| self.get_next_kernel().handle_error(state, error))
    }
}

pub struct DefaultKernel();
impl Kernel for DefaultKernel {
    type NextKernel = Self;
    fn new(_: &mut ForthState) -> Self { 
        Self() 
    }
    fn get_next_kernel(&mut self) -> &mut Self::NextKernel { self }
    fn evaluate(&mut self, _: &mut ForthState) -> ForthResult { Ok(()) }
    fn evaluate_chain(&mut self, _: &mut ForthState) -> ForthResult { Ok(()) }
    fn handle_error_chain(&mut self, _: &mut ForthState, error: Error) -> ForthResult { Err(error) }
}
