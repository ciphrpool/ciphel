use crate::{ast::expressions::data, semantic::MutRc};

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
pub struct StdOut {
    data: MutRc<String>,
    buffer: MutRc<Option<Vec<String>>>,
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
        let mut borrowed = self.buffer.as_ref().borrow_mut();
        match borrowed.as_mut() {
            Some(buffer) => buffer.push(content.to_string()),
            None => {
                self.data.as_ref().borrow_mut().push_str(&content);
            }
        }
    }
    pub fn open_buffer(&self) {
        let mut borrowed = self.buffer.as_ref().borrow_mut();
        *borrowed = Some(Vec::default());
    }
    pub fn flush_buffer(&self) {
        let borrowed = self.buffer.as_ref().borrow();
        match borrowed.as_ref() {
            Some(buffer) => {
                for words in buffer {
                    self.data.as_ref().borrow_mut().push_str(&words);
                }
            }
            None => {}
        }
        drop(borrowed);
        let mut borrowed = self.buffer.as_ref().borrow_mut();
        *borrowed = None;
    }
    pub fn rev_flush_buffer(&self) {
        let borrowed = self.buffer.as_ref().borrow();
        match borrowed.as_ref() {
            Some(buffer) => {
                for words in buffer.iter().rev() {
                    self.data.as_ref().borrow_mut().push_str(&words);
                }
            }
            None => {}
        }
        drop(borrowed);
        let mut borrowed = self.buffer.as_ref().borrow_mut();
        *borrowed = None;
    }

    pub fn take(&self) -> String {
        self.data.as_ref().take()
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
