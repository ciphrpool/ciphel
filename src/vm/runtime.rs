use std::collections::HashMap;

use thiserror::Error;

use crate::semantic::scope::scope::ScopeManager;

use super::{
    allocator::{
        heap::HeapError,
        stack::{Stack, StackError},
    },
    external::ExternThreadIdentifier,
    program::Program,
    scheduler_v2::{Scheduler, SchedulingPolicy},
};

#[derive(Debug, Clone, Error)]
pub enum RuntimeError {
    #[error("StackError : {0}")]
    StackError(#[from] StackError),
    #[error("HeapError : {0}")]
    HeapError(#[from] HeapError),

    #[error("MEMORY VIOLATION")]
    MemoryViolation,
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

    // #[error("InvalidThreadStateTransition")]
    // InvalidThreadStateTransition(ThreadState, ThreadState),
    // #[error("TooManyThread")]
    // TooManyThread,
    // #[error("Signal")]
    // Signal(Signal),
    #[error("AssertError")]
    AssertError,
    #[error("ConcurrencyError")]
    ConcurrencyError,

    #[error("NotEnoughEnergy")]
    NotEnoughEnergy,

    #[error("Default")]
    Default,
}

pub struct ThreadContext<E: crate::vm::external::Engine> {
    pub scope_manager: ScopeManager,
    pub program: Program<E>,
}

pub struct Thread<P: SchedulingPolicy> {
    pub scheduler: Scheduler<P>,
    pub stack: Stack,
}

pub struct Runtime<E: crate::vm::external::Engine, P: SchedulingPolicy> {
    contexts: HashMap<E::TID, ThreadContext<E>>,
    threads: HashMap<E::TID, Thread<P>>,
}

impl<E: crate::vm::external::Engine, P: SchedulingPolicy> Default for Runtime<E, P> {
    fn default() -> Self {
        Self {
            contexts: HashMap::default(),
            threads: HashMap::default(),
        }
    }
}

impl<E: crate::vm::external::Engine, P: SchedulingPolicy> Runtime<E, P> {
    const INSTRUCTION_MAX_COUNT: usize = 1024;

    pub fn prepare(&mut self) {
        for Thread { scheduler, .. } in self.threads.values_mut() {
            scheduler.prepare();
        }
    }

    pub fn spawn(&mut self, engine: &mut E) -> Result<E::TID, RuntimeError> {
        let scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let program = crate::vm::program::Program::default();
        let scheduler = Scheduler::default();
        let stack = Stack::default();

        let tid = engine.spawn()?;
        self.contexts.insert(
            tid.clone(),
            ThreadContext {
                scope_manager,
                program,
            },
        );
        self.threads
            .insert(tid.clone(), Thread { scheduler, stack });
        Ok(tid)
    }

    pub fn context_of(&mut self, tid: &E::TID) -> Result<&mut ThreadContext<E>, RuntimeError> {
        self.contexts
            .get_mut(tid)
            .ok_or(RuntimeError::InvalidTID(0))
    }

    pub fn thread_with_context_of(
        &self,
        tid: &E::TID,
    ) -> Result<(&Thread<P>, &ThreadContext<E>), RuntimeError> {
        Ok((
            self.threads.get(tid).ok_or(RuntimeError::InvalidTID(0))?,
            self.contexts.get(tid).ok_or(RuntimeError::InvalidTID(0))?,
        ))
    }

    pub fn run(
        &mut self,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
    ) -> Result<(), RuntimeError> {
        for (tid, Thread { scheduler, stack }) in P::schedule::<E>(self.threads.iter_mut()) {
            let Some(ThreadContext { program, .. }) = self.contexts.get(tid) else {
                return Err(RuntimeError::Default);
            };
            let mut watchdog = Self::INSTRUCTION_MAX_COUNT;

            loop {
                match scheduler.run(program, stack, heap, stdio, engine)? {
                    std::ops::ControlFlow::Continue(_) => {
                        watchdog = watchdog.checked_sub(1).unwrap_or(0);
                        if watchdog == 0 {
                            break;
                        }
                        continue;
                    }
                    std::ops::ControlFlow::Break(_) => break,
                }
            }
        }
        Ok(())
    }
}

// pub const MAX_THREAD_COUNT: usize = 4;

// #[derive(Debug, Clone, PartialEq, Eq, Copy)]
// pub enum Player {
//     P1,
//     P2,
// }

// #[derive(Debug, Clone, PartialEq, Eq, Copy)]
// pub enum ThreadState {
//     IDLE,
//     WAITING,
//     SLEEPING(usize), // remaining maf until awakening
//     ACTIVE,
//     COMPLETED_MAF,
//     STARVED,
//     EXITED,
// }

// impl ThreadState {
//     pub fn is_noop(&self) -> bool {
//         match self {
//             ThreadState::IDLE => true,
//             ThreadState::WAITING => true,
//             ThreadState::SLEEPING(_) => true,
//             ThreadState::ACTIVE => false,
//             ThreadState::COMPLETED_MAF => true,
//             ThreadState::STARVED => true,
//             ThreadState::EXITED => true,
//         }
//     }

//     pub fn init_maf<E: crate::vm::external::Engine>(&mut self, engine: &mut E, program: &Program) {
//         let program_at_end = program.cursor_is_at_end();
//         match self {
//             ThreadState::IDLE => {
//                 if !program_at_end {
//                     *self = ThreadState::ACTIVE
//                 }
//             }
//             ThreadState::WAITING => {}
//             ThreadState::SLEEPING(0) => {
//                 if !program_at_end {
//                     *self = ThreadState::ACTIVE
//                 } else {
//                     *self = ThreadState::IDLE
//                 }
//             }
//             ThreadState::SLEEPING(n) => *n -= 1,
//             ThreadState::ACTIVE => {
//                 if program_at_end {
//                     *self = ThreadState::IDLE
//                 }
//             }
//             ThreadState::COMPLETED_MAF => {
//                 if !program_at_end {
//                     *self = ThreadState::ACTIVE
//                 } else {
//                     *self = ThreadState::IDLE
//                 }
//             }
//             ThreadState::EXITED => {}
//             ThreadState::STARVED => {
//                 let weight = program.current_instruction_weight::<E>();
//                 if engine.get_player_energy() >= weight {
//                     *self = ThreadState::ACTIVE
//                 }
//             }
//         }
//     }

//     pub fn init_mif<E: crate::vm::external::Engine>(&mut self, engine: &mut E, program: &Program) {
//         let program_at_end = program.cursor_is_at_end();
//         match self {
//             ThreadState::IDLE => {
//                 if !program_at_end {
//                     *self = ThreadState::ACTIVE
//                 }
//             }
//             ThreadState::WAITING => {}
//             ThreadState::SLEEPING(0) => {
//                 if !program_at_end {
//                     *self = ThreadState::ACTIVE
//                 } else {
//                     *self = ThreadState::IDLE
//                 }
//             }
//             ThreadState::SLEEPING(n) => {}
//             ThreadState::ACTIVE => {
//                 if program_at_end {
//                     *self = ThreadState::IDLE
//                 }
//             }
//             ThreadState::COMPLETED_MAF => {}
//             ThreadState::EXITED => {}
//             ThreadState::STARVED => {}
//         }
//     }

//     pub fn to(&mut self, dest: Self) -> Result<(), RuntimeError> {
//         let dest = match self {
//             ThreadState::IDLE => match dest {
//                 ThreadState::IDLE => dest,
//                 ThreadState::WAITING => dest,
//                 ThreadState::SLEEPING(_) => dest,
//                 ThreadState::ACTIVE => dest,
//                 ThreadState::COMPLETED_MAF => dest,
//                 ThreadState::STARVED => dest,
//                 ThreadState::EXITED => dest,
//             },
//             ThreadState::WAITING => match dest {
//                 ThreadState::IDLE => dest,
//                 ThreadState::WAITING => dest,
//                 ThreadState::SLEEPING(_) => {
//                     return Err(RuntimeError::InvalidThreadStateTransition(*self, dest))
//                 }
//                 ThreadState::ACTIVE => dest,
//                 ThreadState::COMPLETED_MAF => dest,
//                 ThreadState::STARVED => dest,
//                 ThreadState::EXITED => dest,
//             },
//             ThreadState::SLEEPING(_) => match dest {
//                 ThreadState::IDLE => dest,
//                 ThreadState::WAITING => {
//                     return Err(RuntimeError::InvalidThreadStateTransition(*self, dest))
//                 }
//                 ThreadState::SLEEPING(_) => dest,
//                 ThreadState::ACTIVE => dest,
//                 ThreadState::COMPLETED_MAF => dest,
//                 ThreadState::STARVED => dest,
//                 ThreadState::EXITED => dest,
//             },
//             ThreadState::ACTIVE => match dest {
//                 ThreadState::IDLE => dest,
//                 ThreadState::WAITING => dest,
//                 ThreadState::SLEEPING(_) => dest,
//                 ThreadState::ACTIVE => dest,
//                 ThreadState::COMPLETED_MAF => dest,
//                 ThreadState::STARVED => dest,
//                 ThreadState::EXITED => dest,
//             },
//             ThreadState::COMPLETED_MAF => match dest {
//                 ThreadState::IDLE => dest,
//                 ThreadState::WAITING => dest,
//                 ThreadState::SLEEPING(_) => dest,
//                 ThreadState::ACTIVE => dest,
//                 ThreadState::COMPLETED_MAF => dest,
//                 ThreadState::STARVED => dest,
//                 ThreadState::EXITED => dest,
//             },
//             ThreadState::EXITED => match dest {
//                 ThreadState::IDLE => {
//                     return Err(RuntimeError::InvalidThreadStateTransition(*self, dest))
//                 }
//                 ThreadState::WAITING => {
//                     return Err(RuntimeError::InvalidThreadStateTransition(*self, dest))
//                 }
//                 ThreadState::SLEEPING(_) => {
//                     return Err(RuntimeError::InvalidThreadStateTransition(*self, dest))
//                 }
//                 ThreadState::ACTIVE => {
//                     return Err(RuntimeError::InvalidThreadStateTransition(*self, dest))
//                 }
//                 ThreadState::COMPLETED_MAF => dest,
//                 ThreadState::EXITED => dest,
//                 ThreadState::STARVED => {
//                     return Err(RuntimeError::InvalidThreadStateTransition(*self, dest))
//                 }
//             },
//             ThreadState::STARVED => match dest {
//                 ThreadState::IDLE => dest,
//                 ThreadState::WAITING => dest,
//                 ThreadState::SLEEPING(_) => dest,
//                 ThreadState::ACTIVE => dest,
//                 ThreadState::COMPLETED_MAF => dest,
//                 ThreadState::STARVED => dest,
//                 ThreadState::EXITED => dest,
//             },
//         };
//         *self = dest;
//         Ok(())
//     }
// }

// #[derive(Debug, Clone)]
// pub struct Thread {
//     pub state: ThreadState,
//     pub scope: ScopeManager,
//     pub stack: Stack,
//     pub program: Program,
//     pub tid: Tid,
//     pub current_maf_instruction_count: usize,
// }

// #[derive(Debug, Clone)]
// pub struct Runtime {
//     pub p1_manager: PlayerThreadsManager,
//     pub p2_manager: PlayerThreadsManager,
// }

// #[derive(Debug, Clone)]
// pub struct PlayerThreadsManager {
//     pub threads: [Option<Thread>; MAX_THREAD_COUNT],
// }

// const THREAD_INIT_VALUE_NONE: Option<Thread> = None;
// impl PlayerThreadsManager {
//     pub fn new() -> Self {
//         Self {
//             threads: [THREAD_INIT_VALUE_NONE; MAX_THREAD_COUNT],
//         }
//     }

//     pub fn info(&self) -> String {
//         let mut buf = String::new();

//         for thread in self.threads.iter() {
//             if let Some(Thread {
//                 ref tid, ref state, ..
//             }) = thread
//             {
//                 buf.push_str(&format!(" Tid {tid} = {:?},", state));
//             }
//         }
//         buf
//     }

//     pub fn all_noop(&self) -> bool {
//         let mut noop = true;
//         for thread in &self.threads {
//             if let Some(Thread { state, .. }) = thread {
//                 if !state.is_noop() {
//                     noop = false
//                 }
//             }
//         }
//         noop
//     }

//     pub fn thread_count(&self) -> usize {
//         self.threads.iter().filter(|t| t.is_some()).count()
//     }

//     pub fn alive(&self) -> [bool; MAX_THREAD_COUNT] {
//         let mut buff = [false; MAX_THREAD_COUNT];
//         for i in 0..MAX_THREAD_COUNT {
//             buff[i] = self.threads[i].is_some();
//         }
//         buff
//     }

//     pub fn spawn<E: crate::vm::external::Engine>(&mut self, engine: &mut E) -> Result<usize, RuntimeError> {
//         if self.thread_count() >= MAX_THREAD_COUNT {
//             return Err(RuntimeError::TooManyThread);
//         }

//         let scope = ScopeManager::default();
//         let program = Program::default();
//         let stack = Stack::new();
//         let Some((tid, _)) = self.threads.iter().enumerate().find(|(i, t)| t.is_none()) else {
//             return Err(RuntimeError::TooManyThread);
//         };

//         self.threads[tid].replace(Thread {
//             scope,
//             stack,
//             program,
//             tid,
//             state: ThreadState::IDLE,
//             current_maf_instruction_count: 0,
//         });
//         engine.spawn(tid);
//         Ok(tid)
//     }

//     pub fn spawn_with_tid<E: crate::vm::external::Engine>(
//         &mut self,
//         tid: Tid,
//         engine: &mut E,
//     ) -> Result<(), RuntimeError> {
//         if self.thread_count() >= MAX_THREAD_COUNT {
//             return Err(RuntimeError::TooManyThread);
//         }
//         if tid >= MAX_THREAD_COUNT {
//             return Err(RuntimeError::InvalidTID(tid));
//         }
//         let scope = ScopeManager::default();
//         let program = Program::default();
//         let stack = Stack::new();

//         self.threads[tid].replace(Thread {
//             scope,
//             stack,
//             program,
//             tid,
//             state: ThreadState::IDLE,
//             current_maf_instruction_count: 0,
//         });
//         engine.spawn(tid);
//         Ok(())
//     }

//     pub fn close<E: crate::vm::external::Engine>(
//         &mut self,
//         tid: Tid,
//         engine: &mut E,
//     ) -> Result<(), RuntimeError> {
//         if tid >= MAX_THREAD_COUNT || self.threads[tid].is_none() {
//             return Err(RuntimeError::InvalidTID(tid));
//         }

//         let _ = self.threads[tid].take();
//         engine.close(tid);
//         Ok(())
//     }

//     pub fn spawn_with_scope(&mut self, scope: ScopeManager) -> Result<usize, RuntimeError> {
//         let program = Program::default();
//         let stack = Stack::new();
//         let Some((tid, _)) = self.threads.iter().enumerate().find(|(i, t)| t.is_none()) else {
//             return Err(RuntimeError::TooManyThread);
//         };
//         self.threads[tid].replace(Thread {
//             scope,
//             stack,
//             program,
//             tid,
//             state: ThreadState::IDLE,
//             current_maf_instruction_count: 0,
//         });
//         Ok(tid)
//     }
// }

// impl Runtime {
//     pub fn new() -> (Self, Heap, StdIO) {
//         (
//             Self {
//                 p1_manager: PlayerThreadsManager::new(),
//                 p2_manager: PlayerThreadsManager::new(),
//             },
//             Heap::new(),
//             StdIO::default(),
//         )
//     }

//     pub fn tid_info(&self) -> String {
//         let mut buf = String::new();
//         buf.push_str(&self.p1_manager.info());
//         buf.push_str(&self.p2_manager.info());
//         buf
//     }

//     pub fn spawn<E: crate::vm::external::Engine>(
//         &mut self,
//         player: Player,
//         engine: &mut E,
//     ) -> Result<Tid, RuntimeError> {
//         match player {
//             Player::P1 => self.p1_manager.spawn(engine),
//             Player::P2 => self.p2_manager.spawn(engine),
//         }
//     }

//     pub fn spawn_with_tid<E: crate::vm::external::Engine>(
//         &mut self,
//         player: Player,
//         tid: Tid,
//         engine: &mut E,
//     ) -> Result<(), RuntimeError> {
//         match player {
//             Player::P1 => self.p1_manager.spawn_with_tid(tid, engine),
//             Player::P2 => self.p2_manager.spawn_with_tid(tid, engine),
//         }
//     }
//     pub fn close<E: crate::vm::external::Engine>(
//         &mut self,
//         player: Player,
//         tid: Tid,
//         engine: &mut E,
//     ) -> Result<(), RuntimeError> {
//         match player {
//             Player::P1 => self.p1_manager.close(tid, engine),
//             Player::P2 => self.p2_manager.close(tid, engine),
//         }
//     }

//     pub fn spawn_with_scope(
//         &mut self,
//         player: Player,
//         scope: ScopeManager,
//     ) -> Result<usize, RuntimeError> {
//         match player {
//             Player::P1 => self.p1_manager.spawn_with_scope(scope),
//             Player::P2 => self.p2_manager.spawn_with_scope(scope),
//         }
//     }

//     pub fn get_mut<'runtime>(
//         &'runtime mut self,
//         player: Player,
//         tid: Tid,
//     ) -> Result<
//         (
//             &'runtime mut ScopeManager,
//             &'runtime mut Stack,
//             &mut crate::vm::program::Program,
//         ),
//         RuntimeError,
//     > {
//         match player {
//             Player::P1 => {
//                 if tid >= MAX_THREAD_COUNT {
//                     return Err(RuntimeError::InvalidTID(tid));
//                 }
//                 let Some(thread) = &mut self.p1_manager.threads[tid] else {
//                     return Err(RuntimeError::InvalidTID(tid));
//                 };
//                 Ok((&mut thread.scope, &mut thread.stack, &mut thread.program))
//             }
//             Player::P2 => {
//                 if tid >= MAX_THREAD_COUNT {
//                     return Err(RuntimeError::InvalidTID(tid));
//                 }
//                 let Some(thread) = &mut self.p2_manager.threads[tid] else {
//                     return Err(RuntimeError::InvalidTID(tid));
//                 };
//                 Ok((&mut thread.scope, &mut thread.stack, &mut thread.program))
//             }
//         }
//     }

//     pub fn iter_mut(&mut self) -> impl Iterator<Item = (Option<&mut Thread>, Option<&mut Thread>)> {
//         let p1_iter = self.p1_manager.threads.iter_mut().map(|t| t.as_mut());

//         let p2_iter = self.p2_manager.threads.iter_mut().map(|t| t.as_mut());

//         std::iter::zip(p1_iter, p2_iter)
//     }
// }
