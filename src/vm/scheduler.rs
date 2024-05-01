use super::{
    allocator::{heap::Heap, stack::Stack, Memory},
    casm::CasmProgram,
    stdio::StdIO,
    vm::{Runtime, RuntimeError},
};

#[derive(Debug, Clone)]
pub struct Scheduler {}

// #[derive(Debug, Clone)]
// pub struct Env {
//     pub program: CasmProgram,
//     pub stack: Stack,
// }

// impl Default for Env {
//     fn default() -> Self {
//         Self {
//             program: CasmProgram::default(),
//             stack: Stack::new(),
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct Thread {
//     pub env: Env,
//     pub tid: usize,
// }

// impl Thread {
//     pub fn run(&mut self, heap: &mut Heap, stdio: &mut StdIO) -> Result<(), RuntimeError> {
//         self.env.program.execute(self, heap, stdio)
//     }

//     pub fn push_instr(&mut self, instructions: CasmProgram) {
//         self.env.program.merge(instructions)
//     }
// }
