use std::cell::Cell;

use num_traits::ToBytes;

use crate::vm::{
    platform::{
        core::thread::{sig_close, sig_spawn},
        utils::lexem::SPAWN,
    },
    vm::{Signal, MAX_THREAD_COUNT},
};

use super::{
    allocator::{heap::Heap, stack::Stack, Memory},
    casm::CasmProgram,
    stdio::StdIO,
    vm::{CasmMetadata, Executable, Runtime, RuntimeError, Tid},
};

const INSTRUCTION_MAX_COUNT: usize = 100;

#[derive(Debug, Clone)]
pub struct Scheduler {
    minor_frame_max_count: Cell<usize>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            minor_frame_max_count: Cell::new(INSTRUCTION_MAX_COUNT),
        }
    }

    pub fn prepare(&self, number_of_thread: usize) {
        if number_of_thread == 0 {
            return;
        }
        self.minor_frame_max_count
            .set(INSTRUCTION_MAX_COUNT / number_of_thread);
    }

    pub fn run_major_frame(&self, runtime: &mut Runtime, heap: &mut Heap, stdio: &mut StdIO) {
        let mut availaible_tids: Vec<usize> =
            runtime.threads.iter().map(|(_, _, _, tid)| *tid).collect();
        let mut spawned_tid = Vec::new();
        let mut closed_tid = Vec::new();

        let mut requested_spawn_count = runtime.tid_count.get();

        println!("MAJOR FRAME START");
        let minor_frame_max_count = self.minor_frame_max_count.get();
        for (_, ref mut stack, ref mut program, tid) in runtime.iter_mut() {
            let mut thread_instruction_count = 0;
            println!("MINOR FRAME ::tid {tid}\n");
            while thread_instruction_count < minor_frame_max_count {
                let return_status = program.evaluate(|instruction| {
                    instruction.name(stdio, program);
                    thread_instruction_count += instruction.weight();
                    match instruction.execute(program, stack, heap, stdio) {
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
                            Signal::WAIT(_) => todo!(),
                            Signal::WAKE(_) => todo!(),
                            Signal::SLEEP(_) => todo!(),
                            Signal::JOIN(_) => todo!(),
                        },
                        err @ Err(RuntimeError::Exit) => err,
                        Err(err) => {
                            println!("RUNTIME ERROR :: {:?} in {:?}", err, instruction);
                            Err(err)
                        }
                    }
                });
                match return_status {
                    Ok(_) => {}
                    Err(RuntimeError::Exit) => break,
                    Err(_) => {
                        /* TODO : error handling */
                        break;
                    }
                }
            }
        }
        println!("MAJOR FRAME END");

        // spawn and close the needed thread
        for tid in spawned_tid {
            let _ = runtime.spawn_with_tid(tid);
        }
        for tid in closed_tid {
            let _ = runtime.close(tid);
        }
    }
}
