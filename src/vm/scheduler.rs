use std::{
    ops::ControlFlow,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use crate::vm::{
    casm::Casm,
    core::thread::{sig_close, sig_exit, sig_join, sig_sleep, sig_spawn, sig_wait, sig_wake},
    vm::{Signal, Thread, ThreadState, MAX_THREAD_COUNT},
};

use super::{
    allocator::heap::Heap,
    core::thread::sig_wait_stdin,
    stdio::StdIO,
    vm::{CasmMetadata, Executable, GameEngineStaticFn, Player, Runtime, RuntimeError, Tid},
};

pub const INSTRUCTION_MAX_COUNT: usize = 64;

#[derive(Debug, Clone, Copy)]
pub enum WaitingStatus {
    Join {
        join_tid: Tid,
        to_be_completed_tid: Tid,
    },
    ForStdin(Tid),
}

#[derive(Debug, Clone)]
pub struct SchedulerContext {
    pub threads: [bool; MAX_THREAD_COUNT],
    pub wakingup_tid: [bool; MAX_THREAD_COUNT],
    pub spawned_tid: [bool; MAX_THREAD_COUNT],
    pub closed_tid: [bool; MAX_THREAD_COUNT],
}

impl SchedulerContext {
    pub fn new() -> Self {
        Self {
            threads: [false; MAX_THREAD_COUNT],
            wakingup_tid: [false; MAX_THREAD_COUNT],
            spawned_tid: [false; MAX_THREAD_COUNT],
            closed_tid: [false; MAX_THREAD_COUNT],
        }
    }

    pub fn thread_count(&self) -> usize {
        self.threads.iter().filter(|t| **t).count()
    }

    pub fn is_alive(&self, tid: Tid) -> Result<bool, RuntimeError> {
        if tid >= MAX_THREAD_COUNT {
            return Err(RuntimeError::InvalidTID(tid));
        }
        Ok(self.threads[tid])
    }

    pub fn request_spawn(&mut self) -> Result<Tid, RuntimeError> {
        let mut tid = None;
        for i in 0..MAX_THREAD_COUNT {
            if !self.threads[i] && !self.spawned_tid[i] {
                tid = Some(i);
                self.spawned_tid[i] = true;
                break;
            }
        }
        tid.ok_or(RuntimeError::TooManyThread)
    }
    pub fn request_close(&mut self, tid: Tid) -> Result<(), RuntimeError> {
        if tid >= MAX_THREAD_COUNT {
            return Err(RuntimeError::InvalidTID(tid));
        }
        if self.threads[tid] && !self.closed_tid[tid] {
            self.closed_tid[tid] = true;
            Ok(())
        } else {
            Err(RuntimeError::InvalidTID(tid))
        }
    }

    pub fn request_wake(&mut self, tid: Tid) -> Result<(), RuntimeError> {
        if tid >= MAX_THREAD_COUNT {
            return Err(RuntimeError::InvalidTID(tid));
        }
        if self.threads[tid] {
            self.wakingup_tid[tid] = true;
            Ok(())
        } else {
            Err(RuntimeError::InvalidTID(tid))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scheduler {
    p1_context: SchedulerContext,
    p2_context: SchedulerContext,
    p1_waiting_list: Arc<Mutex<Vec<WaitingStatus>>>,
    p2_waiting_list: Arc<Mutex<Vec<WaitingStatus>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            p1_waiting_list: Default::default(),
            p2_waiting_list: Default::default(),
            p1_context: SchedulerContext::new(),
            p2_context: SchedulerContext::new(),
        }
    }

    pub fn prepare(&mut self, runtime: &mut Runtime) {
        self.p1_context.threads = runtime.p1_manager.alive();
        self.p1_context.wakingup_tid = [false; MAX_THREAD_COUNT];
        self.p1_context.spawned_tid = [false; MAX_THREAD_COUNT];
        self.p1_context.closed_tid = [false; MAX_THREAD_COUNT];

        self.p2_context.threads = runtime.p2_manager.alive();
        self.p2_context.wakingup_tid = [false; MAX_THREAD_COUNT];
        self.p2_context.spawned_tid = [false; MAX_THREAD_COUNT];
        self.p2_context.closed_tid = [false; MAX_THREAD_COUNT];
    }

    fn run_minor_frame<G: crate::GameEngineStaticFn>(
        &mut self,
        player: crate::vm::vm::Player,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        thread: &mut Thread,
    ) -> Result<(), RuntimeError> {
        thread.state.init_mif(engine, &thread.program);

        match self.wake_waiting_threads(player, &mut thread.state, &thread.tid)? {
            ControlFlow::Continue(_) => {}
            ControlFlow::Break(weight) => {
                thread.current_maf_instruction_count += weight;
                if thread.current_maf_instruction_count >= INSTRUCTION_MAX_COUNT {
                    thread.state.to(ThreadState::COMPLETED_MAF);
                }
                return Ok(());
            }
        }

        let context = match player {
            Player::P1 => &mut self.p1_context,
            Player::P2 => &mut self.p2_context,
        };
        stdio.push_casm_info(engine, &format!("MINOR FRAME ::tid {0}\n", thread.tid));
        while thread.current_maf_instruction_count < INSTRUCTION_MAX_COUNT {
            if thread.program.cursor_is_at_end() {
                let _ = thread.state.to(ThreadState::IDLE);
                break;
            }
            let return_status = thread.program.evaluate(|program, instruction| {
                instruction.name(stdio, program, engine);
                let weight = <Casm as CasmMetadata<G>>::weight(instruction).get();

                engine.consume_energy(weight)?;

                thread.current_maf_instruction_count += weight;

                match instruction.execute(
                    program,
                    &mut thread.stack,
                    heap,
                    stdio,
                    engine,
                    thread.tid,
                ) {
                    Ok(_) => Ok(()),
                    Err(RuntimeError::Signal(signal)) => match signal {
                        Signal::SPAWN => sig_spawn(context, program, &mut thread.stack),
                        Signal::CLOSE(tid) => sig_close(tid, context, program, &mut thread.stack),
                        Signal::WAIT => sig_wait(&mut thread.state, program),
                        Signal::WAKE(wake_tid) => {
                            sig_wake(wake_tid, context, program, &mut thread.stack)
                        }
                        Signal::SLEEP(nb_maf) => sig_sleep(&nb_maf, &mut thread.state, program),
                        Signal::JOIN(join_tid) => {
                            let mut waiting_list = match player {
                                Player::P1 => self
                                    .p1_waiting_list
                                    .as_ref()
                                    .lock()
                                    .map_err(|_| RuntimeError::ConcurrencyError)?,

                                Player::P2 => self
                                    .p2_waiting_list
                                    .as_ref()
                                    .lock()
                                    .map_err(|_| RuntimeError::ConcurrencyError)?,
                            };

                            let _ = sig_join(
                                thread.tid,
                                join_tid,
                                context,
                                &mut thread.state,
                                program,
                                &mut thread.stack,
                                &mut waiting_list,
                            )?;
                            drop(waiting_list);
                            Ok(())
                        }
                        Signal::EXIT => {
                            let mut waiting_list = match player {
                                Player::P1 => self
                                    .p1_waiting_list
                                    .as_ref()
                                    .lock()
                                    .map_err(|_| RuntimeError::ConcurrencyError)?,

                                Player::P2 => self
                                    .p2_waiting_list
                                    .as_ref()
                                    .lock()
                                    .map_err(|_| RuntimeError::ConcurrencyError)?,
                            };
                            let _ = sig_exit(
                                thread.tid,
                                context,
                                &mut thread.state,
                                &mut waiting_list,
                            )?;
                            drop(waiting_list);
                            Ok(())
                        }
                        Signal::WAIT_STDIN => {
                            let mut waiting_list = match player {
                                Player::P1 => self
                                    .p1_waiting_list
                                    .as_ref()
                                    .lock()
                                    .map_err(|_| RuntimeError::ConcurrencyError)?,

                                Player::P2 => self
                                    .p2_waiting_list
                                    .as_ref()
                                    .lock()
                                    .map_err(|_| RuntimeError::ConcurrencyError)?,
                            };

                            let _ =
                                sig_wait_stdin(thread.tid, &mut thread.state, &mut waiting_list)?;
                            drop(waiting_list);
                            Ok(())
                        }
                    },
                    Err(err) => {
                        stdio.push_casm_info(
                            engine,
                            &format!("RUNTIME ERROR :: {:?} in {:?}", err, instruction),
                        );
                        program.catch(err)
                    }
                }
            });

            match return_status {
                Ok(_) => {
                    if thread.program.cursor_is_at_end() {
                        let _ = thread.state.to(ThreadState::IDLE);
                    }
                }
                Err(RuntimeError::Signal(signal)) => match signal {
                    Signal::EXIT
                    | Signal::WAIT
                    | Signal::SLEEP(_)
                    | Signal::JOIN(_)
                    | Signal::WAIT_STDIN => {
                        break;
                    }
                    _ => {}
                },
                Err(e @ RuntimeError::AssertError) => {
                    return Err(e);
                }
                Err(e) => {
                    thread.program.cursor_to_end();
                    let _ = thread.state.to(ThreadState::IDLE);

                    return Err(e);
                }
            }
            if thread.current_maf_instruction_count >= INSTRUCTION_MAX_COUNT {
                thread.state.to(ThreadState::COMPLETED_MAF);
            }
        }
        Ok(())
    }

    pub fn run_major_frame<G: crate::GameEngineStaticFn>(
        &mut self,
        runtime: &mut Runtime,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        // Wake thread waiting on stdin
        self.wake_for_stdin(Player::P1, runtime, stdio, engine)?;
        self.wake_for_stdin(Player::P2, runtime, stdio, engine)?;

        let info = runtime.tid_info();
        stdio.push_casm_info(engine, &format!("MAJOR FRAME START : {info}"));

        for (p1, p2) in runtime.iter_mut() {
            if let Some(p1) = p1 {
                p1.state.init_maf(engine, &p1.program);
                p1.current_maf_instruction_count = 0;
            }

            if let Some(p2) = p2 {
                p2.state.init_maf(engine, &p2.program);
                p2.current_maf_instruction_count = 0;
            }
        }

        loop {
            for (p1, p2) in runtime.iter_mut() {
                if let Some(p1) = p1 {
                    let result = self.run_minor_frame(Player::P1, heap, stdio, engine, p1);
                    if let Err(err) = result {
                        stdio.print_stderr(engine, &err.to_string());
                        if let RuntimeError::NotEnoughEnergy = err {
                            let _ = p1.state.to(ThreadState::STARVED);
                        }
                    }
                }
                if let Some(p2) = p2 {
                    let result = self.run_minor_frame(Player::P2, heap, stdio, engine, p2);
                    if let Err(err) = result {
                        stdio.print_stderr(engine, &err.to_string());
                        if let RuntimeError::NotEnoughEnergy = err {
                            let _ = p2.state.to(ThreadState::STARVED);
                        }
                    }
                }
            }

            // Waking up needed thread
            for (p1, p2) in runtime.iter_mut() {
                if let Some(p1) = p1 {
                    if p1.tid > MAX_THREAD_COUNT {
                        return Err(RuntimeError::InvalidTID(p1.tid));
                    }
                    if self.p1_context.wakingup_tid[p1.tid] {
                        let _ = p1.state.to(ThreadState::ACTIVE);
                        self.p1_context.wakingup_tid[p1.tid] = false;
                    }
                }
                if let Some(p2) = p2 {
                    if p2.tid > MAX_THREAD_COUNT {
                        return Err(RuntimeError::InvalidTID(p2.tid));
                    }
                    if self.p2_context.wakingup_tid[p2.tid] {
                        let _ = p2.state.to(ThreadState::ACTIVE);
                        self.p2_context.wakingup_tid[p2.tid] = false;
                    }
                }
            }
            if runtime.p1_manager.all_noop() && runtime.p2_manager.all_noop() {
                break;
            }
        }
        stdio.push_casm_info(engine, "MAJOR FRAME END".into());
        self.conclude(runtime, engine)?;
        Ok(())
    }

    fn conclude<G: GameEngineStaticFn>(
        &mut self,
        runtime: &mut Runtime,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        // Waking up needed thread
        for (p1, p2) in runtime.iter_mut() {
            if let Some(p1) = p1 {
                if p1.tid > MAX_THREAD_COUNT {
                    return Err(RuntimeError::InvalidTID(p1.tid));
                }
                if self.p1_context.wakingup_tid[p1.tid] {
                    let _ = p1.state.to(ThreadState::ACTIVE);
                    self.p1_context.wakingup_tid[p1.tid] = false;
                }
            }
            if let Some(p2) = p2 {
                if p2.tid > MAX_THREAD_COUNT {
                    return Err(RuntimeError::InvalidTID(p2.tid));
                }
                if self.p2_context.wakingup_tid[p2.tid] {
                    let _ = p2.state.to(ThreadState::ACTIVE);
                    self.p2_context.wakingup_tid[p2.tid] = false;
                }
            }
        }
        // spawn and close the needed thread
        for (tid, to_spawn) in self.p1_context.spawned_tid.iter().enumerate() {
            if *to_spawn {
                let _ = runtime.spawn_with_tid(super::vm::Player::P1, tid, engine)?;
            }
        }
        for (tid, to_spawn) in self.p2_context.spawned_tid.iter().enumerate() {
            if *to_spawn {
                let _ = runtime.spawn_with_tid(super::vm::Player::P2, tid, engine)?;
            }
        }
        for (tid, to_close) in self.p1_context.closed_tid.iter().enumerate() {
            if *to_close {
                let _ = runtime.close(super::vm::Player::P1, tid, engine)?;
            }
        }
        for (tid, to_close) in self.p2_context.closed_tid.iter().enumerate() {
            if *to_close {
                let _ = runtime.close(super::vm::Player::P2, tid, engine)?;
            }
        }
        Ok(())
    }

    fn wake_for_stdin<G: GameEngineStaticFn>(
        &self,
        player: crate::vm::vm::Player,
        runtime: &mut Runtime,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        let stdin_data = engine.stdin_scan();
        let Some(data) = stdin_data else {
            return Ok(());
        };
        stdio.stdin.write(data);

        let mut waiting_list = match player {
            Player::P1 => self
                .p1_waiting_list
                .as_ref()
                .lock()
                .map_err(|_| RuntimeError::ConcurrencyError)?,

            Player::P2 => self
                .p2_waiting_list
                .as_ref()
                .lock()
                .map_err(|_| RuntimeError::ConcurrencyError)?,
        };

        let mut idx_tid_list: Vec<(usize, usize)> = waiting_list
            .iter()
            .enumerate()
            .filter(|(_, status)| match status {
                WaitingStatus::Join { .. } => false,
                WaitingStatus::ForStdin(_) => true,
            })
            .map(|(idx, status)| {
                (
                    idx,
                    match status {
                        WaitingStatus::Join { join_tid, .. } => *join_tid, // unreachable,
                        WaitingStatus::ForStdin(id) => *id,
                    },
                )
            })
            .collect();
        idx_tid_list.sort_by(|left, right| left.0.cmp(&right.0));
        for (idx, wtid) in idx_tid_list.iter().rev() {
            waiting_list.remove(*idx);
            for (p1, p2) in runtime.iter_mut() {
                match player {
                    Player::P1 => {
                        if let Some(p1) = p1 {
                            if *wtid == p1.tid {
                                let _ = p1.state.to(ThreadState::ACTIVE);
                            }
                        }
                    }
                    Player::P2 => {
                        if let Some(p2) = p2 {
                            if *wtid == p2.tid {
                                let _ = p2.state.to(ThreadState::ACTIVE);
                            }
                        }
                    }
                }
            }
        }
        drop(waiting_list);
        Ok(())
    }

    fn wake_waiting_threads(
        &mut self,
        player: crate::vm::vm::Player,
        state: &mut ThreadState,
        tid: &usize,
    ) -> Result<ControlFlow<usize, ()>, RuntimeError> {
        match *state {
            ThreadState::IDLE | ThreadState::EXITED => {
                // Wake up all threads that are waiting on tid
                let mut waiting_list = match player {
                    Player::P1 => self
                        .p1_waiting_list
                        .as_ref()
                        .lock()
                        .map_err(|_| RuntimeError::ConcurrencyError)?,

                    Player::P2 => self
                        .p2_waiting_list
                        .as_ref()
                        .lock()
                        .map_err(|_| RuntimeError::ConcurrencyError)?,
                };
                let mut idx_tid_list: Vec<(usize, usize)> = waiting_list
                    .iter()
                    .enumerate()
                    .filter(|(_, status)| match status {
                        WaitingStatus::Join {
                            to_be_completed_tid,
                            ..
                        } => *to_be_completed_tid == *tid,
                        WaitingStatus::ForStdin(_) => false,
                    })
                    .map(|(idx, status)| {
                        (
                            idx,
                            match status {
                                WaitingStatus::Join { join_tid, .. } => *join_tid,
                                WaitingStatus::ForStdin(id) => *id, // unreachable,
                            },
                        )
                    })
                    .collect();
                idx_tid_list.sort_by(|left, right| left.0.cmp(&right.0));

                let context = match player {
                    Player::P1 => &mut self.p1_context,
                    Player::P2 => &mut self.p2_context,
                };
                for (idx, tid) in idx_tid_list.iter().rev() {
                    waiting_list.remove(*idx);
                    context.request_wake(*tid)?;
                }
                drop(waiting_list);
                //if idx_tid_list.len() > 0 {
                //    return Ok(ControlFlow::Break(0));
                //}
                return Ok(ControlFlow::Break(INSTRUCTION_MAX_COUNT));
            }
            ThreadState::WAITING => return Ok(ControlFlow::Break(0)),
            ThreadState::SLEEPING(_) => return Ok(ControlFlow::Break(0)),
            ThreadState::COMPLETED_MAF => return Ok(ControlFlow::Break(0)),
            ThreadState::ACTIVE => return Ok(ControlFlow::Continue(())),
            ThreadState::STARVED => return Ok(ControlFlow::Break(INSTRUCTION_MAX_COUNT)),
        }
    }
}
