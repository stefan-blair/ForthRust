pub struct ForthConfig {
    pub return_stack_addr: usize,
    pub stack_addr: usize,
    pub data_space_addr: usize,
    pub pad_addr: usize,
    pub heap_addr: usize,
    pub internal_state_memory_addr: usize,
    pub anonymous_mappings_addr: usize,

    // the number of bytes a definition can have and be copied by the compile, word
    pub definition_copy_threshold: usize,
}

impl Default for ForthConfig {
    fn default() -> Self {
        Self {
            return_stack_addr: 0x56cadeace000,
            stack_addr: 0x7aceddead000,
            data_space_addr: 0x7feaddead000,
            pad_addr: 0x76beaded5000,
            heap_addr: 0x44ea5c69c000,
            internal_state_memory_addr: 0x5deadbeef000,
            anonymous_mappings_addr: 0x55bedead1000,
            definition_copy_threshold: 0x20
        }
    }
}