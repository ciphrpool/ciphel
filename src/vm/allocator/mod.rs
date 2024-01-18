use self::{heap::Heap, stack::Stack};

pub mod heap;
pub mod stack;
pub mod vtable;

#[derive(Debug, Clone)]
pub enum MemoryAddress {
    Heap {
        address: usize,
        offset: usize,
        size: usize,
    },
    Stack {
        offset: usize,
        size: usize,
    },
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub heap: Heap,
    pub stack: Stack,
}
