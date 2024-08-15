use super::vm::GameEngineStaticFn;

#[derive(Debug, Clone)]
pub struct StdIO {
    pub stdout: StdOut,
    // pub casm_out: String,
    pub stdin: StdIn,
}

impl Default for StdIO {
    fn default() -> Self {
        Self {
            stdin: StdIn::default(),
            stdout: StdOut::default(),
            // casm_out: String::new(),
        }
    }
}

impl StdIO {
    pub fn push_casm_info<G: GameEngineStaticFn>(&mut self, engine: &mut G, content: &str) {
        engine.stdcasm_print(format!("INFO :: {content}\n"));
    }
    pub fn push_casm<G: GameEngineStaticFn>(&mut self, engine: &mut G, content: &str) {
        // self.casm_out.push('\t');
        // self.casm_out.push_str(content);
        // self.casm_out.push('\n');
        engine.stdcasm_print(format!("\t{content}"));
    }
    pub fn push_casm_lib<G: GameEngineStaticFn>(&mut self, engine: &mut G, content: &str) {
        // self.casm_out.push_str("\tsyscall ");
        // self.casm_out.push_str(content);
        // self.casm_out.push('\n');
        engine.stdcasm_print(format!("\tsyscall {content}"));
    }
    pub fn push_casm_label<G: GameEngineStaticFn>(&mut self, engine: &mut G, content: &str) {
        // self.casm_out.push_str(content);
        // self.casm_out.push_str(" :\n");
        engine.stdcasm_print(format!("{content} :"));
    }

    pub fn print_stderr<G: GameEngineStaticFn>(&mut self, engine: &mut G, content: &str) {
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

    pub fn flush<G: GameEngineStaticFn>(&mut self, engine: &mut G) {
        let binding = self.take();
        let content = binding.trim_matches('\"');
        engine.stdout_print(content.into());
    }

    pub fn flushln<G: GameEngineStaticFn>(&mut self, engine: &mut G) {
        let binding = self.take();
        let content = binding.trim_matches('\"');
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
    pub fn request<G: GameEngineStaticFn>(&mut self, engine: &mut G) {
        self.data.clear();
        self.valid = false;
        engine.stdin_request();
    }
    pub fn read<G: GameEngineStaticFn>(&mut self, engine: &mut G) -> Option<String> {
        if !self.valid {
            None
        } else {
            Some(self.data.clone())
        }
    }
}
