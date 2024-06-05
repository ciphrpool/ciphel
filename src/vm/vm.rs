use std::cell::Cell;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use ulid::Ulid;

use crate::semantic::{ArcMutex, ArcRwLock};
use crate::{ast::utils::strings::ID, semantic::scope::scope::Scope};

use super::{
    allocator::{
        heap::{Heap, HeapError},
        stack::{Stack, StackError},
    },
    casm::CasmProgram,
    stdio::StdIO,
};

#[derive(Debug, Clone)]
pub enum CodeGenerationError {
    CantLocate,
    UnresolvedError,
    ConcurrencyError,
    Default,
}

pub type Tid = usize;

#[derive(Debug, Clone)]
pub enum Signal {
    SPAWN,
    EXIT,
    CLOSE(Tid),
    WAIT,
    WAIT_STDIN,
    WAKE(Tid),
    SLEEP(usize),
    JOIN(Tid),
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    StackError(StackError),
    HeapError(HeapError),
    // VTableError(VTableError),
    Deserialization,
    UnsupportedOperation,
    MathError,
    ReturnFlagError,
    InvalidUTF8Char,
    CodeSegmentation,
    IncorrectVariant,
    IndexOutOfBound,
    InvalidTID(usize),
    InvalidThreadStateTransition(ThreadState, ThreadState),
    TooManyThread,
    Signal(Signal),
    AssertError,
    ConcurrencyError,
    Default,
}

pub trait GenerateCode {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError>;
}

pub trait Locatable {
    fn locate(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError>;

    fn is_assignable(&self) -> bool;

    fn most_left_id(&self) -> Option<ID>;
}

pub trait GameEngineStaticFn {
    fn stdout_print(&mut self, content: String);
    fn stdout_println(&mut self, content: String);
    fn stderr_print(&mut self, content: String);
    fn stdin_scan(&mut self) -> Option<String>;
    fn stdin_request(&mut self);
    fn stdcasm_print(&mut self, content: String);
}

#[derive(Debug, Clone)]
pub struct NoopGameEngine {}

impl GameEngineStaticFn for NoopGameEngine {
    fn stdout_print(&mut self, content: String) {}
    fn stdout_println(&mut self, content: String) {}

    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan(&mut self) -> Option<String> {
        None
    }
    fn stdin_request(&mut self) {}

    fn stdcasm_print(&mut self, content: String) {}
}

#[derive(Debug, Clone)]
pub struct StdoutTestGameEngine {
    pub out: String,
}

impl GameEngineStaticFn for StdoutTestGameEngine {
    fn stdout_print(&mut self, content: String) {
        self.out = content;
    }
    fn stdout_println(&mut self, content: String) {
        self.out = format!("{}\n", content);
    }
    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan(&mut self) -> Option<String> {
        None
    }
    fn stdin_request(&mut self) {}

    fn stdcasm_print(&mut self, content: String) {}
}

#[derive(Debug, Clone)]
pub struct StdinTestGameEngine {
    pub out: String,
    pub in_buf: String,
}

impl GameEngineStaticFn for StdinTestGameEngine {
    fn stdout_print(&mut self, content: String) {
        self.out = content;
    }
    fn stdout_println(&mut self, content: String) {
        self.out = format!("{}\n", content);
    }
    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan(&mut self) -> Option<String> {
        if self.in_buf.is_empty() {
            None
        } else {
            Some(self.in_buf.clone())
        }
    }
    fn stdin_request(&mut self) {}

    fn stdcasm_print(&mut self, content: String) {
        println!("{}", content);
    }
}

#[derive(Debug, Clone)]
pub struct DbgGameEngine {}

impl GameEngineStaticFn for DbgGameEngine {
    fn stdout_print(&mut self, content: String) {
        print!("{}", content);
    }
    fn stdout_println(&mut self, content: String) {
        println!("{}", content);
    }

    fn stderr_print(&mut self, content: String) {
        eprintln!("{}", content);
    }

    fn stdin_scan(&mut self) -> Option<String> {
        Some("Hello World".to_string())
    }
    fn stdin_request(&mut self) {}

    fn stdcasm_print(&mut self, content: String) {
        println!("{}", content);
    }
}

pub trait Executable<G: GameEngineStaticFn> {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError>;
}

pub trait CasmMetadata<G: GameEngineStaticFn> {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram, engine: &mut G);
    fn weight(&self) -> usize {
        1
    }
}

pub trait DeserializeFrom {
    type Output;
    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError>;
}

pub trait Printer {
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError>;
}

pub trait NextItem {
    fn init_address(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError>;

    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError>;

    fn build_item(
        &self,
        instructions: &CasmProgram,
        end_label: Ulid,
    ) -> Result<(), CodeGenerationError>;

    fn next(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError>;
}

pub const MAX_THREAD_COUNT: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ThreadState {
    IDLE,
    WAITING,
    SLEEPING(usize), // remaining maf until awakening
    ACTIVE,
    COMPLETED,
}

impl ThreadState {
    pub fn is_noop(&self) -> bool {
        match self {
            ThreadState::IDLE => true,
            ThreadState::WAITING => true,
            ThreadState::SLEEPING(_) => true,
            ThreadState::ACTIVE => false,
            ThreadState::COMPLETED => true,
        }
    }

    pub fn init_maf(&mut self, program_at_end: bool) {
        match self {
            ThreadState::IDLE => {
                if !program_at_end {
                    *self = ThreadState::ACTIVE
                }
            }
            ThreadState::WAITING => {}
            ThreadState::SLEEPING(0) => {
                if !program_at_end {
                    *self = ThreadState::ACTIVE
                } else {
                    *self = ThreadState::IDLE
                }
            }
            ThreadState::SLEEPING(n) => *n -= 1,
            ThreadState::ACTIVE => {
                if program_at_end {
                    *self = ThreadState::IDLE
                }
            }
            ThreadState::COMPLETED => {}
        }
    }

    pub fn to(&mut self, dest: Self) -> Result<(), RuntimeError> {
        let dest = match self {
            ThreadState::IDLE => match dest {
                ThreadState::IDLE => dest,
                ThreadState::WAITING => dest,
                ThreadState::SLEEPING(_) => dest,
                ThreadState::ACTIVE => dest,
                ThreadState::COMPLETED => dest,
            },
            ThreadState::WAITING => match dest {
                ThreadState::IDLE => dest,
                ThreadState::WAITING => dest,
                ThreadState::SLEEPING(_) => {
                    return Err(RuntimeError::InvalidThreadStateTransition(*self, dest))
                }
                ThreadState::ACTIVE => dest,
                ThreadState::COMPLETED => dest,
            },
            ThreadState::SLEEPING(_) => match self {
                ThreadState::IDLE => dest,
                ThreadState::WAITING => {
                    return Err(RuntimeError::InvalidThreadStateTransition(*self, dest))
                }
                ThreadState::SLEEPING(_) => dest,
                ThreadState::ACTIVE => dest,
                ThreadState::COMPLETED => dest,
            },
            ThreadState::ACTIVE => match self {
                ThreadState::IDLE => dest,
                ThreadState::WAITING => dest,
                ThreadState::SLEEPING(_) => dest,
                ThreadState::ACTIVE => dest,
                ThreadState::COMPLETED => dest,
            },
            ThreadState::COMPLETED => match self {
                ThreadState::IDLE => {
                    return Err(RuntimeError::InvalidThreadStateTransition(*self, dest))
                }
                ThreadState::WAITING => {
                    return Err(RuntimeError::InvalidThreadStateTransition(*self, dest))
                }
                ThreadState::SLEEPING(_) => {
                    return Err(RuntimeError::InvalidThreadStateTransition(*self, dest))
                }
                ThreadState::ACTIVE => {
                    return Err(RuntimeError::InvalidThreadStateTransition(*self, dest))
                }
                ThreadState::COMPLETED => dest,
            },
        };
        *self = dest;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Thread {
    pub state: ThreadState,
    pub scope: ArcRwLock<Scope>,
    pub stack: Stack,
    pub program: CasmProgram,
    pub tid: Tid,
}

#[derive(Debug, Clone)]
pub struct Runtime {
    pub tid_count: Arc<AtomicUsize>,
    pub threads: Vec<Thread>,
}

impl Runtime {
    pub fn new() -> (Self, Heap, StdIO) {
        (
            Self {
                tid_count: Default::default(),
                threads: Vec::with_capacity(MAX_THREAD_COUNT),
            },
            Heap::new(),
            StdIO::default(),
        )
    }

    pub fn tid_info(&self) -> String {
        let info = {
            let mut buf = String::new();
            for Thread {
                ref tid, ref state, ..
            } in self.threads.iter()
            {
                buf.push_str(&format!(" Tid {tid} = {:?},", state));
            }
            buf
        };
        info
    }

    pub fn all_noop(&self) -> bool {
        let mut noop = true;
        for thread in &self.threads {
            if !thread.state.is_noop() {
                noop = false
            }
        }
        noop
    }

    pub fn spawn(&mut self) -> Result<Tid, RuntimeError> {
        if self.threads.len() >= MAX_THREAD_COUNT {
            return Err(RuntimeError::TooManyThread);
        }
        let scope = Scope::new();
        let program = CasmProgram::default();
        let stack = Stack::new();
        let tid = self.tid_count.as_ref().load(Ordering::Acquire) + 1;
        self.tid_count.as_ref().store(tid, Ordering::Release);
        self.threads.push(Thread {
            scope,
            stack,
            program,
            tid,
            state: ThreadState::IDLE,
        });
        Ok(tid)
    }

    pub fn spawn_with_tid(&mut self, tid: Tid) -> Result<(), RuntimeError> {
        if self.threads.len() >= MAX_THREAD_COUNT {
            return Err(RuntimeError::TooManyThread);
        }
        let scope = Scope::new();
        let program = CasmProgram::default();
        let stack = Stack::new();
        self.tid_count.as_ref().store(tid + 1, Ordering::Release);
        self.threads.push(Thread {
            scope,
            stack,
            program,
            tid,
            state: ThreadState::IDLE,
        });
        Ok(())
    }
    pub fn close(&mut self, tid: Tid) -> Result<(), RuntimeError> {
        let idx = self.threads.iter().position(|t| t.tid == tid);
        if let Some(idx) = idx {
            let _ = self.threads.remove(idx);
            Ok(())
        } else {
            Err(RuntimeError::InvalidTID(tid))
        }
    }

    pub fn spawn_with_scope(&mut self, scope: ArcRwLock<Scope>) -> Result<usize, RuntimeError> {
        let program = CasmProgram::default();
        let stack = Stack::new();
        let tid = self.tid_count.as_ref().load(Ordering::Acquire) + 1;
        self.tid_count.as_ref().store(tid, Ordering::Release);
        self.threads.push(Thread {
            scope,
            stack,
            program,
            tid,
            state: ThreadState::IDLE,
        });
        Ok(tid)
    }

    pub fn get_mut<'runtime>(
        &'runtime mut self,
        tid: Tid,
    ) -> Result<
        (
            &'runtime mut ArcRwLock<Scope>,
            &'runtime mut Stack,
            &mut CasmProgram,
        ),
        RuntimeError,
    > {
        self.threads
            .iter_mut()
            .find(|thread| thread.tid == tid)
            .ok_or(RuntimeError::InvalidTID(tid))
            .map(
                |Thread {
                     scope,
                     stack,
                     program,
                     ..
                 }| (scope, stack, program),
            )
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Thread> {
        self.threads.iter_mut()
    }
}
