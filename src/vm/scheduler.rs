use super::{
    allocator::{heap::Heap, stack::Stack, Memory},
    casm::CasmProgram,
    stdio::StdIO,
    vm::{Runtime, RuntimeError},
};

#[derive(Debug, Clone)]
pub struct Scheduler {}
