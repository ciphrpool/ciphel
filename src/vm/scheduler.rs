use std::cell::Cell;

use super::{
    allocator::{heap::Heap, stack::Stack, Memory},
    casm::CasmProgram,
    stdio::StdIO,
    vm::{CasmMetadata, Executable, Runtime, RuntimeError},
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
        println!("MAJOR FRAME START");
        let minor_frame_max_count = self.minor_frame_max_count.get();
        for (_, ref mut stack, ref mut program, tid) in runtime.iter_mut() {
            let mut thread_instruction_count = 0;
            println!("MINOR FRAME ::tid {tid}\n");
            while thread_instruction_count < minor_frame_max_count {
                let return_status = program.evaluate(|instruction| {
                    instruction.name(stdio, program);
                    match instruction.execute(program, stack, heap, stdio) {
                        Ok(_) => Ok(()),
                        err @ Err(RuntimeError::Exit) => err,
                        Err(err) => {
                            println!("RUNTIME ERROR :: {:?} in {:?}", err, instruction);
                            Err(err)
                        }
                    }
                });
                thread_instruction_count += 1;
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
    }
}
