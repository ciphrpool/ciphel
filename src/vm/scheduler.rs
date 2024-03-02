use super::{
    allocator::{stack::Stack, Memory},
    casm::CasmProgram,
    vm::{Runtime, RuntimeError},
};

#[derive(Debug, Clone)]
pub struct Scheduler {}

#[derive(Debug, Clone)]
pub struct Env {
    pub program: CasmProgram,
    pub stack: Stack,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            program: CasmProgram::default(),
            stack: Stack::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Thread<'runtime> {
    pub env: &'runtime Env,
    pub tid: usize,
    pub runtime: &'runtime Runtime,
}

impl<'runtime> Thread<'runtime> {
    pub fn memory(&self) -> Memory {
        Memory {
            stack: &self.env.stack,
            heap: &self.runtime.heap,
        }
    }

    pub fn run(&self) -> Result<(), RuntimeError> {
        self.env.program.execute(self)
    }

    pub fn push_instr(&self, instructions: CasmProgram) {
        self.env.program.merge(instructions)
    }
}
