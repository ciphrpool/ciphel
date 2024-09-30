use num_traits::ToBytes;

use crate::ast::expressions::locate::Locatable;
use crate::ast::expressions::{CompletePath, Path};
use crate::semantic::scope::scope::GlobalMapping;
use crate::semantic::scope::static_types::POINTER_SIZE;
use crate::semantic::scope::static_types::{MapType, StaticType, TupleType};
use crate::vm::asm::locate::LocateOffset;
use crate::vm::core::format::FormatAsm;
use crate::vm::core::map::MapAsm;
use crate::vm::core::vector::VEC_HEADER;
use crate::vm::core::CoreAsm;
use crate::vm::{CodeGenerationError, GenerateCode};
use crate::{
    semantic::{scope::user_types::UserType, EType, SizeOf},
    vm::{
        allocator::{align, MemoryAddress},
        asm::{
            alloc::{Access, Alloc},
            branch::{Goto, Label},
            data,
            locate::Locate,
            mem::Mem,
            Asm,
        },
    },
};

use super::{
    Address, Call, Closure, ClosureReprData, CoreCall, Data, Enum, Format, Lambda, Map, Number,
    Primitive, Printf, PtrAccess, Slice, StrSlice, Struct, Tuple, Union, VarCall, Variable, Vector,
};

impl GenerateCode for Data {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            Data::Primitive(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Data::Slice(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Data::Vec(value) => value.gencode::<E>(scope_manager, scope_id, instructions, context),
            Data::Closure(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Data::Tuple(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Data::Address(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Data::PtrAccess(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Data::Variable(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Data::Unit => Ok(()),
            Data::Map(value) => value.gencode::<E>(scope_manager, scope_id, instructions, context),
            Data::Struct(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Data::Union(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Data::Enum(value) => value.gencode::<E>(scope_manager, scope_id, instructions, context),
            Data::StrSlice(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Data::Call(value) => value.gencode::<E>(scope_manager, scope_id, instructions, context),
            Data::Lambda(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Data::Printf(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Data::Format(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
        }
    }
}

impl GenerateCode for Number {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let asm = match self {
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
        instructions.push(Asm::Data(asm));
        Ok(())
    }
}

impl GenerateCode for Primitive {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let asm = match self {
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
        instructions.push(Asm::Data(asm));
        Ok(())
    }
}

impl GenerateCode for Variable {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let Some(super::VariableState::Variable { id }) = &self.state else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Ok(crate::semantic::scope::scope::VariableInfo {
            ctype,
            address,
            marked_as_closed_var,
            ..
        }) = scope_manager.find_var_by_id(*id)
        else {
            return Err(CodeGenerationError::Unlocatable);
        };

        if let Some((env_address, offset)) = marked_as_closed_var.get(scope_id, &scope_manager) {
            // The variable is in a closed environment
            instructions.push(Asm::Access(Access::Static {
                address: env_address,
                size: POINTER_SIZE,
            }));
            instructions.push(Asm::Offset(LocateOffset { offset }));
            instructions.push(Asm::Access(Access::Runtime {
                size: Some(ctype.size_of()),
            }));
        } else {
            let Ok(address) = (address.clone()).try_into() else {
                return Err(CodeGenerationError::Unlocatable);
            };
            instructions.push(Asm::Access(Access::Static {
                address,
                size: ctype.size_of(),
            }));
        }

        Ok(())
    }
}

impl GenerateCode for Slice {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        for element in &self.value {
            let _ = element.gencode::<E>(scope_manager, scope_id, instructions, context)?;
        }
        if let Some(address) = self.address {
            instructions.push(Asm::Mem(Mem::Store {
                size: self.size,
                address,
            }));
            instructions.push(Asm::Locate(Locate { address }));
        } else {
            let address = scope_manager.global_mapping.alloc(self.size);
            let address = address
                .try_into()
                .map_err(|_| CodeGenerationError::Default)?;
            instructions.push(Asm::Alloc(Alloc::GlobalFromStack {
                address,
                size: self.size,
            }));
        }

        Ok(())
    }
}

impl GenerateCode for StrSlice {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let mut buffer = (self.value.len()).to_le_bytes().to_vec();
        buffer.extend_from_slice(self.value.as_bytes());

        if self.value != "" {
            if let Some(address) = self.address {
                instructions.push(Asm::Data(data::Data::Serialized {
                    data: buffer.into(),
                }));
                instructions.push(Asm::Mem(Mem::Store {
                    size: POINTER_SIZE + self.value.len(),
                    address,
                }));
                instructions.push(Asm::Locate(Locate { address }));
            } else {
                let address = scope_manager
                    .global_mapping
                    .alloc(POINTER_SIZE + self.value.len());

                let address = address
                    .try_into()
                    .map_err(|_| CodeGenerationError::Default)?;
                instructions.push(Asm::Alloc(Alloc::Global {
                    address,
                    data: buffer.into(),
                }));
            }
        } else {
            instructions.push(Asm::Alloc(Alloc::Global {
                address: GlobalMapping::EMPTY_STRING_ADDRESS,
                data: buffer.into(),
            }));
        }

        Ok(())
    }
}

impl GenerateCode for Vector {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let Some(EType::Static(StaticType::Vec(item_type))) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let item_size = item_type.size_of();

        let len_bytes = (self.length as u64).to_le_bytes().as_slice().into();
        let cap_bytes = (self.capacity as u64).to_le_bytes().as_slice().into();

        // Push Length on stack
        instructions.push(Asm::Data(data::Data::Serialized { data: len_bytes }));
        // Push Capacity on stack
        instructions.push(Asm::Data(data::Data::Serialized { data: cap_bytes }));

        for element in &self.value {
            let _ = element.gencode::<E>(scope_manager, scope_id, instructions, context)?;
        }

        // Copy data on stack to heap at address

        // Alloc and push heap address on stack
        instructions.push(Asm::Alloc(Alloc::Heap {
            size: item_size * self.capacity + VEC_HEADER,
        }));
        // Take the address on the top of the stack
        // and copy the data on the stack in the heap at given address and given offset
        // ( removing the data from the stack )
        instructions.push(Asm::Mem(Mem::Take {
            //offset: vec_stack_address + 8,
            size: item_size * self.value.len() + VEC_HEADER,
        }));

        Ok(())
    }
}

impl GenerateCode for Tuple {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let Some(EType::Static(StaticType::Tuple(TupleType(tuple_types)))) =
            self.metadata.signature()
        else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        for (expr, expr_type) in self.value.iter().zip(tuple_types) {
            let _ = expr.gencode::<E>(scope_manager, scope_id, instructions, context)?;
        }
        Ok(())
    }
}

impl GenerateCode for ClosureReprData {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        instructions.push(Asm::Mem(Mem::Label(self.closure_label)));
        instructions.push(Asm::Data(data::Data::Serialized {
            data: self.bucket_size.to_le_bytes().into(),
        }));
        for (id, _) in &self.offsets {
            let Ok(crate::semantic::scope::scope::VariableInfo { ctype, address, .. }) =
                scope_manager.find_var_by_id(*id)
            else {
                return Err(CodeGenerationError::Unlocatable);
            };

            let Ok(address) = (address.clone()).try_into() else {
                return Err(CodeGenerationError::Unlocatable);
            };

            instructions.push(Asm::Access(Access::Static {
                address,
                size: ctype.size_of(),
            }));
        }

        // Alloc Repr data
        instructions.push(Asm::Alloc(Alloc::Heap {
            size: self.bucket_size + POINTER_SIZE * 2,
        })); // the heap pointer is pushed on the stack

        instructions.push(Asm::Mem(Mem::Take {
            size: self.bucket_size + POINTER_SIZE * 2,
        })); // the heap pointer is pushed on the stack

        Ok(())
    }
}

impl GenerateCode for Closure {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let store_label = Label::gen();

        let Some(repr_data) = &self.repr_data else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(inner_scope) = self.block.scope else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        for (id, offset) in &repr_data.offsets {
            let _ = scope_manager.mark_as_closed_var(
                inner_scope,
                *id,
                MemoryAddress::Frame { offset: 0 },
                *offset + POINTER_SIZE * 2,
            )?;
        }

        instructions.push(Asm::Goto(Goto {
            label: Some(store_label),
        }));
        instructions.push_label_by_id(repr_data.closure_label, "closure".to_string());
        self.block
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;

        instructions.push_label_by_id(store_label, "store_closure".to_string());

        repr_data.gencode::<E>(scope_manager, scope_id, instructions, context)?;
        Ok(())
    }
}

impl GenerateCode for Lambda {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let lambda_label = Label::gen();
        let store_label = Label::gen();

        instructions.push(Asm::Goto(Goto {
            label: Some(store_label),
        }));
        instructions.push_label_by_id(lambda_label, "lambda".to_string());
        self.block
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;

        instructions.push_label_by_id(store_label, "store_lambda".to_string());

        instructions.push(Asm::Mem(Mem::Label(lambda_label)));

        Ok(())
    }
}

impl GenerateCode for Address {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self.value.locate(scope_manager, scope_id, instructions)? {
            Some(address) => {
                instructions.push(Asm::Locate(Locate { address }));
            }
            None => {
                // Noop : the address was push on the stack
            }
        }
        Ok(())
    }
}

impl GenerateCode for PtrAccess {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let _ = self
            .value
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;
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
        instructions.push(Asm::Access(Access::Runtime { size: Some(size) }));

        Ok(())
    }
}

impl GenerateCode for Struct {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let Some(EType::User { id, size }) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(UserType::Struct(crate::semantic::scope::user_types::Struct { id, fields })) =
            scope_manager.find_type_by_id(id, scope_id).ok()
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

            let _ = field_expr.gencode::<E>(scope_manager, scope_id, instructions, context)?;
            // // Add padding
            // let padding = align(field_size) - field_size;
            // if padding > 0 {
            //     instructions.push(Asm::Data(data::Data::Serialized {
            //         data: vec![0; padding].into(),
            //     }));
            // }
        }
        Ok(())
    }
}

impl GenerateCode for Union {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let Some(EType::User { id, size }) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(UserType::Union(
            ref union_type @ crate::semantic::scope::user_types::Union { ref variants, .. },
        )) = scope_manager.find_type_by_id(id, scope_id).ok()
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

            let _ = field_expr.gencode::<E>(scope_manager, scope_id, instructions, context)?;
            total_size += field_size;
        }

        if total_size < union_size {
            // Add padding
            let padding = union_size - total_size;
            if padding > 0 {
                instructions.push(Asm::Data(data::Data::Serialized {
                    data: vec![0; padding].into(),
                }));
            }
        }

        instructions.push(Asm::Data(data::Data::Serialized {
            data: (idx as u64).to_le_bytes().into(),
        }));

        Ok(())
    }
}

impl GenerateCode for Enum {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let Some(EType::User { id, size }) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(UserType::Enum(crate::semantic::scope::user_types::Enum { values, .. })) =
            scope_manager.find_type_by_id(id, scope_id).ok()
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

        instructions.push(Asm::Data(data::Data::Serialized {
            data: (index as u64).to_le_bytes().into(),
        }));
        Ok(())
    }
}

impl GenerateCode for Map {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
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
        instructions.push(Asm::Data(data::Data::Serialized {
            data: cap.to_le_bytes().into(),
        }));
        instructions.push(Asm::Core(CoreAsm::Map(MapAsm::MapWithCapacity {
            key_size,
            item_size,
        })));

        for (key, value) in &self.fields {
            let _ = key.gencode::<E>(scope_manager, scope_id, instructions, context)?;
            let _ = value.gencode::<E>(scope_manager, scope_id, instructions, context)?;

            instructions.push(Asm::Core(CoreAsm::Map(MapAsm::Insert {
                key_size,
                ref_access: keys_type.as_ref().into(),
                item_size,
            })));
        }

        Ok(())
    }
}

impl GenerateCode for Call {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match &self.path {
            super::LeftCall::VarCall(VarCall {
                path: CompletePath { path, name },
                id,
                is_closure,
            }) => match path {
                Path::Segment(vec) => todo!("module function"),
                Path::Empty => {
                    let Some(id) = id else {
                        return Err(CodeGenerationError::UnresolvedError);
                    };
                    let Some(param_size) = self.args.size else {
                        return Err(CodeGenerationError::UnresolvedError);
                    };
                    let Ok(crate::semantic::scope::scope::VariableInfo { address, .. }) =
                        scope_manager.find_var_by_id(*id)
                    else {
                        return Err(CodeGenerationError::Unlocatable);
                    };
                    let Ok(address) = address.clone().try_into() else {
                        return Err(CodeGenerationError::Unlocatable);
                    };

                    for arg in self.args.args.iter() {
                        arg.gencode::<E>(scope_manager, scope_id, instructions, context)?;
                    }

                    // call function stored in this address
                    instructions.push(Asm::Access(Access::Static {
                        address,
                        size: POINTER_SIZE,
                    }));

                    if *is_closure {
                        instructions.push(Asm::Call(crate::vm::asm::branch::Call::Closure {
                            param_size,
                        }));
                    } else {
                        instructions.push(Asm::Call(crate::vm::asm::branch::Call::Function {
                            param_size,
                        }));
                    }

                    Ok(())
                }
            },
            super::LeftCall::ExternCall(extern_call) => todo!(),
            super::LeftCall::CoreCall(CoreCall { path }) => {
                for arg in self.args.args.iter() {
                    arg.gencode::<E>(scope_manager, scope_id, instructions, context)?;
                }
                path.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
        }
    }
}

impl GenerateCode for Printf {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PrintfStart)));

        for segment in self.args.iter() {
            match segment {
                super::FormatItem::Str(segment) => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStr(
                        segment.as_bytes().into(),
                    ))));
                }
                super::FormatItem::Expr(expression) => {
                    let _ =
                        expression.gencode::<E>(scope_manager, scope_id, instructions, context)?;

                    let Some(ctype) = expression.signature() else {
                        return Err(CodeGenerationError::UnresolvedError);
                    };

                    let _ = crate::vm::core::format::type_printer::build(
                        &ctype,
                        scope_manager,
                        scope_id,
                        instructions,
                    )?;
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::Push)));
                }
            }
        }

        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PrintfEnd)));
        Ok(())
    }
}

impl GenerateCode for Format {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));

        for segment in self.args.iter() {
            match segment {
                super::FormatItem::Str(segment) => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStr(
                        segment.as_bytes().into(),
                    ))));
                }
                super::FormatItem::Expr(expression) => {
                    let _ =
                        expression.gencode::<E>(scope_manager, scope_id, instructions, context)?;

                    let Some(ctype) = expression.signature() else {
                        return Err(CodeGenerationError::UnresolvedError);
                    };

                    let _ = crate::vm::core::format::type_printer::build(
                        &ctype,
                        scope_manager,
                        scope_id,
                        instructions,
                    )?;
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::Push)));
                }
            }
        }

        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        test_extract_variable, test_extract_variable_with, test_statements,
        vm::{
            asm::operation::{GetNumFrom, OpPrimitive},
            core::vector::VEC_HEADER,
        },
    };

    #[test]
    fn valid_primitive() {
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
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
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
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
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
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
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
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
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
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

        let res = {
            let buf = [1,2,3,4];
            unit
        };
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_vector() {
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
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
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
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
        let text = core::string::string("Hello World");
        let text_complex = string("你好世界");

        
        let res = {
            let buf = "Hello World";
            unit
        };
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_tuple() {
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
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

    #[test]
    fn valid_lambda() {
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<u64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<u64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 6);
            let res = test_extract_variable::<u64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3);
            let res = test_extract_variable::<u64>("res4", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 7);
            let res = test_extract_variable::<u64>("res5", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1);
            true
        }
        test_statements(
            r##"
            let lambda1 = (x:i64) -> x + 1; 
            let res1 = lambda1(1);

            let lambda2 = (x:i64) -> {
                let y = 5;
                return y + x;
            }; 
            let res2 = lambda2(1);

            let z = 2;
            let lambda3 : (i64) -> i64 = (x:i64) -> {x + z}; 
            let res3 = lambda3(1);
            
            let arr = [lambda1,lambda2];
            let res4 = arr[1](2);

            let lambda5 = (x:i64) -> {vec[x]}; 
            let res5 = lambda5(1)[0];

        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_closure() {
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<u64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3);
            let res = test_extract_variable::<u64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<u64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            let res = test_extract_variable::<u64>("res4", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            true
        }
        test_statements(
            r##"
            let closure1 = {
                let z = 2;
                let closure1 = move (x:i64) -> x + z;
                let u = z + 1;
                closure1
            };
            
            let res1 = closure1(1);

            let closure2 = {
                let z = 2;
                let closure2 = move (x:i64) -> x + 1;
                let u = z + 1;
                closure2
            };
            
            let res2 = closure2(1);

            let closure3 = {
                let z = 2;
                let closure3 = move (x:i64) -> {
                    z = z + x;
                    z
                };
                z = z + 1;
                closure3
            };
            
            let res3 = closure3(1);
            res3 = closure3(1);
            res3 = closure3(1);

            let closure4 = {
                let arr = vec[1,2,3];
                let closure4 = move (x:i64) -> {
                    arr[1] = arr[1] + x;
                    arr[1]
                };
                closure4
            };
            
            let res4 = closure4(1);
            res4 = closure4(1);
            res4 = closure4(1);
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
    //     let user_type = user_types::Struct {
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
    //     let user_type = user_types::Struct {
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
    //     let user_type = user_types::Union {
    //         id: "Geo".to_string().into(),
    //         variants: {
    //             let mut res = Vec::new();
    //             res.push((
    //                 "Point".to_string().into(),
    //                 user_types::Struct {
    //                     id: "Point".to_string().into(),
    //                     fields: vec![
    //                         ("x".to_string().into(), p_num!(I64)),
    //                         ("y".to_string().into(), p_num!(I64)),
    //                     ],
    //                 },
    //             ));
    //             res.push((
    //                 "Axe".to_string().into(),
    //                 user_types::Struct {
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
    //     let user_type = user_types::Enum {
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
    //         <user_types::Enum as DeserializeFrom>::deserialize_from(&user_type, &data)
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
    //     expr.resolve::<crate::vm::external::test::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = Program::default();
    //     expr.gencode::<E>(
    //         &mut scope_manager,
    //         None,
    //         &mut instructions,
    //         &crate::vm::CodeGenerationContext::default(),
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
    //     let mut engine = crate::vm::external::test::NoopGameEngine {};

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
