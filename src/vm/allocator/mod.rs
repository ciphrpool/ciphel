use std::ops::Add;

use heap::HEAP_SIZE;
use stack::GLOBAL_SIZE;
use stack::STACK_SIZE;

use crate::semantic::AccessLevel;

use self::{
    heap::{Heap, ALIGNMENT},
    stack::Stack,
};

use super::vm::RuntimeError;

pub mod heap;
pub mod stack;
pub mod vtable;

const RANGE_GLOBAL_SIZE_START: usize = 0;
const RANGE_GLOBAL_SIZE_END: usize = GLOBAL_SIZE - 1;
const RANGE_STACK_SIZE_START: usize = GLOBAL_SIZE;
const RANGE_STACK_SIZE_END: usize = GLOBAL_SIZE + STACK_SIZE - 1;
const RANGE_HEAP_SIZE_START: usize = GLOBAL_SIZE + STACK_SIZE;
const RANGE_HEAP_SIZE_END: usize = GLOBAL_SIZE + STACK_SIZE + HEAP_SIZE - 1;

pub fn align(size: usize) -> usize {
    (size + (ALIGNMENT - 1)) & (!(ALIGNMENT - 1))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryAddress {
    Heap { offset: usize },
    Stack { offset: usize },
    Frame { offset: usize },
    Global { offset: usize },
}

impl MemoryAddress {
    pub fn add(self, n: usize) -> MemoryAddress {
        match self {
            MemoryAddress::Heap { offset } => MemoryAddress::Heap { offset: offset + n },
            MemoryAddress::Stack { offset } => MemoryAddress::Stack { offset: offset + n },
            MemoryAddress::Global { offset } => MemoryAddress::Global { offset: offset + n },
            MemoryAddress::Frame { offset } => MemoryAddress::Global { offset: offset + n },
        }
    }
    pub fn sub(self, n: usize) -> MemoryAddress {
        match self {
            MemoryAddress::Heap { offset } => MemoryAddress::Heap {
                offset: offset.checked_sub(n).unwrap_or(0),
            },
            MemoryAddress::Stack { offset } => MemoryAddress::Stack {
                offset: offset.checked_sub(n).unwrap_or(0),
            },
            MemoryAddress::Global { offset } => MemoryAddress::Global {
                offset: offset.checked_sub(n).unwrap_or(0),
            },
            MemoryAddress::Frame { offset } => MemoryAddress::Global {
                offset: offset.checked_sub(n).unwrap_or(0),
            },
        }
    }

    pub fn into(self, stack: &Stack) -> u64 {
        match self {
            MemoryAddress::Heap { offset } => (offset + GLOBAL_SIZE + STACK_SIZE) as u64,
            MemoryAddress::Stack { offset } => (offset + GLOBAL_SIZE) as u64,
            MemoryAddress::Frame { offset } => (offset + GLOBAL_SIZE + stack.frame_pointer) as u64,
            MemoryAddress::Global { offset } => offset as u64,
        }
    }
}

impl TryInto<MemoryAddress> for u64 {
    type Error = RuntimeError;
    fn try_into(self) -> Result<MemoryAddress, Self::Error> {
        match self as usize {
            RANGE_GLOBAL_SIZE_START..=RANGE_GLOBAL_SIZE_END => Ok(MemoryAddress::Global {
                offset: self as usize,
            }),
            RANGE_STACK_SIZE_START..=RANGE_STACK_SIZE_END => Ok(MemoryAddress::Stack {
                offset: self as usize - GLOBAL_SIZE,
            }),
            RANGE_HEAP_SIZE_START..=RANGE_HEAP_SIZE_END => Ok(MemoryAddress::Heap {
                offset: self as usize - (GLOBAL_SIZE + STACK_SIZE),
            }),
            _ => Err(RuntimeError::MemoryViolation),
        }
    }
}

impl Default for MemoryAddress {
    fn default() -> Self {
        Self::Global { offset: 0 }
    }
}

impl MemoryAddress {
    pub fn name(&self) -> String {
        match self {
            MemoryAddress::Heap { offset } => format!("HP[{offset}]"),
            MemoryAddress::Stack { offset } => format!("SP[{offset}]"),
            MemoryAddress::Frame { offset } => format!("FP[{offset}]"),
            MemoryAddress::Global { offset } => format!("GL[{offset}]"),
        }
    }
}
