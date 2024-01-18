use std::{cell::RefCell, rc::Rc};

use crate::vm::vm::RuntimeError;

use super::heap::ALIGNMENT;

const STACK_SIZE: usize = 512;

#[derive(Debug, Clone)]
pub enum StackError {
    StackOverflow,
    StackUnderflow,
    ReadError,
    WriteError,
    Default,
}

impl Into<RuntimeError> for StackError {
    fn into(self) -> RuntimeError {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Stack {
    stack: Rc<RefCell<Box<[u8; STACK_SIZE]>>>,
    top: Rc<RefCell<usize>>,
}

#[derive(Debug, Clone)]
pub struct Frame {
    zero: usize,
    bottom: usize,
}

#[derive(Debug, Clone)]
pub struct StackSlice {
    pub offset: usize,
    pub size: usize,
}

impl Frame {
    pub fn clean(self, stack: &Stack) -> Result<(), StackError> {
        let top = stack.top.borrow().clone();
        if top < self.zero {
            return Err(StackError::StackUnderflow);
        }
        stack.pop(top - self.zero)
    }

    pub fn saved(&self) -> std::ops::Range<usize> {
        self.bottom..self.zero
    }
}

impl Stack {
    pub fn new() -> Self {
        Self {
            stack: Rc::new(RefCell::new(Box::new([0; STACK_SIZE]))),
            top: Rc::new(RefCell::new(0)),
        }
    }
    pub fn no_return_frame(&self) -> Result<Frame, StackError> {
        let bottom = {
            let top = self.top.borrow();
            top.clone()
        };
        Ok(Frame {
            zero: bottom,
            bottom,
        })
    }
    pub fn frame(&self, offset: usize) -> Result<Frame, StackError> {
        let bottom = {
            let top = self.top.borrow();
            top.clone()
        };
        let _ = self.push(offset)?;
        Ok(Frame {
            zero: bottom + offset,
            bottom,
        })
    }

    pub fn push(&self, size: usize) -> Result<(), StackError> {
        let mut top = self.top.borrow_mut();
        if *top + size >= STACK_SIZE {
            return Err(StackError::StackOverflow);
        }
        *top += size;
        Ok(())
    }

    pub fn pop(&self, size: usize) -> Result<(), StackError> {
        let mut top = self.top.borrow_mut();
        if *top < size {
            return Err(StackError::StackUnderflow);
        }
        *top -= size;
        Ok(())
    }

    pub fn read(&self, offset: usize, size: usize) -> Result<Vec<u8>, StackError> {
        let top = self.top.borrow();
        if offset >= *top || offset + size > *top {
            return Err(StackError::ReadError);
        }
        let borrowed_buffer = self.stack.borrow();
        Ok(borrowed_buffer[offset..offset + size].to_vec())
    }
    pub fn write(&self, offset: usize, data: &Vec<u8>) -> Result<(), StackError> {
        let top = self.top.borrow();
        if offset >= *top || offset + data.len() > *top {
            return Err(StackError::ReadError);
        }
        let mut borrowed_buffer = self.stack.borrow_mut();
        borrowed_buffer[offset..offset + data.len()].copy_from_slice(&data);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_push() {
        let stack = Stack::new();
        let _ = stack.push(8).expect("Push should have succeeded");
        assert_eq!(stack.top.borrow().clone(), 8);
    }

    #[test]
    fn robustness_push() {
        let stack = Stack::new();
        let _ = stack
            .push(STACK_SIZE + 1)
            .expect_err("Push should have failed");
    }

    #[test]
    fn valid_pop() {
        let stack = Stack::new();
        let _ = stack.push(64).expect("Push should have succeeded");
        let _ = stack.pop(32).expect("Pop should have succeeded");

        assert_eq!(stack.top.borrow().clone(), 32);
    }

    #[test]
    fn robustness_pop() {
        let stack = Stack::new();
        let _ = stack.pop(32).expect_err("Pop should have failed");
    }

    #[test]
    fn valid_read() {
        let stack = Stack::new();
        let _ = stack.push(8).expect("Push should have succeeded");

        let mut borrowed_stack = stack.stack.borrow_mut();
        borrowed_stack[0..8].copy_from_slice(&[1u8; 8]);
        drop(borrowed_stack);

        let data = stack.read(0, 8).expect("Read should have succeeded");
        assert_eq!(data, vec![1; 8]);
    }

    #[test]
    fn robustness_read() {
        let stack = Stack::new();
        let _ = stack.read(0, 8).expect_err("Read should have failed");
    }

    #[test]
    fn valid_write() {
        let stack = Stack::new();
        let _ = stack.push(8).expect("Push should have succeeded");

        let _ = stack
            .write(0, &vec![1; 8])
            .expect("Write should have succeeded");

        let borrowed_stack = stack.stack.borrow();
        assert_eq!(borrowed_stack[0..8], vec![1; 8]);
    }

    #[test]
    fn robustness_write() {
        let stack = Stack::new();
        let _ = stack
            .write(0, &vec![1; 8])
            .expect_err("Read should have failed");
    }

    #[test]
    fn valid_frame() {
        let stack = Stack::new();
        let _ = stack.push(8).expect("Push should have succeeded");

        let frame = stack
            .frame(8)
            .expect("Frame creation should have succeeded");
        let _ = stack.push(8).expect("Push should have succeeded");

        assert_eq!(frame.bottom, 8);
        assert_eq!(frame.zero, 16);

        assert_eq!(stack.top.borrow().clone(), 24);
    }

    #[test]
    fn valid_frame_clean() {
        let stack = Stack::new();
        let frame = stack
            .frame(8)
            .expect("Frame creation should have succeeded");
        let _ = stack.push(8).expect("Push should have succeeded");
        let _ = frame.clean(&stack).expect("Clean should have succeeded");

        assert_eq!(stack.top.borrow().clone(), 8);
    }
}
