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
    scheduler::{Scheduler, SchedulingPolicy},
};

#[derive(Debug, Clone, Error)]
pub enum RuntimeError {
    #[error("StackError : {0}")]
    StackError(#[from] StackError),
    #[error("HeapError : {0}")]
    HeapError(#[from] HeapError),

    #[error("MEMORY VIOLATION")]
    MemoryViolation,
    #[error("Deserialization")]
    Deserialization,
    #[error("UnsupportedOperation")]
    UnsupportedOperation,
    #[error("MathError")]
    MathError,
    #[error("CodeSegmentation")]
    CodeSegmentation,
    #[error("IndexOutOfBound")]
    IndexOutOfBound,

    #[error("SignalError")]
    SignalError,

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
    pub state: ThreadState<E::TID>,
}

pub struct Thread<P: SchedulingPolicy> {
    pub scheduler: Scheduler<P>,
    pub stack: Stack,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ThreadState<TID: ExternThreadIdentifier> {
    IDLE,
    RUNNING,
    SLEEPING(usize),
    JOINING { target: TID },
    WAITING,
}

impl<TID: ExternThreadIdentifier> Default for ThreadState<TID> {
    fn default() -> Self {
        Self::RUNNING
    }
}

impl<TID: ExternThreadIdentifier> ThreadState<TID> {
    pub fn init_maf(&mut self, snapshot: &RuntimeSnapshot<TID>) {
        match self {
            ThreadState::IDLE => {}
            ThreadState::RUNNING => {}
            ThreadState::SLEEPING(time) => match time.checked_sub(1) {
                Some(0) => *self = ThreadState::RUNNING,
                Some(time) => *self = ThreadState::SLEEPING(time),
                None => *self = ThreadState::IDLE, // should never happens
            },
            ThreadState::JOINING { ref target } => {
                //if let Some(ThreadState::)
                match snapshot.states.get(target) {
                    Some(ThreadState::IDLE) | None => {
                        *self = ThreadState::RUNNING;
                    }
                    Some(_) => {}
                }
            }
            ThreadState::WAITING => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Signal<TID: ExternThreadIdentifier> {
    Spawn,
    Exit,
    Close(TID),
    Sleep { time: usize },
    Join(TID),
    Wait,
    Wake(TID),
}

#[derive(Clone)]
pub enum SignalAction<TID: ExternThreadIdentifier> {
    Spawn(TID),
    Exit(TID),
    Close(TID),
    Sleep { tid: TID, time: usize },
    Join { caller: TID, target: TID },
    Wait { caller: TID },
    Wake { caller: TID, target: TID },
}

pub enum SignalResult<E: crate::vm::external::Engine> {
    Ok(SignalAction<E::TID>),
    Error,
}

pub struct SignalHandler<E: crate::vm::external::Engine> {
    snapshot: RuntimeSnapshot<E::TID>,
    action_buffer: Vec<SignalAction<E::TID>>,
}

impl<E: crate::vm::external::Engine> Default for SignalHandler<E> {
    fn default() -> Self {
        Self {
            snapshot: RuntimeSnapshot::default(),
            action_buffer: Vec::default(),
        }
    }
}

impl<E: crate::vm::external::Engine> SignalHandler<E> {
    pub fn init(&mut self, snapshot: RuntimeSnapshot<E::TID>) {
        self.snapshot = snapshot;
        self.action_buffer.clear();
    }

    fn handle(
        &mut self,
        caller: E::TID,
        signal: Signal<E::TID>,
        engine: &mut E,
    ) -> SignalResult<E> {
        match signal {
            Signal::Spawn => {
                let Ok(tid) = engine.spawn() else {
                    return SignalResult::Error;
                };
                let action = SignalAction::Spawn(tid);
                self.action_buffer.push(action.clone());
                SignalResult::Ok(action)
            }
            Signal::Exit => {
                if engine.close(&caller).is_err() {
                    return SignalResult::Error;
                };
                let action = SignalAction::Exit(caller);
                self.action_buffer.push(action.clone());
                SignalResult::Ok(action)
            }
            Signal::Close(tid) => {
                if !self.snapshot.states.contains_key(&tid) {
                    return SignalResult::Error;
                }

                if engine.close(&tid).is_err() {
                    return SignalResult::Error;
                };
                let action = SignalAction::Close(tid);
                self.action_buffer.push(action.clone());
                SignalResult::Ok(action)
            }
            Signal::Sleep { time } => {
                let action = SignalAction::Sleep {
                    tid: caller.clone(),
                    time,
                };
                self.action_buffer.push(action.clone());
                SignalResult::Ok(action)
            }
            Signal::Join(target) => {
                if !self.snapshot.states.contains_key(&target) {
                    return SignalResult::Error;
                }

                let action = SignalAction::Join {
                    caller: caller.clone(),
                    target,
                };
                self.action_buffer.push(action.clone());
                SignalResult::Ok(action)
            }
            Signal::Wait => {
                let action = SignalAction::Wait {
                    caller: caller.clone(),
                };
                self.action_buffer.push(action.clone());
                SignalResult::Ok(action)
            }
            Signal::Wake(target) => {
                if !self.snapshot.states.contains_key(&target) {
                    return SignalResult::Error;
                }

                let action = SignalAction::Wake {
                    caller: caller.clone(),
                    target,
                };
                self.action_buffer.push(action.clone());
                SignalResult::Ok(action)
            }
        }
    }

    pub fn commit<P: SchedulingPolicy>(
        &mut self,
        runtime: &mut Runtime<E, P>,
    ) -> Result<(), RuntimeError> {
        for action in self.action_buffer.iter() {
            // apply action in the runtime
            match action {
                SignalAction::Spawn(tid) => {
                    let _ = runtime.spawn_with_id(tid.clone())?;
                }
                SignalAction::Exit(tid) => {
                    let _ = runtime.close(tid.clone())?;
                }
                SignalAction::Close(tid) => {
                    let _ = runtime.close(tid.clone())?;
                }
                SignalAction::Sleep { tid, time } => {
                    let _ = runtime.put_to_sleep_for(tid.clone(), *time)?;
                }
                SignalAction::Join { caller, target } => {
                    let _ = runtime.join(caller.clone(), target.clone())?;
                }
                SignalAction::Wait { caller } => {
                    let _ = runtime.wait(caller.clone())?;
                }
                SignalAction::Wake { caller, target } => {
                    let _ = runtime.wake(caller.clone(), target.clone())?;
                }
            }
        }
        Ok(())
    }

    pub fn notify(
        &mut self,
        signal: Signal<E::TID>,
        stack: &mut crate::vm::allocator::stack::Stack,
        engine: &mut E,
        tid: E::TID,
        callback: fn(
            SignalResult<E>,
            &mut crate::vm::allocator::stack::Stack,
        ) -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        let result = self.handle(tid, signal, engine);
        callback(result, stack)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct RuntimeSnapshot<TID: ExternThreadIdentifier> {
    pub states: HashMap<TID, ThreadState<TID>>,
}

impl<TID: ExternThreadIdentifier> Default for RuntimeSnapshot<TID> {
    fn default() -> Self {
        Self {
            states: HashMap::default(),
        }
    }
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
    pub fn snapshot(&self) -> RuntimeSnapshot<E::TID> {
        let mut states = HashMap::with_capacity(self.contexts.len());
        for (tid, ThreadContext { state, .. }) in self.contexts.iter() {
            states.insert(tid.clone(), state.clone());
        }
        RuntimeSnapshot { states }
    }

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
                state: ThreadState::default(),
            },
        );
        self.threads
            .insert(tid.clone(), Thread { scheduler, stack });
        Ok(tid)
    }

    pub fn spawn_with_id(&mut self, tid: E::TID) -> Result<(), RuntimeError> {
        let scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let program = crate::vm::program::Program::default();
        let scheduler = Scheduler::default();
        let stack = Stack::default();
        let state = ThreadState::default();

        self.contexts.insert(
            tid.clone(),
            ThreadContext {
                scope_manager,
                program,
                state,
            },
        );
        self.threads
            .insert(tid.clone(), Thread { scheduler, stack });
        Ok(())
    }

    pub fn close(&mut self, tid: E::TID) -> Result<(), RuntimeError> {
        self.contexts.remove(&tid);
        self.threads.remove(&tid);
        Ok(())
    }

    pub fn put_to_sleep_for(&mut self, tid: E::TID, time: usize) -> Result<(), RuntimeError> {
        let Some(ThreadContext { state, .. }) = self.contexts.get_mut(&tid) else {
            return Err(RuntimeError::Default);
        };
        *state = ThreadState::SLEEPING(time);
        Ok(())
    }

    pub fn join(&mut self, caller: E::TID, target: E::TID) -> Result<(), RuntimeError> {
        let Some(ThreadContext { state, .. }) = self.contexts.get_mut(&caller) else {
            return Err(RuntimeError::Default);
        };
        *state = ThreadState::JOINING { target };
        Ok(())
    }

    pub fn wait(&mut self, caller: E::TID) -> Result<(), RuntimeError> {
        let Some(ThreadContext { state, .. }) = self.contexts.get_mut(&caller) else {
            return Err(RuntimeError::Default);
        };
        *state = ThreadState::WAITING;
        Ok(())
    }

    pub fn wake(&mut self, caller: E::TID, target: E::TID) -> Result<(), RuntimeError> {
        let Some(ThreadContext { state, .. }) = self.contexts.get_mut(&target) else {
            return Err(RuntimeError::Default);
        };
        if ThreadState::WAITING == *state {
            *state = ThreadState::RUNNING;
        }
        Ok(())
    }

    pub fn context_of(&mut self, tid: &E::TID) -> Result<&mut ThreadContext<E>, RuntimeError> {
        self.contexts.get_mut(tid).ok_or(RuntimeError::Default)
    }

    pub fn thread_with_context_of(
        &self,
        tid: &E::TID,
    ) -> Result<(&Thread<P>, &ThreadContext<E>), RuntimeError> {
        Ok((
            self.threads.get(tid).ok_or(RuntimeError::Default)?,
            self.contexts.get(tid).ok_or(RuntimeError::Default)?,
        ))
    }

    pub fn run(
        &mut self,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
    ) -> Result<(), RuntimeError> {
        let mut signal_handler = SignalHandler::default();
        let snapshot = self.snapshot();
        for (tid, ThreadContext { state, .. }) in self.contexts.iter_mut() {
            state.init_maf(&snapshot);
        }
        signal_handler.init(snapshot);

        for (tid, Thread { scheduler, stack }) in self.threads.iter_mut() {
            let Some(ThreadContext { program, state, .. }) = self.contexts.get(tid) else {
                return Err(RuntimeError::Default);
            };
            scheduler.policy.init_maf::<E>(tid, state);
        }

        for (tid, Thread { scheduler, stack }) in P::schedule::<E>(self.threads.iter_mut()) {
            let Some(ThreadContext {
                ref program, state, ..
            }) = self.contexts.get_mut(tid)
            else {
                return Err(RuntimeError::Default);
            };
            scheduler.cursor.update(program, state);

            if ThreadState::RUNNING != *state {
                continue;
            }
            dbg!((tid, &state));

            scheduler.policy.init_watchdog();

            loop {
                match scheduler.run(
                    tid.clone(),
                    state,
                    program,
                    stack,
                    heap,
                    stdio,
                    engine,
                    &mut signal_handler,
                )? {
                    std::ops::ControlFlow::Continue(_) => match scheduler.policy.watchdog() {
                        std::ops::ControlFlow::Continue(_) => continue,
                        std::ops::ControlFlow::Break(_) => {
                            *state = ThreadState::IDLE;
                            break;
                        }
                    },
                    std::ops::ControlFlow::Break(_) => {
                        *state = ThreadState::IDLE;
                        break;
                    }
                }
            }
        }

        let _ = signal_handler.commit(self)?;
        dbg!("END MAF");
        Ok(())
    }
}
