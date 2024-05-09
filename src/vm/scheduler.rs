use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    marker::PhantomData,
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
    stdio::StdIO,
    vm::{CasmMetadata, Executable, GameEngineStaticFn, Runtime, RuntimeError, Tid},
};

const INSTRUCTION_MAX_COUNT: usize = 20;

#[derive(Debug, Clone)]
pub struct Scheduler<G: GameEngineStaticFn + Clone> {
    join_waiting_list: RefCell<Vec<(Tid, Tid)>>,
    minor_frame_max_count: Cell<usize>,
    _phantom: PhantomData<G>,
}

impl<G: crate::GameEngineStaticFn + Clone> Scheduler<G> {
    pub fn new() -> Self {
        Self {
            join_waiting_list: RefCell::default(),
            minor_frame_max_count: Cell::new(INSTRUCTION_MAX_COUNT),
            _phantom: PhantomData::default(),
        }
    }

    pub fn prepare(&self, number_of_thread: usize) {
        if number_of_thread == 0 {
            return;
        }
        self.minor_frame_max_count
            .set(INSTRUCTION_MAX_COUNT / number_of_thread);
    }

    pub fn run_major_frame(
        &self,
        runtime: &mut Runtime<G>,
        heap: &mut Heap,
        stdio: &mut StdIO<G>,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        let info = runtime.tid_info();
        let mut availaible_tids: Vec<usize> =
            runtime.threads.iter().map(|thread| thread.tid).collect();
        let mut spawned_tid = Vec::new();
        let mut wakingup_tid = HashSet::new();
        let mut closed_tid = Vec::new();

        let mut requested_spawn_count = runtime.tid_count.get();

        stdio.push_casm_info(engine, "MAJOR FRAME START : {info}".into());
        let minor_frame_max_count = self.minor_frame_max_count.get();

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
                        let idx_tid_list: Vec<(usize, usize)> = self
                            .join_waiting_list
                            .borrow()
                            .iter()
                            .enumerate()
                            .filter(|(_, (_, otid))| *otid == *tid)
                            .map(|(idx, (wtid, _))| (idx, *wtid))
                            .collect();

                        for (idx, tid) in &idx_tid_list {
                            self.join_waiting_list.borrow_mut().remove(*idx);
                            wakingup_tid.insert(*tid);
                        }
                        if idx_tid_list.len() > 0 {
                            break;
                        }
                    }
                    continue;
                }

                stdio.push_casm_info(engine, "MINOR FRAME ::tid {tid}\n".into());
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
                                Signal::JOIN(join_tid) => sig_join(
                                    *tid,
                                    join_tid,
                                    state,
                                    &mut self.join_waiting_list.borrow_mut(),
                                    &mut availaible_tids,
                                    program,
                                    stack,
                                ),
                                Signal::EXIT => sig_exit(
                                    *tid,
                                    state,
                                    &mut closed_tid,
                                    &mut self.join_waiting_list.borrow_mut(),
                                    &mut wakingup_tid,
                                ),
                            },
                            Err(err) => {
                                stdio.push_casm_info(
                                    engine,
                                    &format!("RUNTIME ERROR :: {:?} in {:?}", err, instruction),
                                );
                                Err(err)
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
                            Signal::EXIT | Signal::WAIT | Signal::SLEEP(_) | Signal::JOIN(_) => {
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
