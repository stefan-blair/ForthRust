use std::collections::HashMap;

use crate::evaluate::{self, definition, kernels};
use crate::io::output_stream::OutputStream;
use super::debug_operations;


pub struct ProfilerInformation {
    total_instruction_count: usize,
    instruction_counts: HashMap<definition::ExecutionToken, usize>,
}

impl ProfilerInformation {
    fn new() -> Self {
        Self { total_instruction_count: 0, instruction_counts: HashMap::new() }
    }

    pub fn record_instruction(&mut self, instruction: definition::ExecutionToken) {
        self.total_instruction_count += 1;
        match self.instruction_counts.get_mut(&instruction) {
            Some(count) => *count += 1,
            None => { self.instruction_counts.insert(instruction, 1); }
        }
    }

    pub fn dump_statistics(&self, state: &mut evaluate::ForthState) {
        state.output_stream.writeln(&format!("total instructions: {}", self.total_instruction_count));
        for (instruction, count) in self.instruction_counts.iter() {
            state.output_stream.writeln(&format!("   {:>30}: {}", debug_operations::stringify_execution_token(state, *instruction), count));
        }
    }
}

#[derive(PartialEq, Eq)]
struct ProfilingWord {
    execution_token: definition::ExecutionToken,
    stack_depth: usize,
    manually_called: bool
}

impl ProfilingWord {
    fn new(execution_token: definition::ExecutionToken) -> Self {
        Self {
            execution_token, stack_depth: 0, manually_called: false
        }
    }
}

pub struct ProfilerKernel<KN: kernels::Kernel> {
    pub global_information: ProfilerInformation,
    
    pub local_information: ProfilerInformation,
    // an option for local information, by manually triggering
    recording: bool,
    // an option for local information, by marking a specific word and recording its stack position when called / returned
    profiling_word: Option<ProfilingWord>,

    next_kernel: KN
}

impl<KN: kernels::Kernel> kernels::Kernel for ProfilerKernel<KN> {
    type NextKernel = KN;
    fn new(state: &mut evaluate::ForthState) -> Self {         
        Self {
            global_information: ProfilerInformation::new(),
            local_information: ProfilerInformation::new(),
            recording: false,
            profiling_word: None,
            next_kernel: KN::new(state)
        } 
    }

    fn get_next_kernel(&mut self) -> &mut Self::NextKernel { &mut self.next_kernel }

    fn evaluate(&mut self, state: &mut evaluate::ForthState) -> evaluate::ForthResult {
        state.current_instruction.map(|current_instruction| {
            self.global_information.record_instruction(current_instruction);

            if self.recording && self.profiling_word != None {
                let profiling_word = self.profiling_word.as_ref().unwrap();
                self.recording = if profiling_word.manually_called {
                    /* 
                    * if the profiling word was called manually, its return is marked by there being no 
                    * instruction pointer (because its awaiting further instructions) 
                    */
                    state.instruction_pointer != None
                } else {
                    /*
                    * otherwise, if the profiling word was jumped to in the middle of execution, then its
                    * return is marked by the return stack reaching its original depth
                    */
                    profiling_word.stack_depth < state.return_stack.len()
                };
            }

            // check if the current insturction matches the profiling word, and that we aren't already recoring
            if let Some(profiling_word) = &mut self.profiling_word {
                if profiling_word.execution_token == current_instruction && !self.recording {
                    profiling_word.stack_depth = state.return_stack.len();
                    profiling_word.manually_called = state.instruction_pointer == None;
                    self.recording = true;
                }
            }

            if self.recording {
                self.local_information.record_instruction(current_instruction);
            }
        });

        Ok(()) 
    }

    fn handle_error(&mut self, state: &mut evaluate::ForthState, error: evaluate::Error) -> evaluate::ForthResult { 
        match error {
            evaluate::Error::UnknownWord(word) if &word == "PROFILE_START" => {
                self.recording = true;
                self.profiling_word = None;
                self.local_information = ProfilerInformation::new();
                Ok(())
            }
            evaluate::Error::UnknownWord(word) if &word == "PROFILE_END" => {
                self.recording = false;
                self.profiling_word = None;
                self.local_information.dump_statistics(state);
                Ok(())
            }
            evaluate::Error::UnknownWord(word) if &word == "PROFILE_STATS" => {
                state.output_stream.writeln("Global Profiling Stats:");
                self.global_information.dump_statistics(state);
                state.output_stream.writeln("Local Profiling Stats:");
                self.local_information.dump_statistics(state);
                Ok(())
            }
            evaluate::Error::UnknownWord(word) if &word == "PROFILE_WORD" => {
                match state.input_stream.next().and_then(|token| state.definitions.get_from_token(token)).map(|definition| definition.execution_token) {
                    Ok(execution_token) => {
                        self.profiling_word = Some(ProfilingWord::new(execution_token));
                        self.local_information = ProfilerInformation::new();
                        state.output_stream.writeln(&format!("Profiling {}", debug_operations::stringify_execution_token(state, execution_token)));
                        Ok(())
                    }
                    Err(error) => Err(error)
                }
            }
            error => Err(error)
        }
    }
}