#[derive(Debug, Clone)]
pub struct StdIO {
    pub stdout: StdOut,
    // pub asm_out: String,
    pub stdin: StdIn,
}

impl Default for StdIO {
    fn default() -> Self {
        Self {
            stdin: StdIn::default(),
            stdout: StdOut::default(),
            // asm_out: String::new(),
        }
    }
}

impl StdIO {
    pub fn push_asm_info<E: crate::vm::external::Engine>(&mut self, engine: &mut E, content: &str) {
        engine.stdasm_print(format!("INFO :: {content}\n"));
    }
    pub fn push_asm<E: crate::vm::external::Engine>(&mut self, engine: &mut E, content: &str) {
        // self.asm_out.push('\t');
        // self.asm_out.push_str(content);
        // self.asm_out.push('\n');
        engine.stdasm_print(format!("\t{content}"));
    }
    pub fn push_asm_lib<E: crate::vm::external::Engine>(&mut self, engine: &mut E, content: &str) {
        // self.asm_out.push_str("\tsyscall ");
        // self.asm_out.push_str(content);
        // self.asm_out.push('\n');
        engine.stdasm_print(format!("\tsyscall {content}"));
    }
    pub fn push_asm_label<E: crate::vm::external::Engine>(
        &mut self,
        engine: &mut E,
        content: &str,
    ) {
        // self.asm_out.push_str(content);
        // self.asm_out.push_str(" :\n");
        engine.stdasm_print(format!("{content} :"));
    }

    pub fn print_stderr<E: crate::vm::external::Engine>(&mut self, engine: &mut E, content: &str) {
        engine.stderr_print(format!("Error : {content}"));
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
                None => or.push_str(word),
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
                        None => {
                            *self = OutBuffer::Empty;
                            Some(head.into_iter().rev().collect::<Vec<String>>().join(""))
                        }
                    },
                    None => {
                        *self = OutBuffer::Empty;
                        None
                    }
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
                        None => {
                            *self = OutBuffer::Empty;
                            Some(head.join(""))
                        }
                    },
                    None => {
                        *self = OutBuffer::Empty;
                        None
                    }
                }
            }
            OutBuffer::Empty => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StdOut {
    data: String,
    buffer: OutBuffer,
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

    pub fn flush<E: crate::vm::external::Engine>(&mut self, engine: &mut E) {
        let content = self.take();
        engine.stdout_print(content.into());
    }

    pub fn flushln<E: crate::vm::external::Engine>(&mut self, engine: &mut E) {
        let content = self.take();
        engine.stdout_println(content.into());
    }
}

#[derive(Debug, Clone)]
pub struct StdIn {
    data: String,
    valid: bool,
}
impl Default for StdIn {
    fn default() -> Self {
        Self {
            data: String::new(),
            valid: false,
        }
    }
}

impl StdIn {
    pub fn write(&mut self, content: String) {
        self.data.clear();
        self.valid = true;
        self.data.push_str(&content);
    }
    pub fn request<E: crate::vm::external::Engine>(&mut self, engine: &mut E) {
        self.data.clear();
        self.valid = false;
        engine.stdin_request();
    }
    pub fn read<E: crate::vm::external::Engine>(&mut self, engine: &mut E) -> Option<String> {
        if !self.valid {
            None
        } else {
            Some(self.data.clone())
        }
    }
}
