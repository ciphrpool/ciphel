use crate::vm::{
    allocator::{Memory, MemoryAddress},
    vm::{Executable, RuntimeError},
};

use super::operation::OpPrimitive;

#[derive(Debug, Clone)]
pub enum Access {
    Static { address: MemoryAddress, size: usize },
    Runtime { size: usize },
}

impl Executable for Access {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match self {
            Access::Static { address, size } => match address {
                MemoryAddress::Heap => {
                    let address = OpPrimitive::get_num8::<u64>(memory)? as usize;
                    let _data = memory.heap.read(address, *size).map_err(|err| err.into())?;
                    todo!("Copy data onto stack");
                    Ok(())
                }
                MemoryAddress::Stack { offset } => {
                    let _data = memory
                        .stack
                        .read(*offset, *size)
                        .map_err(|err| err.into())?;

                    todo!("Copy data onto stack");
                    Ok(())
                }
            },
            Access::Runtime { size } => {
                let address = OpPrimitive::get_num8::<u64>(memory)?;
                let address = {
                    todo!("Convert u64 to memory address by differenting stack pointer and heap pointer")
                };
                // match address {
                //     MemoryAddress::Heap => {
                //         let _data = memory
                //             .heap
                //             .read(address + (*size) * (index as usize), *size)
                //             .map_err(|err| err.into())?;
                //         todo!("Copy data onto stack");
                //         Ok(())
                //     }
                //     MemoryAddress::Stack { offset } => {
                //         let _data = memory
                //             .stack
                //             .read(*offset, (*size) * (index as usize))
                //             .map_err(|err| err.into())?;

                //         todo!("Copy data onto stack");
                //         Ok(())
                //     }
                // }
            }
        }
    }
}
