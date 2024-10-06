use super::{
    external::ExternThreadIdentifier,
    runtime::{Runtime, RuntimeError, RuntimeSnapshot},
    scheduler::SchedulingPolicy,
};

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Signal<TID: ExternThreadIdentifier> {
    Spawn,
    Exit,
    Close(TID),
    Sleep { time: usize },
    Join(TID),
    Wait,
    Wake(TID),
    WaitSTDIN,
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
    WaitSTDIN(TID),
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
            Signal::WaitSTDIN => {
                let action = SignalAction::WaitSTDIN(caller.clone());
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
                SignalAction::WaitSTDIN(tid) => {
                    let _ = runtime.wait_stdin(tid.clone())?;
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
