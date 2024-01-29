use self::{heap::Heap, stack::Stack};

pub mod heap;
pub mod stack;
pub mod vtable;

#[derive(Debug, Clone)]
pub enum MemoryAddress {
    Heap,
    Stack { offset: usize },
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
