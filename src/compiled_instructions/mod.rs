pub mod instruction_compiler;

use crate::evaluate;


pub type CompiledInstruction<'a> = Box<dyn instruction_compiler::CompiledInstruction<'a> + 'a>;

pub struct CompiledInstructions<'a> {
    compiled_instructions: Vec<CompiledInstruction<'a>>,
}

impl<'a> CompiledInstructions<'a> {
    pub fn new() -> Self {
        Self { compiled_instructions: Vec::new() }
    }

    pub fn add(&mut self, compiled_instruction: CompiledInstruction<'a>) -> evaluate::definition::ExecutionToken {
        self.compiled_instructions.push(compiled_instruction);
        evaluate::definition::ExecutionToken::CompiledInstruction(self.compiled_instructions.len() - 1)
    }

    pub fn get(&self, execution_token: evaluate::definition::ExecutionToken) -> CompiledInstruction<'a> {
        match execution_token {
            evaluate::definition::ExecutionToken::CompiledInstruction(offset) => self.compiled_instructions[offset].clone_boxed(),
            _ => panic!("attempted to execute invalid execution token")
        }
    }

    pub fn len(&self) -> usize {
        self.compiled_instructions.len()
    }
}
