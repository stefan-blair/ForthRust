use std::mem;

use crate::evaluate;
use super::definition;


pub type CompiledCode<'a> = Box<dyn Fn(&mut evaluate::ForthEvaluator) -> evaluate::ForthResult + 'a>;

pub struct CompiledCodeSegment<'a> {
    compiled_code: Vec<CompiledCode<'a>>,
}

impl<'a> CompiledCodeSegment<'a> {
    pub fn new() -> Self {
        Self { compiled_code: Vec::new() }
    }

    pub fn borrow<'b>(&'b self) -> CompilingCodeSegment<'b, 'a> {
        CompilingCodeSegment::new(self)
    }

    pub fn restore(&mut self, mut compiling_code_segment: Vec<CompiledCode<'a>>) {
        self.compiled_code.append(&mut compiling_code_segment);
    }

    pub fn add(&mut self, compiled_code: CompiledCode<'a>) -> definition::ExecutionToken {
        self.compiled_code.push(compiled_code);
        definition::ExecutionToken::CompiledOperation(self.compiled_code.len() - 1)
    }

    pub fn get(&self, execution_token: definition::ExecutionToken) -> &CompiledCode {
        match execution_token {
            definition::ExecutionToken::CompiledOperation(offset) => &self.compiled_code[offset],
            _ => panic!("attempted to execute invalid execution token")
        }
    }

    pub fn len(&self) -> usize {
        self.compiled_code.len()
    }
}

pub struct CompilingCodeSegment<'a, 'b> {
    pub compiled_code: &'a CompiledCodeSegment<'b>,
    pub buffer: Vec<CompiledCode<'b>>
}

impl<'a, 'b> CompilingCodeSegment<'a, 'b> {
    fn new(compiled_code: &'a CompiledCodeSegment<'b>) -> Self {
        Self {
            compiled_code,
            buffer: Vec::new()
        }
    }

    pub fn add_compiled_code(&mut self, compiled_code: CompiledCode<'b>) -> definition::ExecutionToken {
        self.buffer.push(compiled_code);
        definition::ExecutionToken::CompiledOperation(self.compiled_code.len() + self.buffer.len() - 1)
    }
}
