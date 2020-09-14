pub mod instruction_compiler;

use crate::evaluate;


pub type CompiledInstruction<'a> = Box<dyn Fn(&mut evaluate::ForthEvaluator) -> evaluate::ForthResult + 'a>;

pub struct CompiledInstructions<'a> {
    compiled_code: Vec<CompiledInstruction<'a>>,
}

impl<'a> CompiledInstructions<'a> {
    pub fn new() -> Self {
        Self { compiled_code: Vec::new() }
    }

    pub fn borrow<'b>(&'b self) -> CompilingInstructions<'b, 'a> {
        CompilingInstructions::new(self)
    }

    pub fn restore(&mut self, mut compiling_code_segment: Vec<CompiledInstruction<'a>>) {
        self.compiled_code.append(&mut compiling_code_segment);
    }

    pub fn get(&self, execution_token: evaluate::definition::ExecutionToken) -> &CompiledInstruction {
        match execution_token {
            evaluate::definition::ExecutionToken::CompiledInstruction(offset) => &self.compiled_code[offset],
            _ => panic!("attempted to execute invalid execution token")
        }
    }

    pub fn len(&self) -> usize {
        self.compiled_code.len()
    }
}

/**
 * Used as a buffer to store instructions compiled during evaluate, before they are join to the main set of CompiledInstructions
 */
pub struct CompilingInstructions<'a, 'b> {
    pub compiled_code: &'a CompiledInstructions<'b>,
    pub buffer: Vec<CompiledInstruction<'b>>
}

impl<'a, 'b> CompilingInstructions<'a, 'b> {
    fn new(compiled_code: &'a CompiledInstructions<'b>) -> Self {
        Self {
            compiled_code,
            buffer: Vec::new()
        }
    }

    fn add_compiled_code(&mut self, compiled_code: CompiledInstruction<'b>) -> evaluate::definition::ExecutionToken {
        self.buffer.push(compiled_code);
        evaluate::definition::ExecutionToken::CompiledInstruction(self.compiled_code.len() + self.buffer.len() - 1)
    }
}
