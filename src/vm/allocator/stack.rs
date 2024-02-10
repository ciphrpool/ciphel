use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use num_traits::ToBytes;

use crate::{semantic::MutRc, vm::vm::RuntimeError};

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
        RuntimeError::StackError(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Offset {
    SB(usize),
    ST(isize),
    FB(usize),
    FZ(isize),
}

#[derive(Debug, Clone)]
pub struct Stack {
    stack: MutRc<Box<[u8; STACK_SIZE]>>,
    top: Cell<usize>,
    frame_bottom: Cell<usize>,
    frame_zero: Cell<usize>,
}

// #[derive(Debug, Clone)]
// pub struct Frame {
//     zero: usize,
//     bottom: usize,
// }

#[derive(Debug, Clone)]
pub struct StackSlice {
    pub offset: Offset,
    pub size: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            stack: Rc::new(RefCell::new(Box::new([0; STACK_SIZE]))),
            top: Cell::new(0),
            frame_bottom: Cell::new(0),
            frame_zero: Cell::new(0),
        }
    }

    pub fn frame(&self, reserved_space: usize) -> Result<(), StackError> {
        let bottom = self.top();

        let _ = self.push(reserved_space + 8 + 8 + 8)?;
        let mut borrowed_buffer = self.stack.borrow_mut();
        // Copy past FB
        borrowed_buffer[bottom + reserved_space + 8..bottom + reserved_space + 8 + 8]
            .copy_from_slice(&(self.frame_bottom.get() as u64).to_le_bytes());
        // Copy past FZ
        borrowed_buffer[bottom + reserved_space + 8 + 8..bottom + reserved_space + 8 + 8 + 8]
            .copy_from_slice(&(self.frame_zero.get() as u64).to_le_bytes());
        // Update FB
        self.frame_bottom.set(bottom);
        // Update FZ
        self.frame_zero.set(bottom + reserved_space + 8 + 8 + 8);
        Ok(())
    }

    pub fn clean(&self) -> Result<(), StackError> {
        let top = self.top();

        let borrowed_buffer = self.stack.borrow();
        let data = TryInto::<[u8; 8]>::try_into(
            &borrowed_buffer[self.frame_zero.get() - 16..self.frame_zero.get() - 8],
        )
        .map_err(|_| StackError::ReadError)?;
        let previous_frame_bottom = u64::from_le_bytes(data);

        let data = TryInto::<[u8; 8]>::try_into(
            &borrowed_buffer[self.frame_zero.get() - 8..self.frame_zero.get()],
        )
        .map_err(|_| StackError::ReadError)?;
        let previous_frame_zero = u64::from_le_bytes(data);

        self.top.set(self.frame_zero.get() - 16);
        // Update FB
        self.frame_bottom.set(previous_frame_bottom as usize);
        // Update FZ
        self.frame_zero.set(previous_frame_zero as usize);
        Ok(())
    }

    pub fn top(&self) -> usize {
        self.top.get()
    }
    pub fn frame_bottom(&self) -> usize {
        self.frame_bottom.get()
    }
    pub fn frame_zero(&self) -> usize {
        self.frame_zero.get()
    }
    pub fn push(&self, size: usize) -> Result<(), StackError> {
        let top = self.top();
        if top + size >= STACK_SIZE {
            return Err(StackError::StackOverflow);
        }
        self.top.set(top + size);
        Ok(())
    }

    pub fn push_with(&self, data: &[u8]) -> Result<(), StackError> {
        let top = self.top();
        if top + data.len() >= STACK_SIZE {
            return Err(StackError::StackOverflow);
        }
        let mut borrowed_buffer = self.stack.borrow_mut();
        borrowed_buffer[top..top + data.len()].copy_from_slice(&data);
        self.top.set(top + data.len());
        Ok(())
    }

    pub fn pop(&self, size: usize) -> Result<Vec<u8>, StackError> {
        let mut top = self.top();
        if top < size {
            return Err(StackError::StackUnderflow);
        }
        let res = {
            let borrowed = self.stack.borrow();
            borrowed[top - size..top].to_vec()
        };
        self.top.set(top - size);
        Ok(res)
    }

    pub fn read(&self, offset: Offset, size: usize) -> Result<Vec<u8>, StackError> {
        let top = self.top();
        match offset {
            Offset::SB(idx) => {
                if idx >= top || idx + size > top {
                    return Err(StackError::ReadError);
                }
                let borrowed_buffer = self.stack.borrow();
                Ok(borrowed_buffer[idx..idx + size].to_vec())
            }
            Offset::ST(idx) => {
                if idx < 0 && ((-idx) as usize > top || top - ((-idx) as usize) + size > top) {
                    return Err(StackError::ReadError);
                } else if idx >= 0 && (idx as usize >= top || top + idx as usize + size > top) {
                    return Err(StackError::ReadError);
                }
                let start = if idx < 0 {
                    top - (-idx as usize)
                } else {
                    top + (idx as usize)
                };
                let borrowed_buffer = self.stack.borrow();
                Ok(borrowed_buffer[start..start + size].to_vec())
            }
            Offset::FB(idx) => {
                let frame_bottom = self.frame_bottom.get();
                if frame_bottom + idx >= top || frame_bottom + idx + size >= top {
                    return Err(StackError::ReadError);
                }
                let borrowed_buffer = self.stack.borrow();
                Ok(borrowed_buffer[frame_bottom + idx..frame_bottom + idx + size].to_vec())
            }
            Offset::FZ(idx) => {
                let frame_zero = self.frame_zero.get();
                let start = if idx <= 0 {
                    frame_zero - (-idx) as usize
                } else {
                    frame_zero + (idx as usize)
                };
                if start >= top || start + size > top {
                    return Err(StackError::ReadError);
                }
                let borrowed_buffer = self.stack.borrow();
                Ok(borrowed_buffer[start..start + size].to_vec())
            }
        }
    }

    // pub fn read_last(&self, size: usize) -> Result<Vec<u8>, StackError> {
    //     let top = self.top();
    //     if top < size {
    //         return Err(StackError::ReadError);
    //     }
    //     let borrowed_buffer = self.stack.borrow();
    //     Ok(borrowed_buffer[top - size..top].to_vec())
    // }

    pub fn write(&self, offset: Offset, data: &[u8]) -> Result<(), StackError> {
        let top = self.top();
        let size = data.len();
        match offset {
            Offset::SB(idx) => {
                if idx >= top || idx + size > top {
                    return Err(StackError::ReadError);
                }
                let mut borrowed_buffer = self.stack.borrow_mut();
                borrowed_buffer[idx..idx + size].copy_from_slice(&data);
                Ok(())
            }
            Offset::ST(idx) => {
                let start = if idx < 0 {
                    top - (idx as usize)
                } else {
                    top + (idx as usize)
                };
                if start >= top || start + size > top {
                    return Err(StackError::ReadError);
                }
                let mut borrowed_buffer = self.stack.borrow_mut();
                borrowed_buffer[start..start + size].copy_from_slice(&data);
                Ok(())
            }
            Offset::FB(idx) => {
                let frame_bottom = self.frame_bottom.get();
                if frame_bottom + idx >= top || frame_bottom + idx + size >= top {
                    return Err(StackError::ReadError);
                }
                let mut borrowed_buffer = self.stack.borrow_mut();
                borrowed_buffer[frame_bottom + idx..frame_bottom + idx + size]
                    .copy_from_slice(&data);
                Ok(())
            }
            Offset::FZ(idx) => {
                let frame_zero = self.frame_zero.get();
                let start = if idx <= 0 {
                    frame_zero - (-idx) as usize
                } else {
                    frame_zero + (idx as usize)
                };
                if start >= top || start + size > top {
                    return Err(StackError::ReadError);
                }

                let mut borrowed_buffer = self.stack.borrow_mut();
                borrowed_buffer[start..start + size].copy_from_slice(&data);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_push() {
        let stack = Stack::new();
        let _ = stack.push(8).expect("Push should have succeeded");
        assert_eq!(stack.top(), 8);
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

        assert_eq!(stack.top(), 32);
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

        let data = stack
            .read(Offset::SB(0), 8)
            .expect("Read should have succeeded");
        assert_eq!(data, vec![1; 8]);
    }

    #[test]
    fn robustness_read() {
        let stack = Stack::new();
        let _ = stack
            .read(Offset::SB(0), 8)
            .expect_err("Read should have failed");
    }

    #[test]
    fn valid_write() {
        let stack = Stack::new();
        let _ = stack.push(8).expect("Push should have succeeded");

        let _ = stack
            .write(Offset::SB(0), &vec![1; 8])
            .expect("Write should have succeeded");

        let borrowed_stack = stack.stack.borrow();
        assert_eq!(borrowed_stack[0..8], vec![1; 8]);
    }

    #[test]
    fn robustness_write() {
        let stack = Stack::new();
        let _ = stack
            .write(Offset::SB(0), &vec![1; 8])
            .expect_err("Read should have failed");
    }

    #[test]
    fn valid_frame() {
        let stack = Stack::new();
        let _ = stack.push(8).expect("Push should have succeeded");

        let _ = stack
            .frame(8)
            .expect("Frame creation should have succeeded");
        let _ = stack.push(8).expect("Push should have succeeded");
        assert_eq!(stack.frame_bottom(), 8);
        assert_eq!(stack.frame_zero(), 40);

        assert_eq!(stack.top(), 48);
    }

    #[test]
    fn valid_frame_clean() {
        let stack = Stack::new();
        let _ = stack
            .frame(8)
            .expect("Frame creation should have succeeded");
        let _ = stack.push(8).expect("Push should have succeeded");
        let _ = stack.clean().expect("Clean should have succeeded");

        assert_eq!(stack.top(), 16);
    }
}
