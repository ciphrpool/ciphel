use std::marker::PhantomData;

use super::{
    external::{ExternProcessIdentifier, ExternThreadIdentifier},
    runtime::{Runtime, RuntimeError, RuntimeSnapshot},
    scheduler::SchedulingPolicy,
};

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Signal<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> {
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
        callback: super::allocator::MemoryAddress,
        conf: super::scheduler::EventConf,
    },
}

#[derive(Clone)]
pub enum SignalAction<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> {
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
        callback: super::allocator::MemoryAddress,
        conf: super::scheduler::EventConf,
    },
}

pub enum SignalResult<E: crate::vm::external::Engine> {
    Ok(SignalAction<E::PID, E::TID>),
    Error,
}

pub struct SignalHandler<E: crate::vm::external::Engine> {
    snapshot: RuntimeSnapshot<E::PID, E::TID>,
    action_buffer: Vec<SignalAction<E::PID, E::TID>>,
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
        signal: Signal<E::PID, E::TID>,
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
                SignalAction::Sleep { tid, time, .. } => {
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
                SignalAction::WaitSTDIN(tid) => {
                    let _ = runtime.wait_stdin(tid.clone())?;
                }
                SignalAction::EventTrigger { tid, trigger } => {
                    let _ = runtime.trigger(tid.clone(), *trigger)?;
                }
                SignalAction::EventRegistration {
                    tid,
                    trigger,
                    callback,
                    conf,
                } => {
                    let _ = runtime.register_event(tid.clone(), *trigger, *callback, *conf)?;
                }
            }
        }
        Ok(())
    }

    pub fn notify(
        &mut self,
        signal: Signal<E::PID, E::TID>,
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
