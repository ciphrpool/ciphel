use num_traits::ToBytes;
use ulid::Ulid;

use crate::{
    semantic::AccessLevel,
    vm::{
        allocator::{
            heap::Heap,
            stack::{Offset, Stack, StackSlice, UReg, STACK_SIZE},
            MemoryAddress,
        },
        casm::operation::OpPrimitive,
        stdio::StdIO,
        vm::{CasmMetadata, Executable, RuntimeError},
    },
};

use super::CasmProgram;

#[derive(Debug, Clone)]
pub enum Alloc {
    Heap { size: Option<usize> },
    Stack { size: usize },
}

impl CasmMetadata for Alloc {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram) {
        match self {
            Alloc::Heap { size } => match size {
                None => stdio.push_casm("halloc"),
                Some(n) => stdio.push_casm(&format!("halloc {n}")),
            },
            Alloc::Stack { size } => stdio.push_casm(&format!("salloc {size}")),
        }
    }
}
impl Executable for Alloc {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
    ) -> Result<(), RuntimeError> {
        match self {
            Alloc::Heap { size } => {
                let size = match size {
                    Some(size) => *size,
                    None => {
                        let size = OpPrimitive::get_num8::<u64>(stack)?;
                        size as usize
                    }
                };
                let address = heap.alloc(size).map_err(|e| e.into())?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

                let data = (address as u64).to_le_bytes().to_vec();
                let _ = stack.push_with(&data).map_err(|e| e.into())?;
                program.incr();
                Ok(())
            }
            Alloc::Stack { size } => {
                let _ = stack.push(*size).map_err(|e| e.into())?;
                program.incr();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Realloc {
    pub size: Option<usize>,
}
impl CasmMetadata for Realloc {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram) {
        match self.size {
            Some(n) => stdio.push_casm(&format!("realloc {n}")),
            None => stdio.push_casm("realloc"),
        }
    }
}
impl Executable for Realloc {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
    ) -> Result<(), RuntimeError> {
        let size = match self.size {
            Some(size) => size,
            None => {
                let size = OpPrimitive::get_num8::<u64>(stack)?;
                size as usize
            }
        };

        let heap_address = OpPrimitive::get_num8::<u64>(stack)? - 8;
        let address = heap
            .realloc(heap_address as usize, size)
            .map_err(|e| e.into())?;
        let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

        let data = (address as u64).to_le_bytes().to_vec();
        let _ = stack.push_with(&data).map_err(|e| e.into())?;
        program.incr();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Free();
impl CasmMetadata for Free {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram) {
        stdio.push_casm("free");
    }
}
impl Executable for Free {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
    ) -> Result<(), RuntimeError> {
        let address = OpPrimitive::get_num8::<u64>(stack)? - 8;
        if address < STACK_SIZE as u64 {
            todo!("Decide wether runtime error or noop");
        } else {
            heap.free(address as usize).map_err(|e| e.into())?;
        }
        program.incr();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum StackFrame {
    Clean,
    SoftClean,
    Break,
    Continue,
    OpenWindow,
    CloseWindow,
    Return { return_size: Option<usize> },
    Transfer { is_direct_loop: bool },
}
impl CasmMetadata for StackFrame {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram) {
        stdio.push_casm("free");
        match self {
            StackFrame::Clean => stdio.push_casm("clean"),
            StackFrame::SoftClean => stdio.push_casm("soft_clean"),
            StackFrame::Break => stdio.push_casm("break"),
            StackFrame::Continue => stdio.push_casm("continue"),
            StackFrame::OpenWindow => stdio.push_casm("op_win"),
            StackFrame::CloseWindow => stdio.push_casm("cl_win"),
            StackFrame::Return { return_size } => match return_size {
                Some(n) => stdio.push_casm(&format!("return {n}")),
                None => stdio.push_casm("return"),
            },
            StackFrame::Transfer { is_direct_loop } => match is_direct_loop {
                true => stdio.push_casm("transfer_loop"),
                false => stdio.push_casm("transfer"),
            },
        }
    }
}
pub const FLAG_VOID: u8 = 0u8;
pub const FLAG_RETURN: u8 = 1u8;
pub const FLAG_BREAK: u8 = 2u8;
pub const FLAG_CONTINUE: u8 = 3u8;

impl Executable for StackFrame {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
    ) -> Result<(), RuntimeError> {
        match self {
            StackFrame::Break => {
                // let idx = OpPrimitive::get_num8::<u64>(stack)? as usize;
                // program.cursor_set(idx);

                program.cursor.set(stack.registers.link.get());

                let _ = stack.clean().map_err(|e| e.into())?;

                let _ = stack.push_with(&0u64.to_le_bytes()).map_err(|e| e.into())?;
                let _ = stack.push_with(&[FLAG_BREAK]).map_err(|e| e.into())?; /* return flag set to BREAK */

                Ok(())
            }
            StackFrame::Continue => {
                // let idx = OpPrimitive::get_num8::<u64>(stack)? as usize;
                // program.cursor_set(idx);

                program.cursor.set(stack.registers.link.get());

                let _ = stack.clean().map_err(|e| e.into())?;

                let _ = stack.push_with(&0u64.to_le_bytes()).map_err(|e| e.into())?;
                let _ = stack.push_with(&[FLAG_CONTINUE]).map_err(|e| e.into())?; /* return flag set to CONTINUE */

                Ok(())
            }
            StackFrame::SoftClean => {
                program.incr();
                let _ = stack.clean().map_err(|e| e.into())?;

                let _ = stack.push_with(&0u64.to_le_bytes()).map_err(|e| e.into())?;
                let _ = stack.push_with(&[FLAG_VOID]).map_err(|e| e.into())?; /* return flag set to false */
                Ok(())
            }
            StackFrame::Clean => {
                program.cursor.set(stack.registers.link.get());
                let _ = stack.clean().map_err(|e| e.into())?;

                let _ = stack.push_with(&0u64.to_le_bytes()).map_err(|e| e.into())?;
                let _ = stack.push_with(&[FLAG_VOID]).map_err(|e| e.into())?; /* return flag set to false */
                Ok(())
            }
            StackFrame::OpenWindow => {
                let _ = stack.open_window().map_err(|e| e.into())?;
                program.incr();
                Ok(())
            }
            StackFrame::CloseWindow => {
                let _ = stack.open_window().map_err(|e| e.into())?;
                program.incr();
                Ok(())
            }
            StackFrame::Return { return_size } => {
                let link = stack.registers.link.get();
                let return_size = match return_size {
                    Some(size) => *size,
                    None => {
                        let size = OpPrimitive::get_num8::<u64>(stack)?;
                        size as usize
                    }
                };
                let return_data = stack.pop(return_size).map_err(|e| e.into())?.to_owned();

                program.cursor.set(link);
                let _ = stack.clean().map_err(|e| e.into())?;

                let _ = stack.push_with(&return_data).map_err(|e| e.into())?;
                let _ = stack
                    .push_with(&(return_size as u64).to_le_bytes())
                    .map_err(|e| e.into())?;
                let _ = stack.push_with(&[FLAG_RETURN]).map_err(|e| e.into())?; /* return flag set to true */

                Ok(())
            }
            StackFrame::Transfer { is_direct_loop } => {
                let flag = OpPrimitive::get_num1::<u8>(stack)?;
                let return_size = OpPrimitive::get_num8::<u64>(stack)? as usize;
                match flag {
                    FLAG_BREAK => {
                        if !*is_direct_loop {
                            program.cursor.set(stack.registers.link.get());

                            let _ = stack.clean().map_err(|e| e.into())?;

                            let _ = stack.push_with(&0u64.to_le_bytes()).map_err(|e| e.into())?;
                            let _ = stack.push_with(&[FLAG_BREAK]).map_err(|e| e.into())?;
                        /* return flag set to BREAK */
                        } else {
                            let break_link = stack.get_reg(UReg::R4);
                            program.cursor.set(break_link as usize);
                        }
                    }
                    FLAG_CONTINUE => {
                        if !*is_direct_loop {
                            program.cursor.set(stack.registers.link.get());

                            let _ = stack.clean().map_err(|e| e.into())?;

                            let _ = stack.push_with(&0u64.to_le_bytes()).map_err(|e| e.into())?;
                            let _ = stack.push_with(&[FLAG_CONTINUE]).map_err(|e| e.into())?;
                        /* return flag set to BREAK */
                        } else {
                            let break_link = stack.get_reg(UReg::R3);
                            program.cursor.set(break_link as usize);
                        }
                    }
                    FLAG_RETURN => {
                        let return_data = stack.pop(return_size).map_err(|e| e.into())?.to_owned();

                        program.cursor.set(stack.registers.link.get());
                        let _ = stack.clean().map_err(|e| e.into())?;

                        let _ = stack.push_with(&return_data).map_err(|e| e.into())?;
                        let _ = stack
                            .push_with(&(return_size as u64).to_le_bytes())
                            .map_err(|e| e.into())?;
                        let _ = stack.push_with(&[FLAG_RETURN]).map_err(|e| e.into())?;
                        /* return flag set to true */
                    }
                    FLAG_VOID => {
                        program.incr();
                    }
                    _ => return Err(RuntimeError::ReturnFlagError),
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Access {
    Static { address: MemoryAddress, size: usize },
    Runtime { size: Option<usize> },
    RuntimeCharUTF8,
    RuntimeCharUTF8AtIdx,
}

impl CasmMetadata for Access {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram) {
        match self {
            Access::Static { address, size } => {
                stdio.push_casm(&format!("ld {} {size}", address.name()))
            }
            Access::Runtime { size } => match size {
                Some(n) => stdio.push_casm(&format!("ld {n}")),
                None => stdio.push_casm("ld"),
            },
            Access::RuntimeCharUTF8 => stdio.push_casm("ld_utf8"),
            Access::RuntimeCharUTF8AtIdx => stdio.push_casm("ld_utf8_at"),
        }
    }
}

impl Executable for Access {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
    ) -> Result<(), RuntimeError> {
        let index = match self {
            Access::RuntimeCharUTF8AtIdx => Some(OpPrimitive::get_num8::<u64>(stack)?),
            _ => None,
        };
        let (address, size) = match self {
            Access::Static { address, size } => (*address, *size),
            Access::RuntimeCharUTF8 | Access::RuntimeCharUTF8AtIdx => {
                let address = OpPrimitive::get_num8::<u64>(stack)?;
                let address = {
                    if address < STACK_SIZE as u64 {
                        MemoryAddress::Stack {
                            offset: Offset::SB(address as usize),
                            level: AccessLevel::General,
                        }
                    } else {
                        MemoryAddress::Heap {
                            offset: address as usize,
                        }
                    }
                };
                (address, 1)
            }
            Access::Runtime { size } => {
                let size = match size {
                    Some(size) => *size,
                    None => {
                        let size = OpPrimitive::get_num8::<u64>(stack)?;
                        size as usize
                    }
                };
                let address = OpPrimitive::get_num8::<u64>(stack)?;
                let address = {
                    if address < STACK_SIZE as u64 {
                        MemoryAddress::Stack {
                            offset: Offset::SB(address as usize),
                            level: AccessLevel::General,
                        }
                    } else {
                        MemoryAddress::Heap {
                            offset: address as usize,
                        }
                    }
                };
                (address, size)
            }
        };
        match address {
            MemoryAddress::Heap { offset } => {
                // let address = OpPrimitive::get_num8::<u64>(stack)? as usize;

                let data = match self {
                    Access::RuntimeCharUTF8 | Access::RuntimeCharUTF8AtIdx => {
                        let (bytes, _) = heap
                            .read_utf8(offset, index.unwrap_or(0) as usize)
                            .map_err(|err| err.into())?;

                        bytes.to_vec()
                    }
                    _ => {
                        let bytes = heap.read(offset, size).map_err(|err| err.into())?;
                        bytes
                    }
                };

                let _ = stack.push_with(&data).map_err(|e| e.into())?;
                program.incr();
                Ok(())
            }
            MemoryAddress::Stack { offset, level } => {
                if let Offset::FE(stack_idx, heap_udx) = offset {
                    let heap_address = stack
                        .read(Offset::FP(stack_idx), level, 8)
                        .map_err(|err| err.into())?;
                    let data = TryInto::<&[u8; 8]>::try_into(heap_address)
                        .map_err(|_| RuntimeError::Deserialization)?;
                    let heap_address = u64::from_le_bytes(*data);

                    let data = match self {
                        Access::RuntimeCharUTF8 | Access::RuntimeCharUTF8AtIdx => {
                            let (bytes, _) = heap
                                .read_utf8(
                                    heap_address as usize + heap_udx,
                                    index.unwrap_or(0) as usize,
                                )
                                .map_err(|err| err.into())?;

                            bytes.to_vec()
                        }
                        _ => {
                            let bytes = heap
                                .read(heap_address as usize + heap_udx, size)
                                .map_err(|err| err.into())?;
                            bytes
                        }
                    };

                    let _ = stack.push_with(&data).map_err(|e| e.into())?;
                    program.incr();
                    return Ok(());
                }

                let data = match self {
                    Access::RuntimeCharUTF8 | Access::RuntimeCharUTF8AtIdx => {
                        let (bytes, _) = stack
                            .read_utf8(offset, level, index.unwrap_or(0) as usize)
                            .map_err(|err| err.into())?;

                        bytes.to_vec()
                    }
                    _ => {
                        let bytes = stack.read(offset, level, size).map_err(|err| err.into())?;
                        bytes.to_vec()
                    }
                };
                // dbg!(&data);
                // Copy data onto stack;
                let _ = stack.push_with(&data).map_err(|e| e.into())?;
                program.incr();
                Ok(())
            }
        }
    }
}

// #[derive(Debug, Clone)]
// pub struct Assign {
//     address: MemoryAddress,
//     stack_slice: StackSlice,
// }

// impl CasmMetadata for Assign {
//     fn name(&self, stdio: &mut StdIO, program: &CasmProgram) {
//         stdio.push_casm(&format!("ld {} {size}", address.name()))
//     }

//     fn weight(&self) -> usize {
//         todo!()
//     }
// }
// impl Executable for Assign {
//     fn execute(
//         &self,
//         program: &CasmProgram,
//         stack: &mut Stack,
//         heap: &mut Heap,
//         stdio: &mut StdIO,
//     ) -> Result<(), RuntimeError> {
//         let data = stack
//             .read(
//                 self.stack_slice.offset,
//                 AccessLevel::Direct,
//                 self.stack_slice.size,
//             )
//             .map_err(|err| err.into())?
//             .to_owned();

//         match self.address {
//             MemoryAddress::Heap { offset } => {
//                 // let address = OpPrimitive::get_num8::<u64>(stack)? as usize;
//                 let _ = heap.write(offset, &data.to_vec()).map_err(|e| e.into())?;
//             }
//             MemoryAddress::Stack { offset, level } => {
//                 let _ = stack.write(offset, level, &data).map_err(|e| e.into())?;
//             }
//         };

//         program.incr();
//         Ok(())
//     }
// }
