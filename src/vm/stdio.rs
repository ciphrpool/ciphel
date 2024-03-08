use crate::{ast::expressions::data, semantic::MutRc};

use super::allocator::heap;

#[derive(Debug, Clone)]
pub struct StdIO {
    pub stdout: StdOut,
    pub stdin: StdIn,
}

impl Default for StdIO {
    fn default() -> Self {
        Self {
            stdin: StdIn::default(),
            stdout: StdOut::default(),
        }
    }
}

#[derive(Debug, Clone)]
enum OutBuffer {
    Stack(Vec<Vec<String>>),
    Empty,
}

impl Default for OutBuffer {
    fn default() -> Self {
        Self::Empty
    }
}

impl OutBuffer {
    fn push(&mut self, word: &str, or: &mut String) {
        match self {
            OutBuffer::Stack(stack) => match stack.last_mut() {
                Some(head) => head.push(word.to_owned()),
                None => {}
            },
            OutBuffer::Empty => or.push_str(word),
        }
    }

    fn spawn(&mut self) {
        match self {
            OutBuffer::Stack(stack) => {
                stack.push(Vec::default());
            }
            OutBuffer::Empty => *self = OutBuffer::Stack(vec![Vec::default()]),
        }
    }
    fn rev_flush(&mut self) -> Option<String> {
        match self {
            OutBuffer::Stack(stack) => {
                let head = stack.pop();
                match head {
                    Some(head) => match stack.last_mut() {
                        Some(stack) => {
                            stack.push(head.into_iter().rev().collect::<Vec<String>>().join(""));
                            None
                        }
                        None => Some(head.into_iter().rev().collect::<Vec<String>>().join("")),
                    },
                    None => None,
                }
            }
            OutBuffer::Empty => None,
        }
    }
    fn flush(&mut self) -> Option<String> {
        match self {
            OutBuffer::Stack(stack) => {
                let head = stack.pop();
                match head {
                    Some(head) => match stack.last_mut() {
                        Some(stack) => {
                            stack.push(head.join(""));
                            None
                        }
                        None => Some(head.join("")),
                    },
                    None => None,
                }
            }
            OutBuffer::Empty => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StdOut {
    data: MutRc<String>,
    buffer: MutRc<OutBuffer>,
}
impl Default for StdOut {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            data: Default::default(),
        }
    }
}

impl StdOut {
    pub fn push(&self, content: &str) {
        let mut buffer = self.buffer.borrow_mut();
        buffer.push(content, &mut self.data.borrow_mut());
    }
    pub fn spawn_buffer(&self) {
        self.buffer.borrow_mut().spawn();
    }
    pub fn flush_buffer(&self) {
        match self.buffer.borrow_mut().flush() {
            Some(content) => self.data.borrow_mut().push_str(&content),
            None => {}
        }
    }
    pub fn rev_flush_buffer(&self) {
        match self.buffer.borrow_mut().rev_flush() {
            Some(content) => self.data.borrow_mut().push_str(&content),
            None => {}
        }
    }

    pub fn take(&self) -> String {
        std::mem::replace(&mut *self.data.borrow_mut(), String::new())
    }
}

#[derive(Debug, Clone)]
pub struct StdIn {
    data: MutRc<String>,
    buffer: MutRc<Option<Vec<String>>>,
}
impl Default for StdIn {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            data: Default::default(),
        }
    }
}
