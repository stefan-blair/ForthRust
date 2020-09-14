use super::*;


pub struct InstructionCompiler<'a, 'b, 'c, 'd, 'e, 'f, 'g> {
    state: &'a mut evaluate::ForthEvaluator<'b, 'c, 'd, 'e, 'f, 'g>,
    // marks where the compiled instruction should be loaded.  if None, defaults to pushing the instruction onto the current definition
    address: Option<memory::Address>
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'g> InstructionCompiler<'a, 'b, 'c, 'd, 'e, 'f, 'g> {
    pub fn with_state(state: &'a mut evaluate::ForthEvaluator<'b, 'c, 'd, 'e, 'f, 'g>) -> Self {
        Self { state, address: None }
    }

    pub fn with_address(mut self, address: memory::Address) -> Self {
        self.address = Some(address);
        self
    }

    pub fn branch_false(&mut self, destination: memory::Address) -> ForthResult {
        self.compile_instruction(move |state| {
            if state.stack.pop::<generic_numbers::UnsignedNumber>()? > 0 {
                Ok(())
            } else {
                state.jump_to(destination)
            }
        })
    }
    
    pub fn branch(&mut self, destination: memory::Address) -> ForthResult {
        self.compile_instruction(move |state| state.jump_to(destination))
    }
    
    pub fn push<N: value::ValueVariant + 'g>(&mut self, value: N) -> ForthResult {
        self.compile_instruction(move |state| Ok(state.stack.push(value)))
    }

    pub fn mem_push<N: value::ValueVariant + 'g>(&mut self, value: N) -> ForthResult {
        self.compile_instruction(move |state| Ok(state.memory.push(value)))
    }

    fn compile_instruction<T: Fn(&mut evaluate::ForthEvaluator) -> evaluate::ForthResult + 'g>(&mut self, instruction: T) -> ForthResult {
        let xt = self.state.compiled_code.add_compiled_code(Box::new(instruction));
        if let Some(address) = self.address {
            self.state.memory.write(address, xt)
        } else {
            Ok(self.state.memory.push(xt))
        }
    }
}
