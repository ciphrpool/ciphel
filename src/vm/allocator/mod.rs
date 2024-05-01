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

#[derive(Debug, Clone, Copy)]
pub enum MemoryAddress {
    Heap { offset: usize },
    Stack { offset: Offset, level: AccessLevel },
}

impl MemoryAddress {
    pub fn name(&self) -> String {
        match self {
            MemoryAddress::Heap { offset } => format!("HP[{offset}]"),
            MemoryAddress::Stack { offset, level } => format!("{}", offset.name(level)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Memory<'runtime> {
    pub heap: &'runtime Heap,
    pub stack: &'runtime Stack,
}
