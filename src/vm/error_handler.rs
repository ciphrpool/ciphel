use ulid::Ulid;

use super::{program::Program, runtime::RuntimeError};

pub struct ErrorHandler {
    stacktrace: Vec<Ulid>,
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self {
            stacktrace: Vec::default(),
        }
    }
}

impl ErrorHandler {
    pub fn push_catch(&mut self, label: &Ulid) {
        self.stacktrace.push(*label);
    }

    pub fn pop_catch(&mut self) {
        self.stacktrace.pop();
    }

    pub fn catch<E: crate::vm::external::Engine>(
        &self,
        error: RuntimeError,
        program: &Program<E>,
    ) -> Result<usize, RuntimeError> {
        if let Some(label) = self.stacktrace.last() {
            let Some(idx) = program.get_cursor_from_label(&label) else {
                return Err(RuntimeError::CodeSegmentation);
            };
            Ok(idx)
        } else {
            Err(error)
        }
    }
}
