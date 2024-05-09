use std::marker::PhantomData;

use crate::semantic::MutRc;

use super::vm::GameEngineStaticFn;

#[derive(Debug, Clone)]
pub struct StdIO<G: GameEngineStaticFn + Clone> {
    pub stdout: StdOut<G>,
    // pub casm_out: String,
    pub stdin: StdIn<G>,
}

impl<G: crate::GameEngineStaticFn + Clone> Default for StdIO<G> {
    fn default() -> Self {
        Self {
            stdin: StdIn::default(),
            stdout: StdOut::default(),
            // casm_out: String::new(),
        }
    }
}

impl<G: crate::GameEngineStaticFn + Clone> StdIO<G> {
    pub fn push_casm_info(&mut self, engine: &mut G, content: &str) {
        engine.stdcasm_print(format!("INFO :: {content}\n"));
    }
    pub fn push_casm(&mut self, engine: &mut G, content: &str) {
        // self.casm_out.push('\t');
        // self.casm_out.push_str(content);
        // self.casm_out.push('\n');
        engine.stdcasm_print(format!("\t{content}"));
    }
    pub fn push_casm_lib(&mut self, engine: &mut G, content: &str) {
        // self.casm_out.push_str("\tsyscall ");
        // self.casm_out.push_str(content);
        // self.casm_out.push('\n');
        engine.stdcasm_print(format!("\tsyscall {content}"));
    }
    pub fn push_casm_label(&mut self, engine: &mut G, content: &str) {
        // self.casm_out.push_str(content);
        // self.casm_out.push_str(" :\n");
        engine.stdcasm_print(format!("{content} :"));
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
pub struct StdOut<G: GameEngineStaticFn + Clone> {
    data: String,
    buffer: OutBuffer,
    _phantom: PhantomData<G>,
}
impl<G: GameEngineStaticFn + Clone> Default for StdOut<G> {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            data: Default::default(),
            _phantom: PhantomData::default(),
        }
    }
}

impl<G: GameEngineStaticFn + Clone> StdOut<G> {
    pub fn push(&mut self, content: &str) {
        self.buffer.push(content, &mut self.data);
    }

    pub fn spawn_buffer(&mut self) {
        self.buffer.spawn();
    }
    pub fn flush_buffer(&mut self) {
        match self.buffer.flush() {
            Some(content) => self.data.push_str(&content),
            None => {}
        }
    }
    pub fn rev_flush_buffer(&mut self) {
        match self.buffer.rev_flush() {
            Some(content) => self.data.push_str(&content),
            None => {}
        }
    }

    fn take(&mut self) -> String {
        std::mem::replace(&mut self.data, String::new())
    }

    pub fn flush(&mut self, engine: &mut G) {
        let binding = self.take();
        let content = binding.trim_matches('\"');
        engine.stdout_print(content.into());
    }

    pub fn flushln(&mut self, engine: &mut G) {
        let binding = self.take();
        let content = binding.trim_matches('\"');
        engine.stdout_println(content.into());
    }
}

#[derive(Debug, Clone)]
pub struct StdIn<G: GameEngineStaticFn + Clone> {
    data: String,
    valid: bool,
    _phantom: PhantomData<G>,
}
impl<G: GameEngineStaticFn + Clone> Default for StdIn<G> {
    fn default() -> Self {
        Self {
            data: String::new(),
            valid: false,
            _phantom: PhantomData::default(),
        }
    }
}

impl<G: GameEngineStaticFn + Clone> StdIn<G> {
    pub fn write(&mut self, content: String) {
        self.data.clear();
        self.valid = true;
        self.data.push_str(&content);
    }
    pub fn request(&mut self, engine: &mut G) {
        self.data.clear();
        self.valid = false;
        engine.stdin_request();
    }
    pub fn read(&mut self, engine: &mut G) -> Option<String> {
        if !self.valid {
            None
        } else {
            Some(self.data.clone())
        }
    }
}
