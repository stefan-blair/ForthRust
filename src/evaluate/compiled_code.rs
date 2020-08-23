use std::mem::replace;

use crate::evaluate;
use super::definition;

pub type CompiledCode = Box<dyn Fn(&mut evaluate::ForthEvaluator) -> evaluate::ForthResult>;

pub struct CompiledCodeSegment {
    compiled_code: Vec<CompiledCode>,
}

impl CompiledCodeSegment {
    pub fn new() -> Self {
        Self { compiled_code: Vec::new() }
    }

    pub fn borrow(&self) -> CompilingCodeSegment {
        CompilingCodeSegment::new(&self)
    }

    pub fn restore(&mut self, mut compiling_code_segment: Vec<CompiledCode>) {
        self.compiled_code.append(&mut compiling_code_segment);
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

pub struct CompilingCodeSegment<'a> {
    pub compiled_code: &'a CompiledCodeSegment,
    pub buffer: Vec<CompiledCode>
}

impl<'a> CompilingCodeSegment<'a> {
    fn new(compiled_code: &'a CompiledCodeSegment) -> Self {
        Self {
            compiled_code,
            buffer: Vec::new()
        }
    }

    pub fn add_compiled_code(&mut self, compiled_code: CompiledCode) -> definition::ExecutionToken {
        self.buffer.push(compiled_code);
        definition::ExecutionToken::CompiledOperation(self.compiled_code.len() + self.buffer.len() - 1)
    }

    pub fn take_buffer(&mut self) -> Vec<CompiledCode> {
        replace(&mut self.buffer, Vec::new())
    }
}
