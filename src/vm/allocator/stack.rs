use num_traits::ToBytes;

use thiserror::Error;

use crate::semantic::scope::static_types::POINTER_SIZE;

use super::MemoryAddress;

pub const STACK_SIZE: usize = 2024;
pub const GLOBAL_SIZE: usize = 2024;

#[derive(Debug, Clone, Error)]
pub enum StackError {
    #[error("StackOverflow")]
    StackOverflow,
    #[error("StackUnderflow")]
    StackUnderflow,
    #[error("ReadError")]
    ReadError,
    #[error("WriteError")]
    WriteError,
    #[error("Default")]
    Default,
}

#[derive(Debug, Clone)]
pub struct Stack {
    stack: [u8; STACK_SIZE],
    global: [u8; GLOBAL_SIZE],
    stack_pointer: usize,
    pub frame_pointer: usize,
    pub return_pointer: usize,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Frame {
    frame_pointer: u64,
    return_pointer: u64,
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            stack: [0; STACK_SIZE],
            global: [0; GLOBAL_SIZE],
            stack_pointer: 0,
            frame_pointer: 0,
            return_pointer: 0,
        }
    }
}

impl Stack {
    // pub fn default() -> Self {
    //     Self {
    //         stack: [0; STACK_SIZE],
    //         global: [0; GLOBAL_SIZE],
    //         stack_pointer: 0,
    //         frame_pointer: 0,
    //         return_pointer: 0,
    //     }
    // }

    pub fn top(&self) -> usize {
        self.stack_pointer
    }

    pub fn open_frame(
        &mut self,
        parameters_size: usize,
        return_pointer: usize,
        with_caller_data: Option<u64>,
    ) -> Result<(), StackError> {
        if self.stack_pointer < parameters_size {
            return Err(StackError::StackUnderflow);
        }
        let parameters = if let Some(caller_data) = with_caller_data {
            let mut buffer = Vec::with_capacity(POINTER_SIZE + parameters_size);
            buffer.extend_from_slice(&caller_data.to_le_bytes());
            buffer.extend_from_slice(
                &self.stack[self.stack_pointer - parameters_size..self.stack_pointer],
            );
            buffer
        } else {
            self.stack[self.stack_pointer - parameters_size..self.stack_pointer].to_vec()
        };

        self.stack_pointer -= parameters_size;

        let parameters_size = if with_caller_data.is_some() {
            parameters_size + POINTER_SIZE
        } else {
            parameters_size
        };

        let frame = Frame {
            frame_pointer: self.frame_pointer as u64,
            return_pointer: self.return_pointer as u64,
        };

        self.frame_pointer = self.stack_pointer;
        self.return_pointer = return_pointer;

        // copy the frame onto the stack
        if self.stack_pointer + std::mem::size_of::<Frame>() > STACK_SIZE {
            return Err(StackError::StackOverflow);
        }
        self.stack[self.stack_pointer..self.stack_pointer + std::mem::size_of::<Frame>()]
            .copy_from_slice(unsafe {
                std::slice::from_raw_parts(
                    (&frame as *const Frame) as *const u8,
                    std::mem::size_of::<Frame>(),
                )
            });
        self.stack_pointer += std::mem::size_of::<Frame>();

        // copy the parameter back to the stack
        if self.stack_pointer + parameters_size > STACK_SIZE {
            return Err(StackError::StackOverflow);
        }

        self.stack[self.stack_pointer..self.stack_pointer + parameters_size]
            .copy_from_slice(&parameters);

        self.stack_pointer += parameters_size;

        Ok(())
    }

    pub fn close_frame(&mut self, return_size: usize) -> Result<usize, StackError> {
        if self.stack_pointer <= return_size {
            return Err(StackError::StackUnderflow);
        }
        let return_value =
            self.stack[self.stack_pointer - return_size..self.stack_pointer].to_vec();

        self.stack_pointer = self.frame_pointer;
        if self.frame_pointer + std::mem::size_of::<Frame>() > STACK_SIZE {
            return Err(StackError::StackOverflow);
        }

        let frame = unsafe {
            *(self.stack[self.frame_pointer..self.frame_pointer + std::mem::size_of::<Frame>()]
                .as_ptr() as *const Frame)
        };

        let return_pointer = self.return_pointer;

        self.frame_pointer = frame.frame_pointer as usize;
        self.return_pointer = frame.return_pointer as usize;

        // copy the return value back to the stack
        if self.stack_pointer + return_size > STACK_SIZE {
            return Err(StackError::StackOverflow);
        }

        self.stack[self.stack_pointer..self.stack_pointer + return_size]
            .copy_from_slice(&return_value);

        self.stack_pointer += return_size;

        Ok(return_pointer)
    }

    pub fn push(&mut self, size: usize) -> Result<(), StackError> {
        let top = self.top();
        if top + size >= STACK_SIZE {
            return Err(StackError::StackOverflow);
        }
        self.stack_pointer += size;
        Ok(())
    }

    pub fn push_with(&mut self, data: &[u8]) -> Result<(), StackError> {
        let top = self.top();
        if top + data.len() >= STACK_SIZE {
            return Err(StackError::StackOverflow);
        }
        self.stack[top..top + data.len()].copy_from_slice(&data);
        self.stack_pointer += data.len();
        Ok(())
    }
    pub fn push_with_zero(&mut self, size: usize) -> Result<(), StackError> {
        self.push_with(&vec![0; size])
    }
    pub fn pop<'env>(&'env mut self, size: usize) -> Result<&'env [u8], StackError> {
        let top = self.top();
        if top < size {
            return Err(StackError::StackUnderflow);
        }
        let res = &self.stack[top - size..top];
        self.stack_pointer -= size;
        Ok(res)
    }
    pub fn read<'env>(
        &'env self,
        pointer: MemoryAddress,
        size: usize,
    ) -> Result<&'env [u8], StackError> {
        let MemoryAddress::Stack { offset: pointer } = pointer else {
            return Err(StackError::ReadError);
        };
        let top = self.top();
        if pointer + size > top {
            return Err(StackError::ReadError);
        }
        Ok(&self.stack[pointer..pointer + size])
    }
    pub fn read_in_frame<'env>(
        &'env self,
        pointer: MemoryAddress,
        size: usize,
    ) -> Result<&'env [u8], StackError> {
        let MemoryAddress::Frame { offset } = pointer else {
            return Err(StackError::ReadError);
        };
        let top = self.top();
        let frame_pointer = self.frame_pointer + std::mem::size_of::<Frame>();
        if frame_pointer + offset + size > top {
            return Err(StackError::ReadError);
        }
        let pointer = frame_pointer + offset;
        Ok(&self.stack[pointer..pointer + size])
    }

    pub fn write(&mut self, pointer: MemoryAddress, data: &[u8]) -> Result<(), StackError> {
        let MemoryAddress::Stack { offset: pointer } = pointer else {
            return Err(StackError::ReadError);
        };
        let top = self.top();
        let size = data.len();
        if pointer + size > top {
            return Err(StackError::WriteError);
        }
        self.stack[pointer..pointer + size].copy_from_slice(&data);
        Ok(())
    }

    pub fn write_in_frame(
        &mut self,
        pointer: MemoryAddress,
        data: &[u8],
    ) -> Result<(), StackError> {
        let MemoryAddress::Frame { offset } = pointer else {
            return Err(StackError::ReadError);
        };
        let top = self.top();
        let frame_pointer = self.frame_pointer + std::mem::size_of::<Frame>();
        let size = data.len();
        if frame_pointer + offset + size > top {
            return Err(StackError::WriteError);
        }
        let pointer = frame_pointer + offset;
        self.stack[pointer..pointer + size].copy_from_slice(&data);
        Ok(())
    }

    pub fn write_global(&mut self, pointer: MemoryAddress, data: &[u8]) -> Result<(), StackError> {
        let MemoryAddress::Global { offset: pointer } = pointer else {
            return Err(StackError::ReadError);
        };
        let size = data.len();
        if pointer + size > GLOBAL_SIZE {
            return Err(StackError::WriteError);
        }
        self.global[pointer..pointer + size].copy_from_slice(&data);
        Ok(())
    }

    pub fn read_global<'env>(
        &'env self,
        pointer: MemoryAddress,
        size: usize,
    ) -> Result<&'env [u8], StackError> {
        let MemoryAddress::Global { offset: pointer } = pointer else {
            return Err(StackError::ReadError);
        };
        if pointer + size > GLOBAL_SIZE {
            return Err(StackError::ReadError);
        }
        Ok(&self.global[pointer..pointer + size])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_push() {
        let mut stack = Stack::default();
        let _ = stack.push(8).expect("Push should have succeeded");
        assert_eq!(stack.top(), 8);
    }

    #[test]
    fn robustness_push() {
        let mut stack = Stack::default();
        let _ = stack
            .push(STACK_SIZE + 1)
            .expect_err("Push should have failed");
    }

    #[test]
    fn valid_pop() {
        let mut stack = Stack::default();
        let _ = stack.push(64).expect("Push should have succeeded");
        let _ = stack.pop(32).expect("Pop should have succeeded");

        assert_eq!(stack.top(), 32);
    }

    #[test]
    fn robustness_pop() {
        let mut stack = Stack::default();
        let _ = stack.pop(32).expect_err("Pop should have failed");
    }

    #[test]
    fn valid_read() {
        let mut stack = Stack::default();
        let _ = stack.push(8).expect("Push should have succeeded");

        stack.stack[0..8].copy_from_slice(&[1u8; 8]);

        let data = stack
            .read(MemoryAddress::Stack { offset: 0 }, 8)
            .expect("Read should have succeeded");
        assert_eq!(data, vec![1; 8]);
    }

    #[test]
    fn robustness_read() {
        let stack = Stack::default();
        let _ = stack
            .read(MemoryAddress::Stack { offset: 0 }, 8)
            .expect_err("Read should have failed");
    }

    #[test]
    fn valid_write() {
        let mut stack = Stack::default();
        let _ = stack.push(8).expect("Push should have succeeded");

        let _ = stack
            .write(MemoryAddress::Stack { offset: 0 }, &vec![1; 8])
            .expect("Write should have succeeded");

        assert_eq!(stack.stack[0..8], vec![1; 8]);
    }

    #[test]
    fn robustness_write() {
        let mut stack = Stack::default();
        let _ = stack
            .write(MemoryAddress::Stack { offset: 0 }, &vec![1; 8])
            .expect_err("Read should have failed");
    }

    #[test]
    fn valid_frame() {
        let mut stack = Stack::default();
        let _ = stack.push(8).expect("Push should have succeeded"); /* initial blob */
        let _ = stack.push(8).expect("Push should have succeeded"); /* parameter */

        let _ = stack
            .open_frame(8, 0, None)
            .expect("Frame creation should have succeeded");

        assert_eq!(stack.frame_pointer, 8);
        assert_eq!(stack.return_pointer, 0);

        assert_eq!(
            stack.top(),
            8 /*initial blob */+ std::mem::size_of::<Frame>() + 8 /* parameter */
        );
    }

    #[test]
    fn valid_frame_clean() {
        let mut stack = Stack::default();
        let _ = stack.push(8).expect("Push should have succeeded"); /* initial blob */
        let _ = stack.push(8).expect("Push should have succeeded"); /* parameter */

        let _ = stack
            .open_frame(8, 0, None)
            .expect("Frame creation should have succeeded");
        let _ = stack.push(8).expect("Push should have succeeded"); /* return value */

        let _ = stack
            .close_frame(8)
            .expect("closing the stack frame should have succeeded");

        assert_eq!(stack.top(), 8 + 8);
    }
}
