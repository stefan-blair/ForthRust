use crate::evaluate::{self, definition};
use crate::memory;
use crate::environment::{generic_numbers, value, units};
use super::CompiledInstructions;


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
        Ok(state.data_space.push(self.0))
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

#[derive(Clone)]
struct RelativeBranchMeta(bool, units::Bytes);
impl RelativeBranchMeta {
    fn new(instruction_pointer: memory::Address, destination: memory::Address) -> Self {
        let instruction_pointer = instruction_pointer.plus_cell(units::Cells::one());
        if destination.get() < instruction_pointer.get() {
            Self(true, instruction_pointer.get() - destination.get())
        } else {
            Self(false, destination.get() - instruction_pointer.get())
        }
    }
}
impl ToString for RelativeBranchMeta {
    fn to_string(&self) -> String {
        let sign = if self.0 { "-" } else { "+" };
        format!("[ip {} {}]", sign, self.1.to_string())
    }
}

#[derive(Clone)]
struct RelativeBranch(RelativeBranchMeta);
impl<'a> CompiledInstruction<'a> for RelativeBranch {
    fn execute(&self, state: &mut evaluate::ForthState) -> evaluate::ForthResult {
        state.relative_jump_to(self.0.0, self.0.1)
    }
}
impl ToString for RelativeBranch {
    fn to_string(&self) -> String {
        format!("jmp {}", self.0.to_string())
    }
}

#[derive(Clone)]
struct RelativeBranchFalse(RelativeBranchMeta);
impl<'a> CompiledInstruction<'a> for RelativeBranchFalse {
    fn execute(&self, state: &mut evaluate::ForthState) -> evaluate::ForthResult {
        if state.stack.pop::<generic_numbers::UnsignedNumber>()? > 0 {
            Ok(())
        } else {
            state.relative_jump_to(self.0.0, self.0.1)
        }
    }
}
impl ToString for RelativeBranchFalse {
    fn to_string(&self) -> String {
        format!("jz {}", self.0.to_string())
    }
}

pub struct InstructionCompiler<'b, 'a> {
    pub compiled_instructions: &'b mut CompiledInstructions<'a>
}

impl<'b, 'a> InstructionCompiler<'b, 'a> {
    pub fn branch_false(&mut self, destination: memory::Address) -> definition::ExecutionToken {
        self.compile_instruction(BranchFalse(destination))
    }
    
    pub fn branch(&mut self, destination: memory::Address) -> definition::ExecutionToken {
        self.compile_instruction(Branch(destination))
    }

    pub fn relative_branch(&mut self, instruction_pointer: memory::Address, destination: memory::Address) -> definition::ExecutionToken {
        self.compile_instruction(RelativeBranch(RelativeBranchMeta::new(instruction_pointer, destination)))
    }

    pub fn relative_branch_false(&mut self, instruction_pointer: memory::Address, destination: memory::Address) -> definition::ExecutionToken {
        self.compile_instruction(RelativeBranchFalse(RelativeBranchMeta::new(instruction_pointer, destination)))
    }
    
    pub fn push<N: value::ValueVariant + 'a>(&mut self, value: N) -> definition::ExecutionToken {
        self.compile_instruction(Push(value))
    }

    pub fn mem_push<N: value::ValueVariant + 'a>(&mut self, value: N) -> definition::ExecutionToken {
        self.compile_instruction(MemPush(value))
    }

    fn compile_instruction<T: CompiledInstruction<'a> + 'a>(&mut self, instruction: T) -> definition::ExecutionToken {
        self.compiled_instructions.add(Box::new(instruction))
    }
}
