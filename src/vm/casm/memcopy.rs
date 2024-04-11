use num_traits::ToBytes;
use ulid::Ulid;

use super::operation::OpPrimitive;
use super::CasmProgram;
use crate::{
    semantic::AccessLevel,
    vm::{
        allocator::{
            stack::{Offset, UReg, STACK_SIZE},
            Memory, MemoryAddress,
        },
        scheduler::Thread,
        vm::{Executable, RuntimeError},
    },
};
use std::{cell::Cell, os::raw::c_uint};

#[derive(Debug, Clone)]
pub enum MemCopy {
    Dup(usize),
    DumpRegisters,
    RecoverRegisters,
    SetReg(UReg, Option<u64>),
    GetReg(UReg),
    AddReg(UReg, Option<u64>),
    SubReg(UReg, Option<u64>),
    CloneFromSmartPointer(usize),
    LabelOffset(Ulid),
    TakeToHeap { size: usize },
    TakeToStack { size: usize },
    Take { size: usize },
}

impl Executable for MemCopy {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match self {
            MemCopy::CloneFromSmartPointer(size) => {
                let heap_address = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                let data = thread
                    .runtime
                    .heap
                    .read(heap_address as usize, 8)
                    .map_err(|e| e.into())?;
                let data = TryInto::<&[u8; 8]>::try_into(data.as_slice())
                    .map_err(|_| RuntimeError::Deserialization)?;
                let length = u64::from_le_bytes(*data);

                let data = thread
                    .runtime
                    .heap
                    .read(heap_address as usize + 16, size * length as usize)
                    .map_err(|e| e.into())?;

                let _ = thread.env.stack.push_with(&data).map_err(|e| e.into())?;
                let _ = thread
                    .env
                    .stack
                    .push_with(&length.to_le_bytes())
                    .map_err(|e| e.into())?;
            }
            MemCopy::TakeToHeap { size } => {
                let heap_address = OpPrimitive::get_num8::<u64>(&thread.memory())?;

                let data = thread.env.stack.pop(*size).map_err(|e| e.into())?;
                let _ = thread
                    .runtime
                    .heap
                    .write(heap_address as usize, &data)
                    .map_err(|e| e.into())?;
                let _ = thread
                    .env
                    .stack
                    .push_with(&heap_address.to_le_bytes())
                    .map_err(|e| e.into())?;
            }
            MemCopy::Take { size } => {
                let address = OpPrimitive::get_num8::<u64>(&thread.memory())?;

                if address < STACK_SIZE as u64 {
                    let data = thread.env.stack.pop(*size).map_err(|e| e.into())?;
                    let _ = thread
                        .env
                        .stack
                        .write(Offset::SB(address as usize), AccessLevel::General, &data)
                        .map_err(|e| e.into())?;
                } else {
                    let data = thread.env.stack.pop(*size).map_err(|e| e.into())?;

                    let _ = thread
                        .runtime
                        .heap
                        .write(address as usize, &data)
                        .map_err(|e| e.into())?;
                    let _ = thread
                        .env
                        .stack
                        .push_with(&address.to_le_bytes())
                        .map_err(|e| e.into())?;
                }
            }
            MemCopy::TakeToStack { size } => {
                let stack_address = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                let data = thread.env.stack.pop(*size).map_err(|e| e.into())?;
                let _ = thread
                    .env
                    .stack
                    .write(
                        Offset::SB(stack_address as usize),
                        AccessLevel::General,
                        &data,
                    )
                    .map_err(|e| e.into())?;
            }
            MemCopy::Dup(size) => {
                let data = thread
                    .env
                    .stack
                    .read(Offset::ST(-(*size as isize)), AccessLevel::Direct, *size)
                    .map_err(|e| e.into())?;
                let _ = thread.env.stack.push_with(&data).map_err(|e| e.into())?;
            }
            MemCopy::SetReg(reg, opt_offset) => match opt_offset {
                Some(offset) => {
                    let old = thread.env.stack.set_reg(*reg, *offset);
                }
                None => {
                    let idx = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                    let old = thread.env.stack.set_reg(*reg, idx);
                }
            },
            MemCopy::GetReg(reg) => {
                let idx = thread.env.stack.get_reg(*reg);
                let _ = thread
                    .env
                    .stack
                    .push_with(&idx.to_le_bytes())
                    .map_err(|e| e.into())?;
            }
            MemCopy::AddReg(reg, opt_offset) => match opt_offset {
                Some(offset) => thread
                    .env
                    .stack
                    .reg_add(*reg, *offset)
                    .map_err(|e| e.into())?,
                None => {
                    let offset = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .reg_add(*reg, offset)
                        .map_err(|e| e.into())?;
                }
            },
            MemCopy::SubReg(reg, opt_offset) => match opt_offset {
                Some(offset) => thread
                    .env
                    .stack
                    .reg_sub(*reg, *offset)
                    .map_err(|e| e.into())?,
                None => {
                    let offset = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .reg_sub(*reg, offset)
                        .map_err(|e| e.into())?;
                }
            },
            MemCopy::LabelOffset(label) => {
                let Some(idx) = thread.env.program.get(label) else {
                    return Err(RuntimeError::CodeSegmentation);
                };
                let idx = idx as u64;
                let _ = thread
                    .env
                    .stack
                    .push_with(&idx.to_le_bytes())
                    .map_err(|e| e.into())?;
            }
            MemCopy::DumpRegisters => {
                for reg in [UReg::R1, UReg::R2, UReg::R3, UReg::R4] {
                    let idx = thread.env.stack.get_reg(reg);
                    let _ = thread
                        .env
                        .stack
                        .push_with(&idx.to_le_bytes())
                        .map_err(|e| e.into())?;
                }
            }
            MemCopy::RecoverRegisters => {
                for reg in [UReg::R4, UReg::R3, UReg::R2, UReg::R1] {
                    let idx = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                    let _ = thread.env.stack.set_reg(reg, idx);
                }
            }
        }

        thread.env.program.incr();
        Ok(())
    }
}
