use std::{
    collections::{HashMap, VecDeque},
    marker::PhantomData,
};

use thiserror::Error;

use super::{
    allocator::{
        heap::HeapError,
        stack::{Stack, StackError},
        MemoryAddress,
    },
    external::{ExternProcessIdentifier, ExternThreadIdentifier},
    program::Program,
    scheduler::{
        Event, EventCallback, EventConf, EventExclusivity, EventKind, EventQueue, EventState,
        Scheduler, SchedulingPolicy,
    },
    GenerateCode,
};
use crate::vm::external::ExternEventManager;
use crate::{
    ast::modules::Module,
    semantic::{scope::scope::ScopeManager, Resolve},
    vm::signal::SignalHandler,
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

    #[error("Context Error")]
    ContextError,

    #[error("Default")]
    Default,
}

pub struct ThreadContext<E: crate::vm::external::Engine> {
    pub scope_manager: ScopeManager,
    pub program: Program<E>,
    pub state: ThreadState<E::PID, E::TID>,
}

pub struct Thread<P: SchedulingPolicy> {
    pub scheduler: Scheduler<P>,
    pub stack: Stack,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ThreadState<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> {
    IDLE,
    RUNNING,
    SLEEPING(usize),
    JOINING {
        target: TID,
        _phantom: PhantomData<PID>,
    },
    WAITING,
    WAITING_STDIN,
}

impl<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> Default
    for ThreadState<PID, TID>
{
    fn default() -> Self {
        Self::RUNNING
    }
}

impl<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> ThreadState<PID, TID> {
    pub fn init_maf<E: crate::vm::external::Engine<TID = TID, PID = PID>>(
        &mut self,
        tid: E::TID,
        snapshot: &RuntimeSnapshot<E::PID, E::TID>,
        stdio: &mut super::stdio::StdIO,
        engine: &mut E,
    ) {
        match self {
            ThreadState::IDLE => {}
            ThreadState::RUNNING => {}
            ThreadState::SLEEPING(time) => match time.checked_sub(1) {
                Some(0) => *self = ThreadState::RUNNING,
                Some(time) => *self = ThreadState::SLEEPING(time),
                None => *self = ThreadState::IDLE, // should never happens
            },
            ThreadState::JOINING { ref target, .. } => {
                //if let Some(ThreadState::)
                match snapshot.states.get(target) {
                    Some(ThreadState::IDLE) | None => {
                        *self = ThreadState::RUNNING;
                    }
                    Some(_) => {}
                }
            }
            ThreadState::WAITING => {}
            ThreadState::WAITING_STDIN => {
                if let Some(data) = engine.stdin_scan(tid) {
                    *self = ThreadState::RUNNING;
                    stdio.stdin.write(data);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct RuntimeSnapshot<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> {
    pub states: HashMap<TID, ThreadState<PID, TID>>,
}

impl<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> Default
    for RuntimeSnapshot<PID, TID>
{
    fn default() -> Self {
        Self {
            states: HashMap::default(),
        }
    }
}

pub struct Runtime<E: crate::vm::external::Engine, P: SchedulingPolicy> {
    pub modules: HashMap<E::PID, Vec<Module>>,
    contexts: HashMap<E::TID, ThreadContext<E>>,
    threads: HashMap<E::TID, Thread<P>>,
    pub(crate) event_queue: EventQueue<E>,
}

impl<E: crate::vm::external::Engine, P: SchedulingPolicy> Default for Runtime<E, P> {
    fn default() -> Self {
        Self {
            modules: HashMap::default(),
            contexts: HashMap::default(),
            threads: HashMap::default(),
            event_queue: EventQueue::<E>::default(),
        }
    }
}

impl<E: crate::vm::external::Engine, P: SchedulingPolicy> Runtime<E, P> {
    pub fn snapshot(&self) -> RuntimeSnapshot<E::PID, E::TID> {
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

    pub fn spawn(&mut self, pid: E::PID, engine: &mut E) -> Result<E::TID, RuntimeError> {
        let tid = engine.spawn(&pid)?;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let mut program = crate::vm::program::Program::default();

        // re-import all modules
        if let Some(modules) = self.modules.get_mut(&pid) {
            for module in modules.iter_mut() {
                module
                    .resolve::<E>(&mut scope_manager, None, &(), &mut ())
                    .map_err(|err| RuntimeError::Default)?;
                module
                    .gencode::<E>(
                        &mut scope_manager,
                        None,
                        &mut program,
                        &crate::vm::CodeGenerationContext::default(),
                    )
                    .map_err(|err| RuntimeError::Default)?;
                scope_manager.modules.push(module.clone());
            }
        }

        let scheduler = Scheduler::default();
        let stack = Stack::default();

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

    pub fn close(&mut self, tid: E::TID) {
        self.contexts.remove(&tid);
        self.threads.remove(&tid);
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
        *state = ThreadState::JOINING {
            target,
            _phantom: PhantomData::default(),
        };
        Ok(())
    }

    pub fn wait(&mut self, caller: E::TID) -> Result<(), RuntimeError> {
        let Some(ThreadContext { state, .. }) = self.contexts.get_mut(&caller) else {
            return Err(RuntimeError::Default);
        };
        *state = ThreadState::WAITING;
        Ok(())
    }

    pub fn wait_stdin(&mut self, caller: E::TID) -> Result<(), RuntimeError> {
        let Some(ThreadContext { state, .. }) = self.contexts.get_mut(&caller) else {
            return Err(RuntimeError::Default);
        };
        *state = ThreadState::WAITING_STDIN;
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

    pub fn trigger(&mut self, caller: E::TID, signal: u64) -> Result<(), RuntimeError> {
        let caller_pid = caller.pid();
        let mut triggered = Vec::new();

        for (e_tid, events) in self.event_queue.events.iter_mut() {
            let mut i = 0;
            while i < events.len() {
                let should_trigger = {
                    let event = &events[i];
                    ((EventExclusivity::PerPID == event.conf.exclu && caller_pid == event.pid)
                        || (EventExclusivity::PerTID == event.conf.exclu && event.tid != caller))
                        && (event.callback.manager.event_trigger(signal, event.trigger))
                        && (EventState::IDLE == event.state)
                };

                if should_trigger {
                    if !self.threads.contains_key(&events[i].tid) {
                        return Err(RuntimeError::Default);
                    }
                    let event = events.swap_remove(i);
                    triggered.push((e_tid.clone(), event));
                } else {
                    i += 1;
                }
            }
        }

        // Move triggered events
        for (tid, mut event) in triggered {
            if self.event_queue.current_events.contains_key(&tid) {
                event.state = EventState::Triggered;
                self.event_queue
                    .running_events
                    .entry(tid)
                    .or_insert_with(VecDeque::new)
                    .push_back(event);
            } else {
                event.state = EventState::Running;
                self.event_queue.current_events.insert(tid, event);
            }
        }

        Ok(())
    }

    pub fn register_event(
        &mut self,
        caller: E::TID,
        trigger: u64,
        callback: super::scheduler::EventCallback<E::FunctionContext, E::PID, E::TID, E::Function>,
        conf: EventConf,
    ) -> Result<(), RuntimeError> {
        self.event_queue
            .events
            .entry(caller)
            .or_insert_with(Vec::new)
            .push(Event {
                pid: caller.pid(),
                tid: caller,
                trigger,
                callback,
                conf,
                state: EventState::default(),
            });
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
    ) -> Result<(), (E::PID,RuntimeError)> {
        stdio.push_asm_info(engine, E::PID::default(), "START MAF");

        let mut signal_handler = SignalHandler::default();
        let snapshot = self.snapshot();

        for (tid, ThreadContext { state, .. }) in self.contexts.iter_mut() {
            state.init_maf(tid.clone(), &snapshot, stdio, engine);
        }
        signal_handler.init(snapshot);

        for (tid, Thread { scheduler, stack }) in self.threads.iter_mut() {
            let Some(ThreadContext { program, state, .. }) = self.contexts.get(tid) else {
                return Err((tid.pid(),RuntimeError::ContextError));
            };
            scheduler.policy.init_maf::<E>(tid, state);
        }

        for (tid, Thread { scheduler, stack }) in P::schedule::<E>(self.threads.iter_mut()) {
            let Some(ThreadContext {
                ref program, state, ..
            }) = self.contexts.get_mut(tid)
            else {
                return Err((tid.pid(),RuntimeError::ContextError));
            };
            scheduler.cursor.update(program, state);

            let context = &crate::vm::scheduler::ExecutionContext {
                external: E::FunctionContext::default(),
                tid: tid.clone(),
                pid: tid.pid(),
            };

            self.event_queue.prepare(
                tid.clone(),
                state,
                program,
                stack,
                heap,
                stdio,
                engine,
                context,
            ).map_err(|e| (tid.pid(),e))?;

            if ThreadState::RUNNING != *state {
                continue;
            }

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
                    self.event_queue.current_events.get_mut(tid),
                    context,
                ).map_err(|e| (tid.pid(),e))? {
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

        stdio.push_asm_info(engine, E::PID::default(), "END MAF");
        Ok(())
    }
}
