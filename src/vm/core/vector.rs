use crate::{
    ast::expressions::Expression,
    semantic::{
        scope::static_types::{SliceType, StaticType, VecType},
        CompatibleWith, EType, Resolve, ResolveCore, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        allocator::{align, MemoryAddress},
        asm::{
            operation::{GetNumFrom, OpPrimitive, PopNum},
            Asm,
        },
        core::{lexem, CoreAsm},
        runtime::RuntimeError,
        scheduler::Executable,
        stdio::StdIO,
        GenerateCode,
    },
};

use super::PathFinder;

#[derive(Debug, Clone, PartialEq)]
pub enum VectorFn {
    Vec {
        with_capacity: bool,
        item_size: usize,
    },
    Push {
        item_size: usize,
    },
    Pop {
        item_size: usize,
    },
    Extend {
        item_size: usize,
        len: usize,
    },
    Delete {
        item_size: usize,
    },
    ClearVec {
        item_size: usize,
    },
}

impl PathFinder for VectorFn {
    fn find(path: &[String], name: &str) -> Option<Self>
    where
        Self: Sized,
    {
        if (path.len() == 1 && path[0] == lexem::VEC) || path.len() == 0 {
            return match name {
                lexem::VEC => Some(VectorFn::Vec {
                    with_capacity: false,
                    item_size: 0,
                }),
                lexem::PUSH => Some(VectorFn::Push { item_size: 0 }),
                lexem::POP => Some(VectorFn::Pop { item_size: 0 }),
                lexem::EXTEND => Some(VectorFn::Extend {
                    item_size: 0,
                    len: 0,
                }),
                lexem::DELETE => Some(VectorFn::Delete { item_size: 0 }),
                lexem::CLEAR_VEC => Some(VectorFn::ClearVec { item_size: 0 }),
                _ => None,
            };
        }
        None
    }
}

impl ResolveCore for VectorFn {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        fn vec_param<E: crate::vm::external::Engine>(
            param: &mut Expression,
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            scope_id: Option<u128>,
        ) -> Result<VecType, SemanticError> {
            let _ = param.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
            let EType::Static(StaticType::Vec(vector_type)) =
                param.type_of(scope_manager, scope_id)?
            else {
                return Err(SemanticError::IncorrectArguments);
            };
            Ok(vector_type)
        }
        match self {
            VectorFn::Vec {
                with_capacity,
                item_size,
            } => {
                let Some(context @ EType::Static(StaticType::Vec(VecType(item_type)))) = context
                else {
                    return Err(SemanticError::CantInferType(format!(
                        "of this vector allocation"
                    )));
                };
                if parameters.len() == 0 || parameters.len() > 2 {
                    return Err(SemanticError::IncorrectArguments);
                }

                for param in parameters.iter_mut() {
                    let _ = param.resolve::<E>(
                        scope_manager,
                        scope_id,
                        &Some(crate::p_num!(U64)),
                        &mut None,
                    )?;
                    let crate::p_num!(U64) = param.type_of(&scope_manager, scope_id)? else {
                        return Err(SemanticError::IncorrectArguments);
                    };
                }

                *with_capacity = parameters.len() == 2;
                *item_size = item_type.size_of();

                Ok(context.clone())
            }
            VectorFn::Push { item_size } => {
                if parameters.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, second_part) = parameters.split_at_mut(1);
                let vector = &mut first_part[0];
                let item = &mut second_part[0];
                let vector_type = vec_param::<E>(vector, scope_manager, scope_id)?;

                let _ = item.resolve::<E>(
                    scope_manager,
                    scope_id,
                    &Some(vector_type.0.as_ref().clone()),
                    &mut None,
                )?;
                let item_type = item.type_of(scope_manager, scope_id)?;
                let _ =
                    item_type.compatible_with(vector_type.0.as_ref(), scope_manager, scope_id)?;

                *item_size = item_type.size_of();

                Ok(EType::Static(StaticType::Vec(vector_type)))
            }
            VectorFn::Pop { item_size } => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let vector = &mut parameters[0];
                let vector_type = vec_param::<E>(vector, scope_manager, scope_id)?;

                *item_size = vector_type.0.size_of();

                Ok(vector_type.0.as_ref().clone())
            }
            VectorFn::Delete { item_size } => {
                if parameters.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, second_part) = parameters.split_at_mut(1);
                let vector = &mut first_part[0];
                let index = &mut second_part[0];
                let vector_type = vec_param::<E>(vector, scope_manager, scope_id)?;

                let _ = index.resolve::<E>(
                    scope_manager,
                    scope_id,
                    &Some(crate::p_num!(U64)),
                    &mut None,
                )?;
                let crate::p_num!(U64) = index.type_of(&scope_manager, scope_id)? else {
                    return Err(SemanticError::IncorrectArguments);
                };

                *item_size = vector_type.0.size_of();

                Ok(vector_type.0.as_ref().clone())
            }
            VectorFn::Extend { item_size, len } => {
                if parameters.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, second_part) = parameters.split_at_mut(1);
                let vector = &mut first_part[0];
                let array = &mut second_part[0];
                let vector_type = vec_param::<E>(vector, scope_manager, scope_id)?;

                let _ = array.resolve::<E>(
                    scope_manager,
                    scope_id,
                    &Some(vector_type.0.as_ref().clone()),
                    &mut None,
                )?;

                let EType::Static(StaticType::Slice(SliceType { size, item_type })) =
                    array.type_of(scope_manager, scope_id)?
                else {
                    return Err(SemanticError::IncompatibleTypes);
                };

                *len = size;

                *item_size = item_type.size_of();

                Ok(EType::Static(StaticType::Vec(vector_type)))
            }
            VectorFn::ClearVec { item_size } => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let vector = &mut parameters[0];
                let vector_type = vec_param::<E>(vector, scope_manager, scope_id)?;

                *item_size = vector_type.0.size_of();

                Ok(vector_type.0.as_ref().clone())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VectorAsm {
    Vec { item_size: usize },
    VecWithCapacity { item_size: usize },
    Push { item_size: usize },
    Pop { item_size: usize },
    Extend { item_size: usize, len: usize },
    Delete { item_size: usize },
    Clear { item_size: usize },
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for VectorAsm {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            VectorAsm::Vec { item_size } => stdio.push_asm_lib(engine, "vec"),
            VectorAsm::VecWithCapacity { item_size } => stdio.push_asm_lib(engine, "vec"),
            VectorAsm::Push { item_size } => stdio.push_asm_lib(engine, "push"),
            VectorAsm::Pop { item_size } => stdio.push_asm_lib(engine, "pop"),
            VectorAsm::Extend { item_size, len } => stdio.push_asm_lib(engine, "extend"),
            VectorAsm::Delete { item_size } => stdio.push_asm_lib(engine, "delete"),
            VectorAsm::Clear { item_size } => stdio.push_asm_lib(engine, "clear_vec"),
        }
    }
}

impl crate::vm::AsmWeight for VectorAsm {
    fn weight(&self) -> crate::vm::Weight {
        match self {
            VectorAsm::Vec { .. } => crate::vm::Weight::HIGH,
            VectorAsm::VecWithCapacity { .. } => crate::vm::Weight::HIGH,
            VectorAsm::Push { .. } => crate::vm::Weight::MEDIUM,
            VectorAsm::Pop { .. } => crate::vm::Weight::LOW,
            VectorAsm::Extend { .. } => crate::vm::Weight::HIGH,
            VectorAsm::Delete { .. } => crate::vm::Weight::LOW,
            VectorAsm::Clear { .. } => crate::vm::Weight::HIGH,
        }
    }
}

impl GenerateCode for VectorFn {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match *self {
            VectorFn::Vec {
                with_capacity,
                item_size,
            } => {
                if with_capacity {
                    instructions.push(Asm::Core(CoreAsm::Vec(VectorAsm::VecWithCapacity {
                        item_size,
                    })));
                } else {
                    instructions.push(Asm::Core(CoreAsm::Vec(VectorAsm::Vec { item_size })));
                }
            }
            VectorFn::Push { item_size } => {
                instructions.push(Asm::Core(CoreAsm::Vec(VectorAsm::Push { item_size })));
            }
            VectorFn::Pop { item_size } => {
                instructions.push(Asm::Core(CoreAsm::Vec(VectorAsm::Pop { item_size })));
            }
            VectorFn::Extend { item_size, len } => {
                instructions.push(Asm::Core(CoreAsm::Vec(VectorAsm::Extend {
                    item_size,
                    len,
                })));
            }
            VectorFn::Delete { item_size } => {
                instructions.push(Asm::Core(CoreAsm::Vec(VectorAsm::Delete { item_size })));
            }
            VectorFn::ClearVec { item_size } => {
                instructions.push(Asm::Core(CoreAsm::Vec(VectorAsm::Clear { item_size })))
            }
        }
        Ok(())
    }
}

/*
    VEC LAYOUT
    CAPACITY u64
    LEN u64
    ...DATA
*/

pub const VEC_HEADER: usize = 16;

impl<E: crate::vm::external::Engine> Executable<E> for VectorAsm {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::TID>,
    ) -> Result<(), RuntimeError> {
        match *self {
            VectorAsm::Vec { item_size } => {
                let len = OpPrimitive::pop_num::<u64>(stack)? as usize;

                let cap = len * 2;

                let address = heap.alloc(align(cap * item_size) + VEC_HEADER)?;

                /* Write capacity */
                let _ = heap.write(address, &(cap as u64).to_le_bytes())?;
                /* Write len */
                let _ = heap.write(address.add(8), &(len as u64).to_le_bytes())?;

                /* Push vec address */
                let address: u64 = address.into(stack);
                stack.push_with(&address.to_le_bytes())?;
            }
            VectorAsm::VecWithCapacity { item_size } => {
                let cap = OpPrimitive::pop_num::<u64>(stack)? as usize;
                let len = OpPrimitive::pop_num::<u64>(stack)? as usize;

                let address = heap.alloc(align(cap * item_size) + VEC_HEADER)?;

                /* Write capacity */
                let _ = heap.write(address, &(cap as u64).to_le_bytes())?;
                /* Write len */
                let _ = heap.write(address.add(8), &(len as u64).to_le_bytes())?;

                /* Push vec address */
                let address: u64 = address.into(stack);
                stack.push_with(&address.to_le_bytes())?;
            }
            VectorAsm::Push { item_size } => {
                let item_data = stack.pop(item_size)?.to_owned();
                let mut address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let mut cap = OpPrimitive::get_num_from::<u64>(address, stack, heap)? as usize;
                let mut len =
                    OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)? as usize;

                if len + 1 >= cap {
                    // Reallocate
                    let size = align(((len + 1) * 2) * item_size + VEC_HEADER);
                    len += 1;
                    cap = (len + 1) * 2;
                    address = heap.realloc(address, size)?;
                }

                /* Write capacity */
                let _ = heap.write(address, &(cap as u64).to_le_bytes())?;
                /* Write len */
                let _ = heap.write(address.add(8), &(len as u64).to_le_bytes())?;

                /* Write new item */
                let _ = heap.write(
                    address.add(VEC_HEADER + ((len - 1) * item_size)),
                    &item_data,
                )?;

                /* Push vec address */
                let address: u64 = address.into(stack);
                stack.push_with(&address.to_le_bytes())?;
            }
            VectorAsm::Pop { item_size } => {
                let address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let cap = OpPrimitive::get_num_from::<u64>(address, stack, heap)? as usize;
                let mut len =
                    OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)? as usize;

                if len > 1 {
                    len -= 1;
                } else {
                    return Err(RuntimeError::IndexOutOfBound);
                }
                /* read popped item */
                let item_data =
                    heap.read(address.add(VEC_HEADER + ((len + 1) * item_size)), item_size)?;

                /* Write capacity */
                let _ = heap.write(address, &(cap as u64).to_le_bytes())?;
                /* Write len */
                let _ = heap.write(address.add(8), &(len as u64).to_le_bytes())?;

                stack.push_with(&item_data)?;
            }
            VectorAsm::Extend {
                item_size,
                len: array_len,
            } => {
                let array_address: MemoryAddress =
                    OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let mut vector_address: MemoryAddress =
                    OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let array_data = match array_address {
                    MemoryAddress::Heap { offset } => {
                        heap.read_slice(array_address, item_size * array_len)?
                    }
                    MemoryAddress::Stack { offset } => {
                        stack.read(array_address, item_size * array_len)?
                    }
                    MemoryAddress::Frame { offset } => {
                        stack.read_in_frame(array_address, item_size * array_len)?
                    }
                    MemoryAddress::Global { offset } => {
                        stack.read_global(array_address, item_size * array_len)?
                    }
                }
                .to_vec();

                let mut cap =
                    OpPrimitive::get_num_from::<u64>(vector_address, stack, heap)? as usize;
                let mut len =
                    OpPrimitive::get_num_from::<u64>(vector_address.add(8), stack, heap)? as usize;

                if len + array_len >= cap {
                    // Reallocate
                    let size = align(((len + array_len) * 2) * item_size + VEC_HEADER);
                    len += array_len;
                    cap = (len + array_len) * 2;
                    vector_address = heap.realloc(vector_address, size)?;
                }

                /* Write capacity */
                let _ = heap.write(vector_address, &(cap as u64).to_le_bytes())?;
                /* Write len */
                let _ = heap.write(vector_address.add(8), &(len as u64).to_le_bytes())?;

                /* Write new items */
                let _ = heap.write(
                    vector_address.add(VEC_HEADER + ((len - array_len) * item_size)),
                    &array_data,
                )?;

                /* Push vec address */
                let address: u64 = vector_address.into(stack);
                stack.push_with(&address.to_le_bytes())?;
            }
            VectorAsm::Delete { item_size } => {
                let index = OpPrimitive::pop_num::<u64>(stack)? as usize;
                let mut address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let mut cap = OpPrimitive::get_num_from::<u64>(address, stack, heap)? as usize;
                let mut len =
                    OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)? as usize;

                if len + 1 >= cap {
                    // Reallocate
                    let size = align(((len + 1) * 2) * item_size + VEC_HEADER);
                    len += 1;
                    cap = (len + 1) * 2;
                    address = heap.realloc(address, size)?;
                }

                if index < len {
                    len -= 1;
                } else {
                    return Err(RuntimeError::IndexOutOfBound);
                }

                let item_data =
                    heap.read(address.add(VEC_HEADER + ((index) * item_size)), item_size)?;

                if (len - 1) - index > 0 {
                    let upper_data = heap.read(
                        address.add(VEC_HEADER + ((index + 1) * item_size)),
                        item_size * ((len - 1) - index),
                    )?;
                    let _ =
                        heap.write(address.add(VEC_HEADER + ((index) * item_size)), &upper_data)?;
                }

                /* Write capacity */
                let _ = heap.write(address, &(cap as u64).to_le_bytes())?;
                /* Write len */
                let _ = heap.write(address.add(8), &(len as u64).to_le_bytes())?;

                stack.push_with(&item_data)?;
            }
            VectorAsm::Clear { item_size } => {
                let address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let cap = OpPrimitive::get_num_from::<u64>(address, stack, heap)? as usize;
                let len = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)? as usize;

                /* Write capacity */
                let _ = heap.write(address, &(cap as u64).to_le_bytes())?;
                /* Write len */
                let _ = heap.write(address.add(8), &(0 as u64).to_le_bytes())?;
                /* Clear items */
                let _ = heap.write(address.add(VEC_HEADER), &vec![0; len * item_size])?;

                let address: u64 = address.into(stack);
                stack.push_with(&address.to_le_bytes())?;
            }
        }
        scheduler.next();
        Ok(())
    }
}
