use std::{cell::RefCell, rc::Rc};

const STACK_SIZE: usize = 512;

#[derive(Debug, Clone)]
pub enum StackError {
    StackOverflow,
    StackUnderflow,
    Default,
}

#[derive(Debug, Clone)]
pub struct Stack {
    stack: Rc<RefCell<Box<[u8; STACK_SIZE]>>>,
    top: Rc<RefCell<usize>>,
}

#[derive(Debug, Clone)]
pub struct Frame {
    top: usize,
    bottom: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            stack: Rc::new(RefCell::new(Box::new([0; STACK_SIZE]))),
            top: Rc::new(RefCell::new(0)),
        }
    }

    pub fn push(&self) -> Result<(), StackError> {
        unimplemented!("push not yet implemented")
    }

    pub fn pop(&self) -> Result<(), StackError> {
        unimplemented!("pop not yet implemented")
    }

    pub fn read(&self, offset: usize, size: usize) -> Result<Vec<u8>, StackError> {
        unimplemented!("read not yet implemented")
    }
}
