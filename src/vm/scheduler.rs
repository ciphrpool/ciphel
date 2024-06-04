use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    marker::PhantomData,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use num_traits::ToBytes;

use crate::vm::{
    casm::Casm,
    platform::{
        core::thread::{sig_close, sig_exit, sig_join, sig_sleep, sig_spawn, sig_wait, sig_wake},
        utils::lexem::SPAWN,
    },
    vm::{Signal, Thread, ThreadState, MAX_THREAD_COUNT},
};

use super::{
    allocator::{heap::Heap, stack::Stack, Memory},
    casm::CasmProgram,
    platform::core::thread::sig_wait_stdin,
    stdio::StdIO,
    vm::{CasmMetadata, Executable, GameEngineStaticFn, Runtime, RuntimeError, Tid},
};

const INSTRUCTION_MAX_COUNT: usize = 20;

#[derive(Debug, Clone)]
pub enum WaitingStatus {
    Join {
        join_tid: Tid,
        to_be_completed_tid: Tid,
    },
    ForStdin(Tid),
}

#[derive(Debug, Clone)]
pub struct Scheduler {
    waiting_list: Arc<Mutex<Vec<WaitingStatus>>>,
    minor_frame_max_count: Arc<AtomicUsize>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            waiting_list: Default::default(),
            minor_frame_max_count: Arc::new(AtomicUsize::new(INSTRUCTION_MAX_COUNT)),
        }
    }

    pub fn prepare(&self, number_of_thread: usize) {
        if number_of_thread == 0 {
            return;
        }
        self.minor_frame_max_count
            .as_ref()
            .store(INSTRUCTION_MAX_COUNT / number_of_thread, Ordering::Release);
    }

    pub fn run_major_frame<G: crate::GameEngineStaticFn>(
        &self,
        runtime: &mut Runtime,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        let mut availaible_tids: Vec<usize> =
            runtime.threads.iter().map(|thread| thread.tid).collect();
        let mut spawned_tid = Vec::new();
        let mut wakingup_tid = HashSet::new();
        let mut closed_tid = Vec::new();

        let mut requested_spawn_count = runtime.tid_count.as_ref().load(Ordering::Acquire);

        let minor_frame_max_count = self.minor_frame_max_count.as_ref().load(Ordering::Acquire);

        // Wake thread waiting on stdin
        let stdin_data = engine.stdin_scan();
        if let Some(data) = stdin_data {
            stdio.stdin.write(data);

            let mut waiting_list = self
                .waiting_list
                .as_ref()
                .lock()
                .map_err(|_| RuntimeError::ConcurrencyError)?;

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
                for Thread { ref tid, state, .. } in runtime.iter_mut() {
                    if *wtid == *tid {
                        let _ = state.to(ThreadState::ACTIVE);
                    }
                }
            }
            drop(waiting_list);
        }

        let info = runtime.tid_info();
        stdio.push_casm_info(engine, &format!("MAJOR FRAME START : {info}"));

        let mut total_instruction_count = 0;
        while total_instruction_count < INSTRUCTION_MAX_COUNT {
            for Thread {
                ref mut stack,
                ref mut program,
                ref tid,
                state,
                ..
            } in runtime.iter_mut()
            {
                let mut thread_instruction_count = 0;

                state.init_maf(program.cursor_is_at_end());

                if state.is_noop() {
                    if ThreadState::IDLE == *state || ThreadState::COMPLETED == *state {
                        // Wake up all threads that are waiting on tid
                        let mut waiting_list = self
                            .waiting_list
                            .as_ref()
                            .lock()
                            .map_err(|_| RuntimeError::ConcurrencyError)?;
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
                        for (idx, _) in idx_tid_list.iter().rev() {
                            waiting_list.remove(*idx);
                            wakingup_tid.insert(*tid);
                        }
                        drop(waiting_list);
                        if idx_tid_list.len() > 0 {
                            break;
                        }
                    }
                    continue;
                }

                stdio.push_casm_info(engine, &format!("MINOR FRAME ::tid {tid}\n"));
                while thread_instruction_count < minor_frame_max_count {
                    if program.cursor_is_at_end() {
                        let _ = state.to(ThreadState::IDLE);
                        break;
                    }
                    let return_status = program.evaluate(|instruction| {
                        instruction.name(stdio, program, engine);
                        thread_instruction_count += <Casm as CasmMetadata<G>>::weight(instruction);
                        total_instruction_count += <Casm as CasmMetadata<G>>::weight(instruction);
                        match instruction.execute(program, stack, heap, stdio, engine) {
                            Ok(_) => Ok(()),
                            Err(RuntimeError::Signal(signal)) => match signal {
                                Signal::SPAWN => sig_spawn(
                                    &mut requested_spawn_count,
                                    &mut availaible_tids,
                                    &mut spawned_tid,
                                    program,
                                    stack,
                                ),
                                Signal::CLOSE(tid) => sig_close(
                                    &tid,
                                    &mut availaible_tids,
                                    &mut closed_tid,
                                    program,
                                    stack,
                                ),
                                Signal::WAIT => sig_wait(state, program),
                                Signal::WAKE(wake_tid) => sig_wake(
                                    &wake_tid,
                                    &mut availaible_tids,
                                    &mut wakingup_tid,
                                    program,
                                    stack,
                                ),
                                Signal::SLEEP(nb_maf) => sig_sleep(&nb_maf, state, program),
                                Signal::JOIN(join_tid) => {
                                    let mut waiting_list = self
                                        .waiting_list
                                        .as_ref()
                                        .lock()
                                        .map_err(|_| RuntimeError::ConcurrencyError)?;

                                    let _ = sig_join(
                                        *tid,
                                        join_tid,
                                        state,
                                        &mut waiting_list,
                                        &mut availaible_tids,
                                        program,
                                        stack,
                                    )?;
                                    drop(waiting_list);
                                    Ok(())
                                }
                                Signal::EXIT => {
                                    let mut waiting_list = self
                                        .waiting_list
                                        .as_ref()
                                        .lock()
                                        .map_err(|_| RuntimeError::ConcurrencyError)?;
                                    let _ = sig_exit(
                                        *tid,
                                        state,
                                        &mut closed_tid,
                                        &mut waiting_list,
                                        &mut wakingup_tid,
                                    )?;
                                    drop(waiting_list);
                                    Ok(())
                                }
                                Signal::WAIT_STDIN => {
                                    let mut waiting_list = self
                                        .waiting_list
                                        .as_ref()
                                        .lock()
                                        .map_err(|_| RuntimeError::ConcurrencyError)?;
                                    let _ = sig_wait_stdin(*tid, state, &mut waiting_list)?;
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
                            if program.cursor_is_at_end() {
                                let _ = state.to(ThreadState::IDLE);
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
                            if program.cursor_is_at_end() {
                                let _ = state.to(ThreadState::IDLE);
                            }
                            /* TODO : error handling */
                            panic!("{:?}", e);
                            return Err(e);
                        }
                    }
                }
            }

            // Waking up needed thread
            for Thread { ref tid, state, .. } in runtime.iter_mut() {
                if wakingup_tid.contains(&tid) {
                    let _ = state.to(ThreadState::ACTIVE);
                    wakingup_tid.remove(tid);
                }
            }
            if runtime.all_noop() {
                break;
            }
        }
        stdio.push_casm_info(engine, "MAJOR FRAME END".into());
        // Waking up needed thread
        for Thread { ref tid, state, .. } in runtime.iter_mut() {
            if wakingup_tid.contains(&tid) {
                let _ = state.to(ThreadState::ACTIVE);
                wakingup_tid.remove(tid);
            }
        }
        // spawn and close the needed thread
        for tid in spawned_tid {
            let _ = runtime.spawn_with_tid(tid);
        }
        for tid in closed_tid {
            let _ = runtime.close(tid);
        }
        Ok(())
    }
}
