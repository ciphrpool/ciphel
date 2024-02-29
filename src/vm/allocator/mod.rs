use crate::semantic::{AccessLevel, MutRc};

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
    pub stdout: MutRc<String>,
    pub stdin: MutRc<String>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            heap: Heap::new(),
            stack: Stack::new(),
            stdout: Default::default(),
            stdin: Default::default(),
        }
    }
}
