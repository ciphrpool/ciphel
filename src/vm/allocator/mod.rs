use self::{heap::Heap, stack::Stack};

pub mod heap;
pub mod stack;

#[derive(Debug, Clone)]
pub struct Memory {
    pub heap: Heap,
    pub stack: Stack,
}
