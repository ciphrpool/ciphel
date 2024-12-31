use std::marker::PhantomData;

use super::{
    external::{
        ExternEventManager, ExternExecutionContext, ExternProcessIdentifier, ExternThreadIdentifier,
    },
    runtime::{Runtime, RuntimeError, RuntimeSnapshot},
    scheduler::{EventCallback, SchedulingPolicy},
};

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Signal<
    EC: ExternExecutionContext,
    PID: ExternProcessIdentifier,
    TID: ExternThreadIdentifier<PID>,
    EM: ExternEventManager<EC, PID, TID>,
> {
    Spawn,
    Exit,
    Close(TID),
    Sleep {
        time: usize,
        _phantom: PhantomData<PID>,
    },
    Join(TID),
    Wait,
    Wake(TID),
    WaitSTDIN,
    EventTrigger {
        tid: TID,
        trigger: u64,
    },
    EventRegistration {
        tid: TID,
        trigger: u64,
        callback: EventCallback<EC, PID, TID, EM>,
        conf: super::scheduler::EventConf,
    },
}

#[derive(Clone)]
pub enum SignalAction<
    EC: ExternExecutionContext,
    PID: ExternProcessIdentifier,
    TID: ExternThreadIdentifier<PID>,
    EM: ExternEventManager<EC, PID, TID>,
> {
    Spawn(TID),
    Exit(TID),
    Close(TID),
    Sleep {
        tid: TID,
        time: usize,
        _phantom: PhantomData<PID>,
    },
    Join {
        caller: TID,
        target: TID,
    },
    Wait {
        caller: TID,
    },
    Wake {
        caller: TID,
        target: TID,
    },
    WaitSTDIN(TID),
    EventTrigger {
        tid: TID,
        trigger: u64,
    },
    EventRegistration {
        tid: TID,
        trigger: u64,
        callback: EventCallback<EC, PID, TID, EM>,
        conf: super::scheduler::EventConf,
    },
}

pub enum SignalResult<E: crate::vm::external::Engine> {
    Ok(SignalAction<E::FunctionContext, E::PID, E::TID, E::Function>),
    Error,
}

pub struct SignalHandler<E: crate::vm::external::Engine> {
    snapshot: RuntimeSnapshot<E::PID, E::TID>,
    action_buffer: Vec<SignalAction<E::FunctionContext, E::PID, E::TID, E::Function>>,
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
    pub fn init(&mut self, snapshot: RuntimeSnapshot<E::PID, E::TID>) {
        self.snapshot = snapshot;
        self.action_buffer.clear();
    }

    fn handle(
        &mut self,
        caller: E::TID,
        signal: Signal<E::FunctionContext, E::PID, E::TID, E::Function>,
        engine: &mut E,
    ) -> SignalResult<E> {
        match signal {
            Signal::Spawn => {
                let Ok(tid) = engine.spawn(&caller.pid()) else {
                    return SignalResult::Error;
                };
                let action = SignalAction::Spawn(tid);
                self.action_buffer.push(action.clone());
                SignalResult::Ok(action)
            }
            Signal::Exit => {
                if engine.close(&caller.pid(), &caller).is_err() {
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

                if engine.close(&caller.pid(), &tid).is_err() {
                    return SignalResult::Error;
                };
                let action = SignalAction::Close(tid);
                self.action_buffer.push(action.clone());
                SignalResult::Ok(action)
            }
            Signal::Sleep { time, .. } => {
                let action = SignalAction::Sleep {
                    tid: caller.clone(),
                    time,
                    _phantom: PhantomData::default(),
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
            Signal::WaitSTDIN => {
                let action = SignalAction::WaitSTDIN(caller.clone());
                self.action_buffer.push(action.clone());
                SignalResult::Ok(action)
            }
            Signal::EventTrigger { tid, trigger } => {
                let action = SignalAction::EventTrigger { tid, trigger };
                self.action_buffer.push(action.clone());
                SignalResult::Ok(action)
            }
            Signal::EventRegistration {
                tid,
                trigger,
                callback,
                conf,
            } => {
                let action = SignalAction::EventRegistration {
                    tid,
                    trigger,
                    callback,
                    conf,
                };
                self.action_buffer.push(action.clone());
                SignalResult::Ok(action)
            }
        }
    }

    pub fn commit<P: SchedulingPolicy>(
        &mut self,
        runtime: &mut Runtime<E, P>,
    ) -> Result<(), (E::PID, RuntimeError)> {
        for action in self.action_buffer.iter() {
            // apply action in the runtime
            match action {
                SignalAction::Spawn(tid) => {
                    let _ = runtime.spawn_with_id(tid.clone()).map_err(|e| (tid.pid(),e))?;
                }
                SignalAction::Exit(tid) => {
                    runtime.close(tid.clone());
                }
                SignalAction::Close(tid) => {
                    runtime.close(tid.clone());
                }
                SignalAction::Sleep { tid, time, .. } => {
                    let _ = runtime.put_to_sleep_for(tid.clone(), *time).map_err(|e| (tid.pid(),e))?;
                }
                SignalAction::Join { caller, target } => {
                    let _ = runtime.join(caller.clone(), target.clone()).map_err(|e| (caller.pid(),e))?;
                }
                SignalAction::Wait { caller } => {
                    let _ = runtime.wait(caller.clone()).map_err(|e| (caller.pid(),e))?;
                }
                SignalAction::Wake { caller, target } => {
                    let _ = runtime.wake(caller.clone(), target.clone()).map_err(|e| (caller.pid(),e))?;
                }
                SignalAction::WaitSTDIN(tid) => {
                    let _ = runtime.wait_stdin(tid.clone()).map_err(|e| (tid.pid(),e))?;
                }
                SignalAction::EventTrigger { tid, trigger } => {
                    let _ = runtime.trigger(tid.clone(), *trigger).map_err(|e| (tid.pid(),e))?;
                }
                SignalAction::EventRegistration {
                    tid,
                    trigger,
                    callback,
                    conf,
                } => {
                    let _ = runtime.register_event(tid.clone(), *trigger, *callback, *conf).map_err(|e| (tid.pid(),e))?;
                }
            }
        }
        Ok(())
    }

    pub fn notify(
        &mut self,
        signal: Signal<E::FunctionContext, E::PID, E::TID, E::Function>,
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
