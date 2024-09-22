use crate::{
    ast::expressions::Expression,
    semantic::{
        scope::static_types::{StaticType, VecType},
        CompatibleWith, EType, Metadata, Resolve, ResolvePlatform, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        allocator::{align, heap::Heap, stack::Stack},
        casm::{
            operation::{GetNumFrom, OpPrimitive, PopNum},
            Casm, CasmProgram,
        },
        core::{lexem, CoreCasm},
        stdio::StdIO,
        vm::{CasmMetadata, CodeGenerationError, Executable, GenerateCode, RuntimeError},
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
                // lexem::EXTEND => Some(VectorFn::Extend { item_size: 0 }),
                lexem::DELETE => Some(VectorFn::Delete { item_size: 0 }),
                lexem::CLEAR_VEC => Some(VectorFn::ClearVec { item_size: 0 }),
                _ => None,
            };
        }
        None
    }
}

impl ResolvePlatform for VectorFn {
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        fn vec_param<G: crate::GameEngineStaticFn>(
            param: &mut Expression,
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            scope_id: Option<u128>,
        ) -> Result<VecType, SemanticError> {
            let _ = param.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
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
                    let _ = param.resolve::<G>(
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
                let item = &mut second_part[1];
                let vector_type = vec_param::<G>(vector, scope_manager, scope_id)?;

                let _ = item.resolve::<G>(
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
                let vector_type = vec_param::<G>(vector, scope_manager, scope_id)?;

                *item_size = vector_type.0.size_of();

                Ok(vector_type.0.as_ref().clone())
            }
            VectorFn::Delete { item_size } => {
                if parameters.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, second_part) = parameters.split_at_mut(1);
                let vector = &mut first_part[0];
                let index = &mut second_part[1];
                let vector_type = vec_param::<G>(vector, scope_manager, scope_id)?;

                let _ = index.resolve::<G>(
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
            VectorFn::Extend { item_size } => todo!(),
            VectorFn::ClearVec { item_size } => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let vector = &mut parameters[0];
                let vector_type = vec_param::<G>(vector, scope_manager, scope_id)?;

                *item_size = vector_type.0.size_of();

                Ok(vector_type.0.as_ref().clone())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VectorCasm {
    Vec { item_size: usize },
    VecWithCapacity { item_size: usize },
    Push { item_size: usize },
    Pop { item_size: usize },
    Extend { item_size: usize },
    Delete { item_size: usize },
    Clear { item_size: usize },
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for VectorCasm {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            VectorCasm::Vec { item_size } => stdio.push_casm_lib(engine, "vec"),
            VectorCasm::VecWithCapacity { item_size } => stdio.push_casm_lib(engine, "vec"),
            VectorCasm::Push { item_size } => stdio.push_casm_lib(engine, "push"),
            VectorCasm::Pop { item_size } => stdio.push_casm_lib(engine, "pop"),
            VectorCasm::Extend { item_size } => stdio.push_casm_lib(engine, "extend"),
            VectorCasm::Delete { item_size } => stdio.push_casm_lib(engine, "delete"),
            VectorCasm::Clear { item_size } => stdio.push_casm_lib(engine, "clear_vec"),
        }
    }

    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            VectorCasm::Vec { item_size } => crate::vm::vm::CasmWeight::HIGH,
            VectorCasm::VecWithCapacity { item_size } => crate::vm::vm::CasmWeight::HIGH,
            VectorCasm::Push { item_size } => crate::vm::vm::CasmWeight::MEDIUM,
            VectorCasm::Pop { item_size } => crate::vm::vm::CasmWeight::LOW,
            VectorCasm::Extend { item_size } => crate::vm::vm::CasmWeight::HIGH,
            VectorCasm::Delete { item_size } => crate::vm::vm::CasmWeight::LOW,
            VectorCasm::Clear { item_size } => crate::vm::vm::CasmWeight::HIGH,
        }
    }
}

impl GenerateCode for VectorFn {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match *self {
            VectorFn::Vec {
                with_capacity,
                item_size,
            } => {
                if with_capacity {
                    instructions.push(Casm::Core(CoreCasm::Vec(VectorCasm::VecWithCapacity {
                        item_size,
                    })));
                } else {
                    instructions.push(Casm::Core(CoreCasm::Vec(VectorCasm::Vec { item_size })));
                }
            }
            VectorFn::Push { item_size } => {
                instructions.push(Casm::Core(CoreCasm::Vec(VectorCasm::Push { item_size })));
            }
            VectorFn::Pop { item_size } => {
                instructions.push(Casm::Core(CoreCasm::Vec(VectorCasm::Pop { item_size })));
            }
            VectorFn::Extend { item_size } => {
                instructions.push(Casm::Core(CoreCasm::Vec(VectorCasm::Extend { item_size })));
            }
            VectorFn::Delete { item_size } => {
                instructions.push(Casm::Core(CoreCasm::Vec(VectorCasm::Delete { item_size })));
            }
            VectorFn::ClearVec { item_size } => {
                instructions.push(Casm::Core(CoreCasm::Vec(VectorCasm::Clear { item_size })))
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

impl<G: crate::GameEngineStaticFn> Executable<G> for VectorCasm {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match *self {
            VectorCasm::Vec { item_size } => todo!(),
            VectorCasm::VecWithCapacity { item_size } => todo!(),
            VectorCasm::Push { item_size } => {
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
            VectorCasm::Pop { item_size } => {
                let mut address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let mut cap = OpPrimitive::get_num_from::<u64>(address, stack, heap)? as usize;
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
            VectorCasm::Extend { item_size } => todo!(),
            VectorCasm::Delete { item_size } => {
                let mut index = OpPrimitive::pop_num::<u64>(stack)? as usize;
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
            VectorCasm::Clear { item_size } => {
                let mut address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let mut cap = OpPrimitive::get_num_from::<u64>(address, stack, heap)? as usize;
                let mut len =
                    OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)? as usize;

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
        program.incr();
        Ok(())
    }
}
