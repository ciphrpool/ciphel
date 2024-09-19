use num_traits::ToBytes;

use crate::ast::expressions::locate::Locatable;
use crate::semantic;
use crate::semantic::scope::scope::{self, GlobalMapping, ScopeManager};
use crate::semantic::scope::static_types::{MapType, SliceType, StaticType, TupleType};
use crate::semantic::AccessLevel;
use crate::vm::platform::core::alloc::AllocCasm;
use crate::vm::platform::core::core_map::MapCasm;
use crate::vm::platform::core::CoreCasm;
use crate::vm::platform::LibCasm;
use crate::vm::vm::CodeGenerationContext;
use crate::{
    ast::utils::strings::ID,
    semantic::{scope::user_type_impl::UserType, EType, SizeOf},
    vm::{
        allocator::{align, MemoryAddress},
        casm::{
            alloc::{Access, Alloc},
            branch::{Goto, Label},
            data,
            locate::Locate,
            mem::Mem,
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{
    Address, Closure, Data, Enum, Map, Number, Primitive, PtrAccess, Slice, StrSlice, Struct,
    Tuple, Union, Variable, Vector,
};

impl GenerateCode for Data {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Data::Primitive(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Data::Slice(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Data::Vec(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Data::Closure(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Data::Tuple(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Data::Address(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Data::PtrAccess(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Data::Variable(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Data::Unit => Ok(()),
            Data::Map(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Data::Struct(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Data::Union(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Data::Enum(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Data::StrSlice(value) => value.gencode(scope_manager, scope_id, instructions, context),
        }
    }
}

// impl Locatable for Data {
//     fn locate(
//         &self,
//         scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//         instructions: &mut CasmProgram,
//         context: &crate::vm::vm::CodeGenerationContext,
//     ) -> Result<(), CodeGenerationError> {
//         match self {
//             Data::Variable(Variable {
//                 name,
//                 state,
//                 metadata,
//             }) => {
//                 if *from_field {
//                     Ok(())
//                 } else {
//                     let (var, address, level) = scope_manager.access_var(id)?;

//                     let var_type = &var.type_sig;
//                     let _var_size = var_type.size_of();

//                     instructions.push(Casm::Locate(Locate {
//                         address: MemoryAddress::Stack {
//                             offset: address,
//                             level,
//                         },
//                     }));
//                     Ok(())
//                 }
//             }
//             Data::PtrAccess(PtrAccess { value, .. }) => {
//                 let _ = value.gencode(scope_manager, scope_id, instructions, context)?;
//                 Ok(())
//             }
//             _ => {
//                 let _ = self.gencode(scope_manager, scope_id, instructions, context)?;
//                 let Some(value_type) = self.signature() else {
//                     return Err(CodeGenerationError::UnresolvedError);
//                 };
//                 instructions.push(Casm::Locate(Locate {
//                     address: MemoryAddress::Stack {
//                         offset: Offset::ST(-(value_type.size_of() as isize)),
//                         level: AccessLevel::Direct,
//                     },
//                 }));
//                 Ok(())
//             }
//         }
//     }

//     fn is_assignable(&self) -> bool {
//         match self {
//             Data::Variable(_) => true,
//             Data::PtrAccess(_) => true,
//             Data::Primitive(_) => false,
//             Data::Slice(_) => false,
//             Data::StrSlice(_) => false,
//             Data::Vec(_) => false,
//             Data::Closure(_) => false,
//             Data::Tuple(_) => false,
//             Data::Address(_) => false,
//             Data::Unit => false,
//             Data::Map(_) => false,
//             Data::Struct(_) => false,
//             Data::Union(_) => false,
//             Data::Enum(_) => false,
//         }
//     }

//     fn most_left_id(&self) -> Option<ID> {
//         match self {
//             Data::Variable(Variable { name, metadata, .. }) => Some(name.clone()),
//             _ => None,
//         }
//     }
// }

impl GenerateCode for Number {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let casm = match self {
            super::Number::U8(data) => data::Data::Serialized {
                data: data.to_le_bytes().into(),
            },
            super::Number::U16(data) => data::Data::Serialized {
                data: data.to_le_bytes().into(),
            },
            super::Number::U32(data) => data::Data::Serialized {
                data: data.to_le_bytes().into(),
            },
            super::Number::U64(data) => data::Data::Serialized {
                data: data.to_le_bytes().into(),
            },
            super::Number::U128(data) => data::Data::Serialized {
                data: data.to_le_bytes().into(),
            },
            super::Number::I8(data) => data::Data::Serialized {
                data: data.to_le_bytes().into(),
            },
            super::Number::I16(data) => data::Data::Serialized {
                data: data.to_le_bytes().into(),
            },
            super::Number::I32(data) => data::Data::Serialized {
                data: data.to_le_bytes().into(),
            },
            super::Number::I64(data) => data::Data::Serialized {
                data: data.to_le_bytes().into(),
            },
            super::Number::I128(data) => data::Data::Serialized {
                data: data.to_le_bytes().into(),
            },
            super::Number::F64(data) => data::Data::Serialized {
                data: data.to_le_bytes().into(),
            },
            super::Number::Unresolved(_) => return Err(CodeGenerationError::UnresolvedError),
        };
        instructions.push(Casm::Data(casm));
        Ok(())
    }
}

impl GenerateCode for Primitive {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let casm = match self {
            Primitive::Number(data) => match data {
                super::Number::U8(data) => data::Data::Serialized {
                    data: data.to_le_bytes().into(),
                },
                super::Number::U16(data) => data::Data::Serialized {
                    data: data.to_le_bytes().into(),
                },
                super::Number::U32(data) => data::Data::Serialized {
                    data: data.to_le_bytes().into(),
                },
                super::Number::U64(data) => data::Data::Serialized {
                    data: data.to_le_bytes().into(),
                },
                super::Number::U128(data) => data::Data::Serialized {
                    data: data.to_le_bytes().into(),
                },
                super::Number::I8(data) => data::Data::Serialized {
                    data: data.to_le_bytes().into(),
                },
                super::Number::I16(data) => data::Data::Serialized {
                    data: data.to_le_bytes().into(),
                },
                super::Number::I32(data) => data::Data::Serialized {
                    data: data.to_le_bytes().into(),
                },
                super::Number::I64(data) => data::Data::Serialized {
                    data: data.to_le_bytes().into(),
                },
                super::Number::I128(data) => data::Data::Serialized {
                    data: data.to_le_bytes().into(),
                },
                super::Number::F64(data) => data::Data::Serialized {
                    data: data.to_le_bytes().into(),
                },
                super::Number::Unresolved(_) => return Err(CodeGenerationError::UnresolvedError),
            },
            Primitive::Bool(data) => data::Data::Serialized {
                data: [*data as u8].into(),
            },
            Primitive::Char(data) => {
                let mut buffer = [0u8; 4];
                let _ = data.encode_utf8(&mut buffer);
                data::Data::Serialized {
                    data: buffer.into(),
                }
            }
        };
        instructions.push(Casm::Data(casm));
        Ok(())
    }
}

impl GenerateCode for Variable {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(super::VariableState::Variable { id }) = &self.state else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Ok(crate::semantic::scope::scope::Variable {
            ctype,
            address,
            state,
            ..
        }) = scope_manager.find_var_by_id(*id, scope_id)
        else {
            return Err(CodeGenerationError::Unlocatable);
        };
        let Ok(address) = (address.clone()).try_into() else {
            return Err(CodeGenerationError::Unlocatable);
        };

        instructions.push(Casm::Access(Access::Static {
            address,
            size: ctype.size_of(),
        }));

        Ok(())
    }
}

impl GenerateCode for Slice {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        for element in &self.value {
            let _ = element.gencode(scope_manager, scope_id, instructions, context)?;
        }
        let address = scope_manager.global_mapping.alloc(self.size);
        let address = address
            .try_into()
            .map_err(|_| CodeGenerationError::Default)?;

        instructions.push(Casm::Alloc(Alloc::GlobalFromStack {
            address,
            size: self.size,
        }));
        Ok(())
    }
}

impl GenerateCode for StrSlice {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let mut buffer = (self.value.len()).to_le_bytes().to_vec();
        buffer.extend_from_slice(self.value.as_bytes());

        let address = if self.value != "" {
            let address = scope_manager.global_mapping.alloc(self.value.len());

            address
                .try_into()
                .map_err(|_| CodeGenerationError::Default)?
        } else {
            GlobalMapping::EMPTY_STRING_ADDRESS
        };

        instructions.push(Casm::Alloc(Alloc::Global {
            address,
            data: buffer.into(),
        }));
        Ok(())
    }
}

impl GenerateCode for Vector {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(EType::Static(StaticType::Vec(item_type))) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let item_size = item_type.size_of();

        let len_bytes = (self.length as u64).to_le_bytes().as_slice().into();
        let cap_bytes = (self.capacity as u64).to_le_bytes().as_slice().into();

        // Push Length on stack
        instructions.push(Casm::Data(data::Data::Serialized { data: len_bytes }));
        // Push Capacity on stack
        instructions.push(Casm::Data(data::Data::Serialized { data: cap_bytes }));

        for element in &self.value {
            let _ = element.gencode(scope_manager, scope_id, instructions, context)?;
        }

        // Copy data on stack to heap at address

        // Alloc and push heap address on stack
        instructions.push(Casm::Alloc(Alloc::Heap {
            size: Some(item_size * self.capacity + 16),
        }));
        // Take the address on the top of the stack
        // and copy the data on the stack in the heap at given address and given offset
        // ( removing the data from the stack )
        instructions.push(Casm::Mem(Mem::Take {
            //offset: vec_stack_address + 8,
            size: item_size * self.value.len() + 16,
        }));

        Ok(())
    }
}

impl GenerateCode for Tuple {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(EType::Static(StaticType::Tuple(TupleType(tuple_types)))) =
            self.metadata.signature()
        else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        for (expr, expr_type) in self.value.iter().zip(tuple_types) {
            let _ = expr.gencode(scope_manager, scope_id, instructions, context)?;
        }
        Ok(())
    }
}

impl GenerateCode for Closure {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        todo!()
        // let end_closure = Label::gen();
        // let return_label = Label::gen();

        // instructions.push(Casm::Goto(Goto {
        //     label: Some(end_closure),
        // }));

        // let closure_label = instructions.push_label("fn_closure".into());
        // let _ = self.scope.gencode(
        //     scope_manager,
        //     scope_id,
        //     instructions,
        //     &CodeGenerationContext {
        //         return_label: Some(return_label),
        //         ..Default::default()
        //     },
        // );
        // let Some(return_size) = self.scope.signature().map(|t| t.size_of()) else {
        //     return Err(CodeGenerationError::UnresolvedError);
        // };
        // // Function epilogue
        // instructions.push_label_id(return_label, "function_epilogue".to_string().into());
        // instructions.push(Casm::StackFrame(StackFrame::ReturnV2 {
        //     return_size,
        //     label: None,
        // }));

        // instructions.push_label_id(end_closure, "end_closure".into());

        // instructions.push(Casm::Mem(Mem::Label(closure_label)));

        // if self.closed {
        //     /* Load env and store in the heap */
        //     let mut alloc_size = 16;
        //     let mut env_size = 0;

        //     let binding = self
        //         .scope
        //         .scope()
        //         .map_err(|_| CodeGenerationError::UnresolvedError)?;
        //     let inner_scope = binding.as_ref();

        //     for var in inner_scope
        //         .try_read()
        //         .map_err(|_| CodeGenerationError::ConcurrencyError)?
        //         .env_vars()
        //         .map_err(|_| CodeGenerationError::ConcurrencyError)?
        //     {
        //         let var_type = &var.type_sig;
        //         let var_size = var_type.size_of();
        //         alloc_size += var_size;
        //         env_size += var_size;
        //     }

        //     // Load Env Size
        //     instructions.push(Casm::Data(data::Data::Serialized {
        //         data: env_size.to_le_bytes().into(),
        //     }));
        //     let outer_scope = scope_manager;
        //     // Load Env variables
        //     for var in inner_scope
        //         .try_read()
        //         .map_err(|_| CodeGenerationError::ConcurrencyError)?
        //         .env_vars()
        //         .map_err(|_| CodeGenerationError::ConcurrencyError)?
        //     {
        //         let (var, address, level) = outer_scope.access_var(&var.id)?;
        //         let var_type = &var.type_sig;
        //         let var_size = var_type.size_of();
        //         instructions.push(Casm::Access(Access::Static {
        //             address: MemoryAddress::Stack {
        //                 offset: address,
        //                 level: level,
        //             },
        //             size: var_size,
        //         }));
        //     }
        //     instructions.push(Casm::Alloc(Alloc::Heap {
        //         size: Some(alloc_size),
        //     }));
        //     instructions.push(Casm::Mem(Mem::TakeToHeap { size: alloc_size }));
        // }

        // Ok(())
    }
}

impl GenerateCode for Address {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let _ = self.value.locate(scope_manager, scope_id, instructions)?;
        Ok(())
    }
}

impl GenerateCode for PtrAccess {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let _ = self
            .value
            .gencode(scope_manager, scope_id, instructions, context)?;
        let mut size = self
            .metadata
            .signature()
            .ok_or(CodeGenerationError::UnresolvedError)?
            .size_of();
        if size == 0 {
            size = self
                .metadata
                .context()
                .ok_or(CodeGenerationError::UnresolvedError)?
                .size_of();
        }
        instructions.push(Casm::Access(Access::Runtime { size: Some(size) }));

        Ok(())
    }
}

impl GenerateCode for Struct {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(EType::User { id, size }) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(UserType::Struct(crate::semantic::scope::user_type_impl::Struct { id, fields })) =
            scope_manager.find_type_by_id(id, scope_id).ok().cloned()
        else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        for (field_id, field_type) in fields {
            let field_size = field_type.size_of();

            let Some(field_expr) = self
                .fields
                .iter()
                .find(|(id, _)| *id == field_id)
                .map(|(_, e)| e)
            else {
                return Err(CodeGenerationError::UnresolvedError);
            };

            let _ = field_expr.gencode(scope_manager, scope_id, instructions, context)?;
            // Add padding
            let padding = align(field_size) - field_size;
            if padding > 0 {
                instructions.push(Casm::Data(data::Data::Serialized {
                    data: vec![0; padding].into(),
                }));
            }
        }
        Ok(())
    }
}

impl GenerateCode for Union {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(EType::User { id, size }) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(UserType::Union(
            ref union_type @ crate::semantic::scope::user_type_impl::Union { ref variants, .. },
        )) = scope_manager.find_type_by_id(id, scope_id).ok().cloned()
        else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let union_size = union_type.size_of();

        let Some((idx, struct_type)) = union_type
            .variants
            .iter()
            .enumerate()
            .find(|(_idx, (id, _))| id == &self.variant)
            .map(|(idx, (_, st))| (idx, st))
        else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let mut total_size = 8;
        for (field_id, field) in &struct_type.fields {
            let field_size = field.size_of();

            let Some(field_expr) = self
                .fields
                .iter()
                .find(|(id, _)| id == field_id)
                .map(|(_, e)| e)
            else {
                return Err(CodeGenerationError::UnresolvedError);
            };

            let _ = field_expr.gencode(scope_manager, scope_id, instructions, context)?;
            // Add padding
            let padding = align(field_size) - field_size;
            if padding > 0 {
                instructions.push(Casm::Data(data::Data::Serialized {
                    data: vec![0; padding].into(),
                }));
            }
            total_size += align(field_size);
        }

        if total_size < union_size {
            // Add padding
            let padding = union_size - total_size;
            if padding > 0 {
                instructions.push(Casm::Data(data::Data::Serialized {
                    data: vec![0; padding].into(),
                }));
            }
        }

        instructions.push(Casm::Data(data::Data::Serialized {
            data: (idx as u64).to_le_bytes().into(),
        }));

        Ok(())
    }
}

impl GenerateCode for Enum {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(EType::User { id, size }) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(UserType::Enum(crate::semantic::scope::user_type_impl::Enum { values, .. })) =
            scope_manager.find_type_by_id(id, scope_id).ok().cloned()
        else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let Some(index) = values
            .iter()
            .enumerate()
            .find(|(_, id)| id == &&self.value)
            .map(|(idx, _)| idx)
        else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        instructions.push(Casm::Data(data::Data::Serialized {
            data: (index as u64).to_le_bytes().into(),
        }));
        Ok(())
    }
}

impl GenerateCode for Map {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let cap = align(self.fields.len());
        let Some(EType::Static(StaticType::Map(MapType {
            keys_type,
            values_type,
        }))) = self.metadata.signature()
        else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let key_size = keys_type.size_of();
        let item_size = values_type.size_of();
        instructions.push(Casm::Data(data::Data::Serialized {
            data: cap.to_le_bytes().into(),
        }));
        instructions.push(Casm::Platform(LibCasm::Core(CoreCasm::Map(
            MapCasm::MapWithCapacity {
                key_size,
                item_size,
            },
        ))));

        for (key, value) in &self.fields {
            let _ = key.gencode(scope_manager, scope_id, instructions, context)?;
            let _ = value.gencode(scope_manager, scope_id, instructions, context)?;

            instructions.push(Casm::Platform(LibCasm::Core(CoreCasm::Map(
                MapCasm::Insert {
                    key_size,
                    ref_access: keys_type.as_ref().into(),
                    item_size,
                },
            ))));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use num_traits::Zero;

    use super::*;
    use crate::{
        assert_number,
        ast::{
            expressions::{
                data::{Data, Number},
                Atomic, Expression,
            },
            statements::Statement,
            TryParse,
        },
        e_static, p_num,
        semantic::{
            scope::{
                static_types::{PrimitiveType, SliceType, StrSliceType, TupleType, VecType},
                user_type_impl,
            },
            Resolve,
        },
        test_extract_variable, test_extract_variable_with, test_statements, v_num,
        vm::{
            casm::operation::{GetNumFrom, OpPrimitive},
            platform::core::{core_string::STRING_HEADER, core_vector::VEC_HEADER},
            vm::{Executable, Runtime},
        },
    };

    #[test]
    fn valid_primitive() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<u128>("var_u128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1);
            let res = test_extract_variable::<u64>("var_u64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<u32>("var_u32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3);
            let res = test_extract_variable::<u16>("var_u16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 4);
            let res = test_extract_variable::<u8>("var_u8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            let res = test_extract_variable::<i128>("var_i128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 6);
            let res = test_extract_variable::<i64>("var_i64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 7);
            let res = test_extract_variable::<i32>("var_i32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 8);
            let res = test_extract_variable::<i16>("var_i16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 9);
            let res = test_extract_variable::<i8>("var_i8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 10);
            true
        }

        test_statements(
            r##"
        
        let var_u128 = 1u128;
        let var_u64 = 2u64;
        let var_u32 = 3u32;
        let var_u16 = 4u16;
        let var_u8 = 5u8;
        let var_i128 = 6i128;
        let var_i64 = 7;
        let var_i32 = 8i32;
        let var_i16 = 9i16;
        let var_i8 = 10i8;
        
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_struct() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            test_extract_variable_with(
                "point",
                |address, stack, heap| {
                    let x = OpPrimitive::get_num_from::<u64>(address, stack, heap)
                        .expect("Deserialization should have succeeded");
                    let y = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)
                        .expect("Deserialization should have succeeded");

                    assert_eq!(x, 1);
                    assert_eq!(y, 2);
                },
                scope_manager,
                stack,
                heap,
            );
            true
        }

        test_statements(
            r##"
        struct Point {
            x : u64,
            y : u64,
        }
        let point = Point { x : 1, y : 2 };

        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_union() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            test_extract_variable_with(
                "point",
                |address, stack, heap| {
                    let x = OpPrimitive::get_num_from::<u64>(address, stack, heap)
                        .expect("Deserialization should have succeeded");
                    let y = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)
                        .expect("Deserialization should have succeeded");

                    assert_eq!(x, 1);
                    assert_eq!(y, 2);
                },
                scope_manager,
                stack,
                heap,
            );
            test_extract_variable_with(
                "point3d",
                |address, stack, heap| {
                    let x = OpPrimitive::get_num_from::<u64>(address, stack, heap)
                        .expect("Deserialization should have succeeded");
                    let y = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)
                        .expect("Deserialization should have succeeded");
                    let z = OpPrimitive::get_num_from::<u64>(address.add(16), stack, heap)
                        .expect("Deserialization should have succeeded");

                    assert_eq!(x, 1);
                    assert_eq!(y, 2);
                    assert_eq!(z, 3);
                },
                scope_manager,
                stack,
                heap,
            );
            true
        }

        test_statements(
            r##"
        union Test {
            Point {
                x : u64,
                y : u64,
            },
            Point3D {
                x : u64,
                y : u64,
                z : u64,
            },
        }
        let point = Test::Point { x : 1, y : 2 };
        let point3d = Test::Point3D { x : 1, y : 2, z: 3 };

        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_enum() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let x = test_extract_variable::<u64>("x", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(x, 0);
            let y = test_extract_variable::<u64>("y", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(y, 1);
            let z = test_extract_variable::<u64>("z", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(z, 2);
            true
        }

        test_statements(
            r##"
        enum Test {
            X,
            Y,
            Z,
        }
        let x = Test::X;
        let y = Test::Y;
        let z = Test::Z;

        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_slice() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            test_extract_variable_with(
                "arr",
                |address, stack, heap| {
                    let address: MemoryAddress =
                        OpPrimitive::get_num_from::<u64>(address, stack, heap)
                            .expect("Deserialization should have succeeded")
                            .try_into()
                            .unwrap();
                    let x = OpPrimitive::get_num_from::<u64>(address, stack, heap)
                        .expect("Deserialization should have succeeded");
                    let y = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)
                        .expect("Deserialization should have succeeded");
                    let z = OpPrimitive::get_num_from::<u64>(address.add(16), stack, heap)
                        .expect("Deserialization should have succeeded");

                    assert_eq!(x, 1);
                    assert_eq!(y, 2);
                    assert_eq!(z, 3);
                },
                scope_manager,
                stack,
                heap,
            );
            true
        }

        test_statements(
            r##"
        let arr = [1,2,3];
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_vector() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            test_extract_variable_with(
                "arr",
                |address, stack, heap| {
                    let address: MemoryAddress =
                        OpPrimitive::get_num_from::<u64>(address, stack, heap)
                            .expect("Deserialization should have succeeded")
                            .try_into()
                            .unwrap();
                    let address = address.add(VEC_HEADER);

                    let x = OpPrimitive::get_num_from::<u64>(address, stack, heap)
                        .expect("Deserialization should have succeeded");
                    let y = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)
                        .expect("Deserialization should have succeeded");
                    let z = OpPrimitive::get_num_from::<u64>(address.add(16), stack, heap)
                        .expect("Deserialization should have succeeded");

                    assert_eq!(x, 1);
                    assert_eq!(y, 2);
                    assert_eq!(z, 3);
                },
                scope_manager,
                stack,
                heap,
            );
            true
        }

        test_statements(
            r##"
        let arr = vec[1,2,3];
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_string() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            test_extract_variable_with(
                "text",
                |address, stack, heap| {
                    let address: MemoryAddress =
                        OpPrimitive::get_num_from::<u64>(address, stack, heap)
                            .expect("Deserialization should have succeeded")
                            .try_into()
                            .unwrap();
                    let address = address.add(8);

                    let text = OpPrimitive::get_string_from(address, stack, heap)
                        .expect("Deserialization should have succeeded");

                    assert_eq!(text, "Hello World");
                },
                scope_manager,
                stack,
                heap,
            );
            test_extract_variable_with(
                "text_complex",
                |address, stack, heap| {
                    let address: MemoryAddress =
                        OpPrimitive::get_num_from::<u64>(address, stack, heap)
                            .expect("Deserialization should have succeeded")
                            .try_into()
                            .unwrap();
                    let address = address.add(8);

                    let text = OpPrimitive::get_string_from(address, stack, heap)
                        .expect("Deserialization should have succeeded");

                    assert_eq!(text, "你好世界");
                },
                scope_manager,
                stack,
                heap,
            );
            true
        }

        test_statements(
            r##"
        let text = string("Hello World");
        let text_complex = string("你好世界");
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_tuple() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            test_extract_variable_with(
                "t1",
                |address, stack, heap| {
                    let res = OpPrimitive::get_bool_from(address, stack, heap)
                        .expect("Deserialization should have succeeded");

                    let chara = OpPrimitive::get_char_from(address.add(1), stack, heap)
                        .expect("Deserialization should have succeeded");
                    assert_eq!(res, true);
                    assert_eq!(chara, 'a');
                },
                scope_manager,
                stack,
                heap,
            );
            test_extract_variable_with(
                "t2",
                |address, stack, heap| {
                    let x = OpPrimitive::get_num_from::<u64>(address, stack, heap)
                        .expect("Deserialization should have succeeded");
                    let y = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)
                        .expect("Deserialization should have succeeded");
                    let z = OpPrimitive::get_num_from::<u64>(address.add(16), stack, heap)
                        .expect("Deserialization should have succeeded");

                    assert_eq!(x, 1);
                    assert_eq!(y, 2);
                    assert_eq!(z, 3);
                },
                scope_manager,
                stack,
                heap,
            );
            true
        }
        test_statements(
            r##"
            let t1 = (true,'a');
            let t2 = (1,2,3);
        "##,
            &mut engine,
            assert_fn,
        );
    }

    // #[test]
    // fn valid_number() {
    //     let (expr_i64, data_i64) = compile_expression!(Primitive, "420");
    //     assert_number!(expr_i64, data_i64, I64);

    //     let (expr_u128, data_u128) = compile_expression!(Primitive, "420u128");
    //     assert_number!(expr_u128, data_u128, U128);

    //     let (expr_u64, data_u64) = compile_expression!(Primitive, "420u64");
    //     assert_number!(expr_u64, data_u64, U64);

    //     let (expr_u32, data_u32) = compile_expression!(Primitive, "420u32");
    //     assert_number!(expr_u32, data_u32, U32);

    //     let (expr_u16, data_u16) = compile_expression!(Primitive, "420u16");
    //     assert_number!(expr_u16, data_u16, U16);

    //     let (expr_u8, data_u8) = compile_expression!(Primitive, "20u8");
    //     assert_number!(expr_u8, data_u8, U8);

    //     let (expr_i128, data_i128) = compile_expression!(Primitive, "420i128");
    //     assert_number!(expr_i128, data_i128, I128);

    //     let (expr_i64, data_i64) = compile_expression!(Primitive, "420i64");
    //     assert_number!(expr_i64, data_i64, I64);

    //     let (expr_i32, data_i32) = compile_expression!(Primitive, "420i32");
    //     assert_number!(expr_i32, data_i32, I32);

    //     let (expr_i16, data_i16) = compile_expression!(Primitive, "420i16");
    //     assert_number!(expr_i16, data_i16, I16);

    //     let (expr_i8, data_i8) = compile_expression!(Primitive, "20i8");
    //     assert_number!(expr_i8, data_i8, I8);
    // }

    // #[test]
    // fn valid_primitive() {
    //     let (expr, data) = compile_expression!(Primitive, "'a'");
    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(result, expr);
    //     let (expr, data) = compile_expression!(Primitive, "true");
    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Bool, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(result, expr);

    //     let (expr, data) = compile_expression!(Primitive, "420.69");
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, expr);
    // }

    // #[test]
    // fn valid_tuple() {
    //     let (expr, data) = compile_expression!(Tuple, "(true,17)");
    //     let result: Tuple = TupleType(vec![
    //         e_static!(crate::semantic::scope::static_types::StaticType::Primitive(
    //             PrimitiveType::Bool
    //         )),
    //         p_num!(I64),
    //     ])
    //     .deserialize_from(&data)
    //     .expect("Deserialization should have succeeded");
    //     // assert_eq!(result.value, expr.value);
    //     for (expected, res) in expr.value.into_iter().zip(result.value) {
    //         match (expected, res) {
    //             (
    //                 Expression::Atomic(Atomic::Data(Data::Primitive(expected))),
    //                 Expression::Atomic(Atomic::Data(Data::Primitive(res))),
    //             ) => {
    //                 assert_eq!(expected, res);
    //             }
    //             _ => assert!(false, "Expected boolean or u64"),
    //         }
    //     }
    //     let (expr, data) = compile_expression!(Tuple, "(420i128,true,17,'a')");
    //     let result: Tuple = TupleType(vec![
    //         p_num!(I128),
    //         e_static!(crate::semantic::scope::static_types::StaticType::Primitive(
    //             PrimitiveType::Bool
    //         )),
    //         p_num!(I64),
    //         e_static!(crate::semantic::scope::static_types::StaticType::Primitive(
    //             PrimitiveType::Char
    //         )),
    //     ])
    //     .deserialize_from(&data)
    //     .expect("Deserialization should have succeeded");
    //     // assert_eq!(result.value, expr.value);
    //     for (expected, res) in expr.value.into_iter().zip(result.value) {
    //         match (expected, res) {
    //             (
    //                 Expression::Atomic(Atomic::Data(Data::Primitive(expected))),
    //                 Expression::Atomic(Atomic::Data(Data::Primitive(res))),
    //             ) => {
    //                 assert_eq!(expected, res);
    //             }
    //             _ => assert!(false, "Expected boolean or u64"),
    //         }
    //     }
    // }

    // #[test]
    // fn valid_struct() {
    //     let user_type = user_type_impl::Struct {
    //         id: "Point".to_string().into(),
    //         fields: {
    //             let mut res = Vec::new();
    //             res.push(("x".to_string().into(), p_num!(U64)));
    //             res.push(("y".to_string().into(), p_num!(I32)));
    //             res
    //         },
    //     };
    //     let (expr, data) = compile_expression_with_type!(
    //         Struct,
    //         r##"
    //     Point {
    //         x : 420u64,
    //         y : 69i32
    //     }
    //     "##,
    //         "Point",
    //         user_type
    //     );
    //     let result: Struct = user_type
    //         .deserialize_from(&data)
    //         .expect("Deserialization should have succeeded");
    //     for ((e_id, expected), (r_id, res)) in expr.fields.into_iter().zip(result.fields) {
    //         if e_id != r_id {
    //             assert!(false, "Expected matching field ids")
    //         }
    //         match (expected, res) {
    //             (
    //                 Expression::Atomic(Atomic::Data(Data::Primitive(expected))),
    //                 Expression::Atomic(Atomic::Data(Data::Primitive(res))),
    //             ) => {
    //                 assert_eq!(expected, res);
    //             }
    //             _ => assert!(false, "Expected primitives"),
    //         }
    //     }
    // }

    // #[test]
    // fn valid_struct_complex() {
    //     let user_type = user_type_impl::Struct {
    //         id: "Point".to_string().into(),
    //         fields: {
    //             let mut res = Vec::new();
    //             res.push(("x".to_string().into(), p_num!(I32)));
    //             res.push(("y".to_string().into(), p_num!(I64)));
    //             res
    //         },
    //     };
    //     let (expr, data) = compile_expression_with_type!(
    //         Struct,
    //         r##"
    //     Point {
    //         x : 420,
    //         y : 69
    //     }
    //     "##,
    //         "Point",
    //         user_type
    //     );
    //     let result: Struct = user_type
    //         .deserialize_from(&data)
    //         .expect("Deserialization should have succeeded");
    //     for ((e_id, expected), (r_id, res)) in expr.fields.into_iter().zip(result.fields) {
    //         if e_id != r_id {
    //             assert!(false, "Expected matching field ids")
    //         }
    //         match (expected, res) {
    //             (
    //                 Expression::Atomic(Atomic::Data(Data::Primitive(expected))),
    //                 Expression::Atomic(Atomic::Data(Data::Primitive(res))),
    //             ) => {
    //                 assert_eq!(expected, res);
    //             }
    //             _ => assert!(false, "Expected primitives"),
    //         }
    //     }
    // }

    // #[test]
    // fn valid_union() {
    //     let user_type = user_type_impl::Union {
    //         id: "Geo".to_string().into(),
    //         variants: {
    //             let mut res = Vec::new();
    //             res.push((
    //                 "Point".to_string().into(),
    //                 user_type_impl::Struct {
    //                     id: "Point".to_string().into(),
    //                     fields: vec![
    //                         ("x".to_string().into(), p_num!(I64)),
    //                         ("y".to_string().into(), p_num!(I64)),
    //                     ],
    //                 },
    //             ));
    //             res.push((
    //                 "Axe".to_string().into(),
    //                 user_type_impl::Struct {
    //                     id: "Axe".to_string().into(),
    //                     fields: {
    //                         let mut res = Vec::new();
    //                         res.push(("x".to_string().into(), p_num!(I64)));
    //                         res
    //                     },
    //                 },
    //             ));
    //             res
    //         },
    //     };
    //     let (expr, data) = compile_expression_with_type!(
    //         Union,
    //         r##"
    //         Geo::Point { x : 2, y : 8}
    //     "##,
    //         "Geo",
    //         user_type
    //     );
    //     let result: Union = user_type
    //         .deserialize_from(&data)
    //         .expect("Deserialization should have succeeded");
    //     if expr.variant != result.variant {
    //         assert!(false, "Expected matching field ids")
    //     }
    //     for ((e_id, expected), (r_id, res)) in expr.fields.into_iter().zip(result.fields) {
    //         if e_id != r_id {
    //             assert!(false, "Expected matching field ids")
    //         }
    //         match (expected, res) {
    //             (
    //                 Expression::Atomic(Atomic::Data(Data::Primitive(expected))),
    //                 Expression::Atomic(Atomic::Data(Data::Primitive(res))),
    //             ) => {
    //                 assert_eq!(expected, res);
    //             }
    //             _ => assert!(false, "Expected primitives"),
    //         }
    //     }
    // }

    // #[test]
    // fn valid_enum() {
    //     let user_type = user_type_impl::Enum {
    //         id: "Geo".to_string().into(),
    //         values: {
    //             let mut res = Vec::new();
    //             res.push("Axe".to_string().into());
    //             res.push("Point".to_string().into());
    //             res.push("Space".to_string().into());
    //             res
    //         },
    //     };
    //     let (expr, data) = compile_expression_with_type!(
    //         Enum,
    //         r##"
    //         Geo::Point
    //     "##,
    //         "Geo",
    //         user_type
    //     );
    //     let result: Enum =
    //         <user_type_impl::Enum as DeserializeFrom>::deserialize_from(&user_type, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(expr, result)
    // }

    // #[test]
    // fn valid_slice() {
    //     let (expr, data) = compile_expression!(Slice, "[1,2,3]");
    //     let result: Slice = SliceType {
    //         size: 3,
    //         item_type: Box::new(p_num!(I64)),
    //     }
    //     .deserialize_from(&data)
    //     .expect("Deserialization should have succeeded");

    //     let result: Vec<Option<i64>> = result
    //         .value
    //         .iter()
    //         .map(|e| match e {
    //             Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
    //                 match x {
    //                     Number::I64(n) => Some(*n),
    //                     _ => None,
    //                 }
    //             }
    //             _ => None,
    //         })
    //         .collect();
    //     assert_eq!(result, vec![Some(1), Some(2), Some(3)])
    // }

    // #[test]
    // fn valid_vector() {
    //     // Parse the expression.
    //     let mut expr = Vector::parse("vec[1,2,3,4]".into())
    //         .expect("Parsing should have succeeded")
    //         .1;

    //     // Create a new block.
    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     // Perform semantic check.
    //     expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = CasmProgram::default();
    //     expr.gencode(
    //         &mut scope_manager,
    //         None,
    //         &mut instructions,
    //         &crate::vm::vm::CodeGenerationContext::default(),
    //     )
    //     .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);

    //     // Execute the instructions.

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);
    //     let arr: [u8; 8] = data.try_into().expect("");
    //     let heap_address = u64::from_le_bytes(arr);

    //     let data = heap
    //         .read(
    //             MemoryAddress::Heap {
    //                 offset: heap_address as usize,
    //             },
    //             8 * 4 + 16,
    //         )
    //         .expect("Heap Read should have succeeded");

    //     let result: Vector = VecType(Box::new(p_num!(I64)))
    //         .deserialize_from(&data)
    //         .expect("Deserialization should have succeeded");

    //     let result: Vec<Option<i64>> = result
    //         .value
    //         .iter()
    //         .map(|e| match e {
    //             Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
    //                 match x {
    //                     Number::I64(n) => Some(*n),
    //                     _ => None,
    //                 }
    //             }
    //             _ => None,
    //         })
    //         .collect();
    //     assert_eq!(result, vec![Some(1), Some(2), Some(3), Some(4)])
    // }

    // #[test]
    // fn valid_string() {
    //     let (expr, data) = compile_expression!(StrSlice, "\"Hello World\"");

    //     let result: StrSlice =
    //         <StrSliceType as DeserializeFrom>::deserialize_from(&&StrSliceType {}, &data)
    //             .expect("Deserialization should have succeeded");

    //     assert_eq!(result.value, "Hello World")
    // }

    // #[test]
    // fn valid_string_complex() {
    //     let (expr, data) = compile_expression!(StrSlice, "\"你好世界\"");

    //     let result: StrSlice =
    //         <StrSliceType as DeserializeFrom>::deserialize_from(&StrSliceType {}, &data)
    //             .expect("Deserialization should have succeeded");

    //     assert_eq!(result.value, "你好世界")
    // }

    // #[test]
    // fn valid_array_access() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let arr = [1,2,3,4];
    //         return arr[2];
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 3));
    // }

    // #[test]
    // fn valid_vec_access() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let arr = vec[1,2,3,4];
    //         return arr[2];
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 3));
    // }

    // #[test]
    // fn valid_str_slice_access() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let arr = "Hello World";
    //         return arr[2];
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(result, Primitive::Char('l'));
    // }

    // #[test]
    // fn valid_string_access() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let arr = string("Hello World");
    //         return arr[2];
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(result, Primitive::Char('l'));
    // }

    // #[test]
    // fn valid_tuple_access() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let tuple = (420,69);
    //         return tuple.1;
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 69));
    // }

    // #[test]
    // fn valid_field_access() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         struct Point {
    //             x : i64,
    //             y : i64,
    //         }
    //         let point = Point {
    //             x : 420,
    //             y : 69,
    //         };
    //         return point.y;
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 69));
    // }

    // #[test]
    // fn valid_empty_struct() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         struct Phantom {}
    //         let var = Phantom{};
    //         let copy = (var,50);
    //         return var;
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     assert_eq!(data.len(), 0)
    // }
    // #[test]
    // fn valid_closure() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let f = (x:u64) ->{
    //             let y:u64;
    //             let u:u64;
    //             let i:u64;
    //             let o:u64;
    //             let e:u64;

    //             return x+1;
    //         };
    //         return f(68);
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 69));
    // }
    // #[test]
    // fn valid_closure_with_stack_env() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let env:u64 = 31;

    //         let f = move (x:u64) -> {
    //             return env + x;
    //         };
    //         env = 0;
    //         return f(38);
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 69));
    // }

    // #[test]
    // fn valid_closure_with_heap_env() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let env : Vec<u64> = vec[2,5];
    //         let f = move (x:u64) -> {
    //             return env[1] + x;
    //         };
    //         env[1] = 31;
    //         return f(38);
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 69));
    // }

    // #[test]
    // fn valid_closure_rec() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let blob = 5;
    //         let rec f : fn(u64) -> u64 = (x:u64) -> {
    //             if x >= 5 {
    //                 return 5u64;
    //             }
    //             return f(x+1);
    //         };
    //         return f(0);
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 5));
    // }

    // #[test]
    // fn valid_closure_rec_with_env() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let env = 1u64;
    //         let rec f : dyn fn(u64) -> u64 = move (x:u64) -> {
    //             let y:u64;
    //             let z:u64;
    //             let e:u64;
    //             let t:u64;
    //             let u:u64;
    //             if x >= 5 {
    //                 return 5u64;
    //             }
    //             return f(x+env);
    //         };
    //         return f(0);
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 5));
    // }

    // #[test]
    // fn valid_addr_complex() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let arr = vec[1,2,3,4];
    //         let len = *(arr as &u64);
    //         return len;
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 4));
    // }

    // #[test]
    // fn valid_ptr_access_complex() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let arr = vec[1,2,3,4];
    //         let first = *(((arr as &Any) as u64 + 16) as &u64);
    //         return first;
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 1));
    // }

    // #[test]
    // fn valid_map_init() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let hmap : Map<u64,u64> = map {
    //             5:1,
    //             8:2,
    //             9:3,
    //             7:4,
    //             1:5,
    //         };
    //         let (value,_) = get(&hmap,9);
    //         return value;
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 3));
    // }

    // #[test]
    // fn valid_map_init_no_typedef() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let hmap = map {
    //             "test":1,
    //             "test1":2,
    //             "test11":3,
    //             "test111":4,
    //             "test1111":5,
    //         };
    //         let (value,_) = get(&hmap,"test11");
    //         return value;
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 3));
    // }
}
