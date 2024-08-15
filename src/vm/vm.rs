use ulid::Ulid;

use crate::e_static;
use crate::semantic::scope::static_types::StaticType;
use crate::semantic::{ArcRwLock, EType, SemanticError};
use crate::{ast::utils::strings::ID, semantic::scope::scope::Scope};

use super::{
    allocator::{
        heap::{Heap, HeapError},
        stack::{Stack, StackError},
    },
    casm::CasmProgram,
    stdio::StdIO,
};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum CodeGenerationError {
    #[error("Unresolved Error")]
    UnresolvedError,
    #[error("internal compilation error")]
    ConcurrencyError,
    #[error("unexpected error")]
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

#[derive(Debug, Clone, Error)]
pub enum RuntimeError {
    #[error("StackError : {0}")]
    StackError(#[from] StackError),
    #[error("HeapError : {0}")]
    HeapError(#[from] HeapError),
    // VTableError(VTableError),
    #[error("Deserialization")]
    Deserialization,
    #[error("UnsupportedOperation")]
    UnsupportedOperation,
    #[error("MathError")]
    MathError,
    #[error("ReturnFlagError")]
    ReturnFlagError,
    #[error("InvalidUTF8Char")]
    InvalidUTF8Char,
    #[error("CodeSegmentation")]
    CodeSegmentation,
    #[error("IncorrectVariant")]
    IncorrectVariant,
    #[error("IndexOutOfBound")]
    IndexOutOfBound,
    #[error("InvalidTID")]
    InvalidTID(usize),
    #[error("InvalidThreadStateTransition")]
    InvalidThreadStateTransition(ThreadState, ThreadState),
    #[error("TooManyThread")]
    TooManyThread,
    #[error("Signal")]
    Signal(Signal),
    #[error("AssertError")]
    AssertError,
    #[error("ConcurrencyError")]
    ConcurrencyError,
    #[error("Default")]
    Default,
}

pub trait GenerateCode {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError>;
}

pub trait Locatable {
    fn locate(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError>;

    fn is_assignable(&self) -> bool;

    fn most_left_id(&self) -> Option<ID>;
}

pub trait DynamicFnResolver {
    fn resolve<G: GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        params: &mut Vec<crate::ast::expressions::Expression>,
    ) -> Result<EType, SemanticError>;
}
pub struct DefaultDynamicFn;
impl DynamicFnResolver for DefaultDynamicFn {
    fn resolve<G: GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        params: &mut Vec<crate::ast::expressions::Expression>,
    ) -> Result<EType, SemanticError> {
        Ok(e_static!(StaticType::Unit))
    }
}

pub trait GameEngineStaticFn {
    fn stdout_print(&mut self, content: String);
    fn stdout_println(&mut self, content: String);
    fn stderr_print(&mut self, content: String);
    fn stdin_scan(&mut self) -> Option<String>;
    fn stdin_request(&mut self);
    fn stdcasm_print(&mut self, content: String);

    fn is_dynamic_fn(suffixe: &Option<ID>, id: &ID) -> Option<impl DynamicFnResolver> {
        None::<DefaultDynamicFn>
    }

    fn execute_dynamic_fn(
        fn_id: String,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut Self,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        unimplemented!("This engine does not have dynamic functions")
    }

    fn name_of_dynamic_fn(
        fn_id: String,
        stdio: &mut StdIO,
        program: &mut CasmProgram,
        engine: &mut Self,
    ) {
        unimplemented!("This engine does not have dynamic functions")
    }

    fn weight_of_dynamic_fn(fn_id: String) -> usize {
        unimplemented!("This engine does not have dynamic functions")
    }

    fn spawn(&mut self, tid: Tid) {}
    fn close(&mut self, tid: Tid) {}
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

#[derive(Debug, Clone)]
pub struct ThreadTestGameEngine {
    pub spawned_thread: usize,
    pub closed_thread: usize,
}

impl GameEngineStaticFn for ThreadTestGameEngine {
    fn stdout_print(&mut self, content: String) {}
    fn stdout_println(&mut self, content: String) {}

    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan(&mut self) -> Option<String> {
        None
    }
    fn stdin_request(&mut self) {}

    fn stdcasm_print(&mut self, content: String) {}
    fn close(&mut self, tid: Tid) {
        self.closed_thread = tid;
    }
    fn spawn(&mut self, tid: Tid) {
        self.spawned_thread = tid;
    }
}

#[derive(Debug, Clone)]
pub struct TestDynamicGameEngine {
    pub dynamic_fn_provider: TestDynamicFnProvider,
    pub out: String,
}

pub trait DynamicFnProvider {
    type DynamicFunctions: DynamicFnResolver;

    fn get_dynamic_fn(prefix: &Option<ID>, id: String) -> Option<Self::DynamicFunctions>;
}

pub trait DynamicFnExecutable {
    type G: GameEngineStaticFn;

    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut Self::G,
        tid: usize,
    ) -> Result<(), RuntimeError>;
}

#[derive(Debug, Clone)]
pub struct TestDynamicFnProvider {}

impl DynamicFnProvider for TestDynamicFnProvider {
    type DynamicFunctions = TestDynamicFn;
    fn get_dynamic_fn(prefix: &Option<ID>, id: String) -> Option<Self::DynamicFunctions> {
        if "dynamic_fn" == id {
            return Some(TestDynamicFn {});
        } else {
            return None;
        }
    }
}
pub struct TestDynamicFn {}

impl DynamicFnResolver for TestDynamicFn {
    fn resolve<G: GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        params: &mut Vec<crate::ast::expressions::Expression>,
    ) -> Result<EType, SemanticError> {
        Ok(e_static!(StaticType::Unit))
    }
}

impl<G: GameEngineStaticFn> Executable<G> for TestDynamicFn {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        stdio.stdout.push("\"Hello World from Dynamic function\"");
        stdio.stdout.flush(engine);
        program.incr();
        Ok(())
    }
}

impl GameEngineStaticFn for TestDynamicGameEngine {
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

    fn execute_dynamic_fn(
        fn_id: String,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut Self,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        if let Some(dynamic_fn) = TestDynamicFnProvider::get_dynamic_fn(&None, fn_id) {
            dynamic_fn.execute(program, stack, heap, stdio, engine, tid)?;
        }
        Ok(())
    }
    fn is_dynamic_fn(preffixe: &Option<ID>, id: &ID) -> Option<impl DynamicFnResolver> {
        TestDynamicFnProvider::get_dynamic_fn(preffixe, id.to_string())
    }
    fn name_of_dynamic_fn(
        fn_id: String,
        stdio: &mut StdIO,
        program: &mut CasmProgram,
        engine: &mut Self,
    ) {
        stdio.push_casm_lib(engine, &fn_id);
    }
    fn weight_of_dynamic_fn(fn_id: String) -> usize {
        1
    }
}

pub trait Executable<G: GameEngineStaticFn> {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError>;
}

pub trait CasmMetadata<G: GameEngineStaticFn> {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G);
    fn weight(&self) -> usize {
        1
    }
}

pub trait DeserializeFrom {
    type Output;
    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError>;
}

pub trait Printer {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError>;
}

pub trait NextItem {
    fn init_address(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError>;

    fn init_index(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError>;

    fn build_item(
        &self,
        instructions: &mut CasmProgram,
        end_label: Ulid,
    ) -> Result<(), CodeGenerationError>;

    fn next(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError>;
}

pub const MAX_THREAD_COUNT: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Player {
    P1,
    P2,
}

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
    pub p1_manager: PlayerThreadsManager,
    pub p2_manager: PlayerThreadsManager,
}

#[derive(Debug, Clone)]
pub struct PlayerThreadsManager {
    pub threads: [Option<Thread>; MAX_THREAD_COUNT],
}

const THREAD_INIT_VALUE_NONE: Option<Thread> = None;
impl PlayerThreadsManager {
    pub fn new() -> Self {
        Self {
            threads: [THREAD_INIT_VALUE_NONE; MAX_THREAD_COUNT],
        }
    }

    pub fn info(&self) -> String {
        let mut buf = String::new();

        for thread in self.threads.iter() {
            if let Some(Thread {
                ref tid, ref state, ..
            }) = thread
            {
                buf.push_str(&format!(" Tid {tid} = {:?},", state));
            }
        }
        buf
    }

    pub fn all_noop(&self) -> bool {
        let mut noop = true;
        for thread in &self.threads {
            if let Some(Thread { state, .. }) = thread {
                if !state.is_noop() {
                    noop = false
                }
            }
        }
        noop
    }

    pub fn thread_count(&self) -> usize {
        self.threads.iter().filter(|t| t.is_some()).count()
    }

    pub fn alive(&self) -> [bool; MAX_THREAD_COUNT] {
        let mut buff = [false; MAX_THREAD_COUNT];
        for i in 0..MAX_THREAD_COUNT {
            buff[i] = self.threads[i].is_some();
        }
        buff
    }

    pub fn spawn<G: GameEngineStaticFn>(&mut self, engine: &mut G) -> Result<usize, RuntimeError> {
        if self.thread_count() >= MAX_THREAD_COUNT {
            return Err(RuntimeError::TooManyThread);
        }

        let scope = Scope::new();
        let program = CasmProgram::default();
        let stack = Stack::new();
        let Some((tid, _)) = self.threads.iter().enumerate().find(|(i, t)| t.is_none()) else {
            return Err(RuntimeError::TooManyThread);
        };

        self.threads[tid].replace(Thread {
            scope,
            stack,
            program,
            tid,
            state: ThreadState::IDLE,
        });
        engine.spawn(tid);
        Ok(tid)
    }

    pub fn spawn_with_tid<G: GameEngineStaticFn>(
        &mut self,
        tid: Tid,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        if self.thread_count() >= MAX_THREAD_COUNT {
            return Err(RuntimeError::TooManyThread);
        }
        if tid >= MAX_THREAD_COUNT {
            return Err(RuntimeError::InvalidTID(tid));
        }
        let scope = Scope::new();
        let program = CasmProgram::default();
        let stack = Stack::new();

        self.threads[tid].replace(Thread {
            scope,
            stack,
            program,
            tid,
            state: ThreadState::IDLE,
        });
        engine.spawn(tid);
        Ok(())
    }

    pub fn close<G: GameEngineStaticFn>(
        &mut self,
        tid: Tid,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        if tid >= MAX_THREAD_COUNT || self.threads[tid].is_none() {
            return Err(RuntimeError::InvalidTID(tid));
        }

        let _ = self.threads[tid].take();
        engine.close(tid);
        Ok(())
    }

    pub fn spawn_with_scope(&mut self, scope: ArcRwLock<Scope>) -> Result<usize, RuntimeError> {
        let program = CasmProgram::default();
        let stack = Stack::new();
        let Some((tid, _)) = self.threads.iter().enumerate().find(|(i, t)| t.is_none()) else {
            return Err(RuntimeError::TooManyThread);
        };
        self.threads[tid].replace(Thread {
            scope,
            stack,
            program,
            tid,
            state: ThreadState::IDLE,
        });
        Ok(tid)
    }
}

impl Runtime {
    pub fn new() -> (Self, Heap, StdIO) {
        (
            Self {
                p1_manager: PlayerThreadsManager::new(),
                p2_manager: PlayerThreadsManager::new(),
            },
            Heap::new(),
            StdIO::default(),
        )
    }

    pub fn tid_info(&self) -> String {
        let mut buf = String::new();
        buf.push_str(&self.p1_manager.info());
        buf.push_str(&self.p2_manager.info());
        buf
    }

    pub fn spawn<G: GameEngineStaticFn>(
        &mut self,
        player: Player,
        engine: &mut G,
    ) -> Result<Tid, RuntimeError> {
        match player {
            Player::P1 => self.p1_manager.spawn(engine),
            Player::P2 => self.p2_manager.spawn(engine),
        }
    }

    pub fn spawn_with_tid<G: GameEngineStaticFn>(
        &mut self,
        player: Player,
        tid: Tid,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match player {
            Player::P1 => self.p1_manager.spawn_with_tid(tid, engine),
            Player::P2 => self.p2_manager.spawn_with_tid(tid, engine),
        }
    }
    pub fn close<G: GameEngineStaticFn>(
        &mut self,
        player: Player,
        tid: Tid,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match player {
            Player::P1 => self.p1_manager.close(tid, engine),
            Player::P2 => self.p2_manager.close(tid, engine),
        }
    }

    pub fn spawn_with_scope(
        &mut self,
        player: Player,
        scope: ArcRwLock<Scope>,
    ) -> Result<usize, RuntimeError> {
        match player {
            Player::P1 => self.p1_manager.spawn_with_scope(scope),
            Player::P2 => self.p2_manager.spawn_with_scope(scope),
        }
    }

    pub fn get_mut<'runtime>(
        &'runtime mut self,
        player: Player,
        tid: Tid,
    ) -> Result<
        (
            &'runtime mut ArcRwLock<Scope>,
            &'runtime mut Stack,
            &mut CasmProgram,
        ),
        RuntimeError,
    > {
        match player {
            Player::P1 => {
                if tid >= MAX_THREAD_COUNT {
                    return Err(RuntimeError::InvalidTID(tid));
                }
                let Some(thread) = &mut self.p1_manager.threads[tid] else {
                    return Err(RuntimeError::InvalidTID(tid));
                };
                Ok((&mut thread.scope, &mut thread.stack, &mut thread.program))
            }
            Player::P2 => {
                if tid >= MAX_THREAD_COUNT {
                    return Err(RuntimeError::InvalidTID(tid));
                }
                let Some(thread) = &mut self.p2_manager.threads[tid] else {
                    return Err(RuntimeError::InvalidTID(tid));
                };
                Ok((&mut thread.scope, &mut thread.stack, &mut thread.program))
            }
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Option<&mut Thread>, Option<&mut Thread>)> {
        let p1_iter = self.p1_manager.threads.iter_mut().map(|t| t.as_mut());

        let p2_iter = self.p2_manager.threads.iter_mut().map(|t| t.as_mut());

        std::iter::zip(p1_iter, p2_iter)
    }
}
