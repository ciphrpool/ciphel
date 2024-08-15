use num_traits::ToBytes;
use ulid::Ulid;

use super::{operation::OpPrimitive, CasmProgram};

use crate::{
    semantic::AccessLevel,
    vm::{
        allocator::{
            heap::Heap,
            stack::{Offset, Stack, UReg, STACK_SIZE},
            MemoryAddress,
        },
        stdio::StdIO,
        vm::{CasmMetadata, Executable, RuntimeError},
    },
};

#[derive(Debug, Clone)]
pub enum Mem {
    MemCopy,
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
    TakeUTF8Char,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Mem {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            Mem::MemCopy => stdio.push_casm(engine, "memcpy"),
            Mem::Dup(n) => stdio.push_casm(engine, &format!("dup {n}")),
            Mem::DumpRegisters => stdio.push_casm(engine, "dmp_regs"),
            Mem::RecoverRegisters => stdio.push_casm(engine, "set_regs"),
            Mem::SetReg(reg, Some(n)) => {
                stdio.push_casm(engine, &format!("set_reg {} {n}", reg.name()))
            }
            Mem::SetReg(reg, None) => stdio.push_casm(engine, &format!("set_reg {}", reg.name())),
            Mem::GetReg(reg) => stdio.push_casm(engine, &format!("dmp_reg {}", reg.name())),
            Mem::AddReg(reg, Some(n)) => {
                stdio.push_casm(engine, &format!("reg_add {} {n}", reg.name()))
            }
            Mem::AddReg(reg, None) => stdio.push_casm(engine, &format!("reg_add {}", reg.name())),
            Mem::SubReg(reg, Some(n)) => {
                stdio.push_casm(engine, &format!("reg_sub {} {n}", reg.name()))
            }
            Mem::SubReg(reg, None) => stdio.push_casm(engine, &format!("reg_sub {}", reg.name())),
            Mem::CloneFromSmartPointer(n) => stdio.push_casm(engine, &format!("clone {n}")),
            Mem::LabelOffset(label) => {
                let label = program
                    .get_label_name(label)
                    .unwrap_or("".to_string().into())
                    .to_string();
                stdio.push_casm(engine, &format!("dmp_label {label}"))
            }
            Mem::TakeToHeap { size } => stdio.push_casm(engine, &format!("htake {size}")),
            Mem::TakeToStack { size } => stdio.push_casm(engine, &format!("stake {size}")),
            Mem::Take { size } => stdio.push_casm(engine, &format!("take {size}")),
            Mem::TakeUTF8Char => stdio.push_casm(engine, "take_utf8_char"),
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Mem {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self {
            Mem::CloneFromSmartPointer(size) => {
                let heap_address = OpPrimitive::get_num8::<u64>(stack)?;
                let data = heap.read(heap_address as usize, 8)?;
                let data = TryInto::<&[u8; 8]>::try_into(data.as_slice())
                    .map_err(|_| RuntimeError::Deserialization)?;
                let length = u64::from_le_bytes(*data);

                let data = heap.read(heap_address as usize + 16, size * length as usize)?;

                let _ = stack.push_with(&data)?;
                let _ = stack.push_with(&length.to_le_bytes())?;
            }
            Mem::TakeToHeap { size } => {
                let heap_address = OpPrimitive::get_num8::<u64>(stack)?;

                let data = stack.pop(*size)?;
                let _ = heap.write(heap_address as usize, &data.to_vec())?;
                let _ = stack.push_with(&heap_address.to_le_bytes())?;
            }
            Mem::Take { size } => {
                let address = OpPrimitive::get_num8::<u64>(stack)?;
                if address < STACK_SIZE as u64 {
                    let data = stack.pop(*size)?.to_owned();
                    let _ =
                        stack.write(Offset::SB(address as usize), AccessLevel::General, &data)?;
                } else {
                    let data = stack.pop(*size)?;
                    let _ = heap.write(address as usize, &data.to_vec())?;
                    let _ = stack.push_with(&address.to_le_bytes())?;
                }
            }
            Mem::TakeUTF8Char => {
                let address = OpPrimitive::get_num8::<u64>(stack)?;
                let value = OpPrimitive::get_char(stack)?;
                if address < STACK_SIZE as u64 {
                    let _ = stack.write(
                        Offset::SB(address as usize),
                        AccessLevel::General,
                        &value.to_string().as_bytes(),
                    )?;
                } else {
                    let _ = heap.write(address as usize, &value.to_string().as_bytes().to_vec())?;
                    let _ = stack.push_with(&address.to_le_bytes())?;
                }
            }
            Mem::TakeToStack { size } => {
                let stack_address = OpPrimitive::get_num8::<u64>(stack)?;
                let data = stack.pop(*size)?.to_owned();
                let _ = stack.write(
                    Offset::SB(stack_address as usize),
                    AccessLevel::General,
                    &data,
                )?;
            }
            Mem::Dup(size) => {
                let data = stack
                    .read(Offset::ST(-(*size as isize)), AccessLevel::Direct, *size)?
                    .to_owned();
                let _ = stack.push_with(&data)?;
            }
            Mem::SetReg(reg, opt_offset) => match opt_offset {
                Some(offset) => {
                    let _old = stack.set_reg(*reg, *offset);
                }
                None => {
                    let idx = OpPrimitive::get_num8::<u64>(stack)?;
                    let _old = stack.set_reg(*reg, idx);
                }
            },
            Mem::GetReg(reg) => {
                let idx = stack.get_reg(*reg);
                let _ = stack.push_with(&idx.to_le_bytes())?;
            }
            Mem::AddReg(reg, opt_offset) => match opt_offset {
                Some(offset) => stack.reg_add(*reg, *offset)?,
                None => {
                    let offset = OpPrimitive::get_num8::<u64>(stack)?;
                    stack.reg_add(*reg, offset)?;
                }
            },
            Mem::SubReg(reg, opt_offset) => match opt_offset {
                Some(offset) => stack.reg_sub(*reg, *offset)?,
                None => {
                    let offset = OpPrimitive::get_num8::<u64>(stack)?;
                    stack.reg_sub(*reg, offset)?;
                }
            },
            Mem::LabelOffset(label) => {
                let Some(idx) = program.get(label) else {
                    return Err(RuntimeError::CodeSegmentation);
                };
                let idx = idx as u64;
                let _ = stack.push_with(&idx.to_le_bytes())?;
            }
            Mem::DumpRegisters => {
                for reg in [UReg::R1, UReg::R2, UReg::R3, UReg::R4] {
                    let idx = stack.get_reg(reg);
                    let _ = stack.push_with(&idx.to_le_bytes())?;
                }
            }
            Mem::RecoverRegisters => {
                for reg in [UReg::R4, UReg::R3, UReg::R2, UReg::R1] {
                    let idx = OpPrimitive::get_num8::<u64>(stack)?;
                    let _ = stack.set_reg(reg, idx);
                }
            }
            Mem::MemCopy => {
                let size = OpPrimitive::get_num8::<u64>(stack)? as usize;
                let from_address = OpPrimitive::get_num8::<u64>(stack)?;
                let to_address = OpPrimitive::get_num8::<u64>(stack)?;

                let from_address = {
                    if from_address < STACK_SIZE as u64 {
                        MemoryAddress::Stack {
                            offset: Offset::SB(from_address as usize),
                            level: AccessLevel::General,
                        }
                    } else {
                        MemoryAddress::Heap {
                            offset: from_address as usize,
                        }
                    }
                };

                let from_data = match from_address {
                    MemoryAddress::Heap { offset } => {
                        // let address = OpPrimitive::get_num8::<u64>(stack)? as usize;
                        let bytes = heap.read(offset, size)?;
                        bytes
                    }
                    MemoryAddress::Stack { offset, level } => {
                        if let Offset::FE(stack_idx, heap_udx) = offset {
                            let heap_address = stack.read(Offset::FP(stack_idx), level, 8)?;
                            let data = TryInto::<&[u8; 8]>::try_into(heap_address)
                                .map_err(|_| RuntimeError::Deserialization)?;
                            let heap_address = u64::from_le_bytes(*data);

                            let bytes = heap.read(heap_address as usize + heap_udx, size)?;
                            bytes
                        } else {
                            let bytes = stack.read(offset, level, size)?;
                            bytes.to_vec()
                        }
                    }
                };
                if to_address < STACK_SIZE as u64 {
                    let _ = stack.write(
                        Offset::SB(to_address as usize),
                        AccessLevel::General,
                        &from_data,
                    )?;
                } else {
                    let _ = heap.write(to_address as usize, &from_data)?;
                }
            }
        };

        program.incr();
        Ok(())
    }
}
