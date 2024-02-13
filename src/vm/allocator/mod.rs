use crate::semantic::AccessLevel;

use self::{
    heap::{Heap, ALIGNMENT},
    stack::{Offset, Stack},
};

pub mod heap;
pub mod stack;
pub mod vtable;

pub fn align(size: usize) -> usize {
    (size + (ALIGNMENT - 1)) & (!(ALIGNMENT - 1))
}

#[derive(Debug, Clone)]
pub enum MemoryAddress {
    Heap,
    Stack { offset: Offset, level: AccessLevel },
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub heap: Heap,
    pub stack: Stack,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            heap: Heap::new(),
            stack: Stack::new(),
        }
    }
}
