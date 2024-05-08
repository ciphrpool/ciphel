use std::cell::Cell;

use ulid::Ulid;

use crate::semantic::MutRc;
use crate::{ast::utils::strings::ID, semantic::scope::scope::Scope};

use super::{
    allocator::{
        heap::{Heap, HeapError},
        stack::{Stack, StackError},
        vtable::VTableError,
    },
    casm::CasmProgram,
    stdio::StdIO,
};

#[derive(Debug, Clone)]
pub enum CodeGenerationError {
    CantLocate,
    UnresolvedError,
    Default,
}

pub type Tid = usize;

#[derive(Debug, Clone)]
pub enum Signal {
    SPAWN,
    EXIT,
    CLOSE(Tid),
    WAIT,
    WAKE(Tid),
    SLEEP(usize),
    JOIN(Tid),
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    StackError(StackError),
    HeapError(HeapError),
    VTableError(VTableError),
    Deserialization,
    UnsupportedOperation,
    MathError,
    ReturnFlagError,
    InvalidUTF8Char,
    CodeSegmentation,
    IncorrectVariant,
    InvalidTID(usize),
    InvalidThreadStateTransition(ThreadState, ThreadState),
    TooManyThread,
    Signal(Signal),
    Default,
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
    pub scope: MutRc<Scope>,
    pub stack: Stack,
    pub program: CasmProgram,
    pub tid: Tid,
}

#[derive(Debug, Clone)]
pub struct Runtime {
    pub tid_count: Cell<usize>,
    pub threads: Vec<Thread>,
}

impl Runtime {
    pub fn new() -> (Self, Heap, StdIO) {
        (
            Self {
                tid_count: Cell::new(0),
                threads: Vec::with_capacity(MAX_THREAD_COUNT),
            },
            Heap::new(),
            StdIO::default(),
        )
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
        let tid = self.tid_count.get() + 1;
        self.tid_count.set(tid);
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
        self.tid_count.set(tid + 1);
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

    pub fn spawn_with_scope(&mut self, scope: MutRc<Scope>) -> Result<usize, RuntimeError> {
        let program = CasmProgram::default();
        let stack = Stack::new();
        let tid = self.tid_count.get() + 1;
        self.tid_count.set(tid);
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
            &'runtime mut MutRc<Scope>,
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

pub trait GenerateCode {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError>;
}

pub trait Locatable {
    fn locate(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError>;

    fn is_assignable(&self) -> bool;

    fn most_left_id(&self) -> Option<ID>;
}
pub trait Executable {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
    ) -> Result<(), RuntimeError>;
}

pub trait CasmMetadata {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram);
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
