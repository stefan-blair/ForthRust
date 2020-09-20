use crate::evaluate::{self, ForthResult};
use crate::memory;
use crate::environment::{generic_numbers, value};


pub trait CloneCompiledInstruction<'a> {
    fn clone_boxed(&self) -> Box<dyn CompiledInstruction<'a> + 'a>;
}

pub trait CompiledInstruction<'a>: CloneCompiledInstruction<'a> + ToString {
    fn execute(&self, state: &mut evaluate::ForthState) -> evaluate::ForthResult;
}

impl <'a, T: 'a + Clone + CompiledInstruction<'a>> CloneCompiledInstruction<'a> for T {
    fn clone_boxed(&self) -> Box<dyn CompiledInstruction<'a> + 'a> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct Push<N: value::ValueVariant>(N);
impl<'a, N: value::ValueVariant + 'a> CompiledInstruction<'a> for Push<N> {
    fn execute(&self, state: &mut evaluate::ForthState) -> evaluate::ForthResult {
        Ok(state.stack.push(self.0))        
    }
}
impl<N: value::ValueVariant> ToString for Push<N> {
    fn to_string(&self) -> String {
        format!("push {}", self.0.to_string())
    }
}

#[derive(Clone)]
struct MemPush<N: value::ValueVariant>(N);
impl<'a, N: value::ValueVariant + 'a> CompiledInstruction<'a> for MemPush<N> {
    fn execute(&self, state: &mut evaluate::ForthState) -> evaluate::ForthResult {
        Ok(state.heap.push(self.0))
    }
}
impl<N: value::ValueVariant> ToString for MemPush<N> {
    fn to_string(&self) -> String {
        format!("[alloc + push mem] {}", self.0.to_string())
    }
}

#[derive(Clone)]
struct Branch(memory::Address);
impl<'a> CompiledInstruction<'a> for Branch {
    fn execute(&self, state: &mut evaluate::ForthState) -> evaluate::ForthResult {
        state.jump_to(self.0)
    }
}
impl ToString for Branch {
    fn to_string(&self) -> String {
        format!("jmp {}", self.0.to_string())
    }
}

#[derive(Clone)]
struct BranchFalse(memory::Address);
impl<'a> CompiledInstruction<'a> for BranchFalse {
    fn execute(&self, state: &mut evaluate::ForthState) -> evaluate::ForthResult {
        if state.stack.pop::<generic_numbers::UnsignedNumber>()? > 0 {
            Ok(())
        } else {
            state.jump_to(self.0)
        }
    }
}
impl ToString for BranchFalse {
    fn to_string(&self) -> String {
        format!("jz {}", self.0.to_string())
    }
}


pub struct InstructionCompiler<'a, 'b, 'c, 'd> {
    state: &'a mut evaluate::ForthState<'b, 'c, 'd>,
    // marks where the compiled instruction should be loaded.  if None, defaults to pushing the instruction onto the current definition
    address: Option<memory::Address>
}

impl<'a, 'b, 'c, 'd> InstructionCompiler<'a, 'b, 'c, 'd> {
    pub fn with_state(state: &'a mut evaluate::ForthState<'b, 'c, 'd>) -> Self {
        Self { state, address: None }
    }

    pub fn with_address(mut self, address: memory::Address) -> Self {
        self.address = Some(address);
        self
    }

    pub fn branch_false(&mut self, destination: memory::Address) -> ForthResult {
        self.compile_instruction(BranchFalse(destination))
    }
    
    pub fn branch(&mut self, destination: memory::Address) -> ForthResult {
        self.compile_instruction(Branch(destination))
    }
    
    pub fn push<N: value::ValueVariant + 'b>(&mut self, value: N) -> ForthResult {
        self.compile_instruction(Push(value))
    }

    pub fn mem_push<N: value::ValueVariant + 'b>(&mut self, value: N) -> ForthResult {
        self.compile_instruction(MemPush(value))
    }

    fn compile_instruction<T: CompiledInstruction<'b> + 'b>(&mut self, instruction: T) -> ForthResult {
        let xt = self.state.compiled_instructions.add(Box::new(instruction));
        if let Some(address) = self.address {
            self.state.write(address, xt)
        } else {
            Ok(self.state.heap.push(xt))
        }
    }
}