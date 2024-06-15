use num_traits::ToBytes;

use crate::arw_read;
use crate::semantic::scope::scope::Scope;
use crate::semantic::AccessLevel;
use crate::vm::allocator::stack::Offset;
use crate::vm::platform::core::alloc::{AllocCasm, DerefHashing};
use crate::vm::platform::core::CoreCasm;
use crate::vm::platform::LibCasm;
use crate::vm::vm::Locatable;
use crate::{
    ast::utils::strings::ID,
    semantic::{
        scope::{type_traits::GetSubTypes, user_type_impl::UserType},
        Either, SizeOf,
    },
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
    Address, Closure, Data, Enum, ExprScope, Map, Number, Primitive, PtrAccess, Slice, StrSlice,
    Struct, Tuple, Union, Variable, Vector,
};

impl GenerateCode for Data {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Data::Primitive(value) => value.gencode(scope, instructions),
            Data::Slice(value) => value.gencode(scope, instructions),
            Data::Vec(value) => value.gencode(scope, instructions),
            Data::Closure(value) => value.gencode(scope, instructions),
            Data::Tuple(value) => value.gencode(scope, instructions),
            Data::Address(value) => value.gencode(scope, instructions),
            Data::PtrAccess(value) => value.gencode(scope, instructions),
            Data::Variable(value) => value.gencode(scope, instructions),
            Data::Unit => Ok(()),
            Data::Map(value) => value.gencode(scope, instructions),
            Data::Struct(value) => value.gencode(scope, instructions),
            Data::Union(value) => value.gencode(scope, instructions),
            Data::Enum(value) => value.gencode(scope, instructions),
            Data::StrSlice(value) => value.gencode(scope, instructions),
        }
    }
}

impl Locatable for Data {
    fn locate(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Data::Variable(Variable {
                id,
                from_field,
                metadata,
            }) => {
                if *from_field {
                    Ok(())
                } else {
                    let (var, address, level) =
                        crate::arw_read!(scope, CodeGenerationError::ConcurrencyError)?
                            .access_var(id)?;

                    let var_type = &arw_read!(var, CodeGenerationError::ConcurrencyError)?.type_sig;
                    let _var_size = var_type.size_of();

                    instructions.push(Casm::Locate(Locate {
                        address: MemoryAddress::Stack {
                            offset: address,
                            level,
                        },
                    }));
                    Ok(())
                }
            }
            Data::PtrAccess(PtrAccess { value, .. }) => {
                let _ = value.gencode(scope, instructions)?;
                Ok(())
            }
            _ => {
                let _ = self.gencode(scope, instructions)?;
                let Some(value_type) = self.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                instructions.push(Casm::Locate(Locate {
                    address: MemoryAddress::Stack {
                        offset: Offset::ST(-(value_type.size_of() as isize)),
                        level: AccessLevel::Direct,
                    },
                }));
                Ok(())
            }
        }
    }

    fn is_assignable(&self) -> bool {
        match self {
            Data::Variable(_) => true,
            Data::PtrAccess(_) => true,
            Data::Primitive(_) => false,
            Data::Slice(_) => false,
            Data::StrSlice(_) => false,
            Data::Vec(_) => false,
            Data::Closure(_) => false,
            Data::Tuple(_) => false,
            Data::Address(_) => false,
            Data::Unit => false,
            Data::Map(_) => false,
            Data::Struct(_) => false,
            Data::Union(_) => false,
            Data::Enum(_) => false,
        }
    }

    fn most_left_id(&self) -> Option<ID> {
        match self {
            Data::Variable(Variable { id, metadata, .. }) => Some(id.clone()),
            _ => None,
        }
    }
}

impl GenerateCode for Number {
    fn gencode(
        &self,
        _scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
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
        _scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
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
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        if self.from_field {
            let Some(var_type) = self.metadata.signature() else {
                return Err(CodeGenerationError::UnresolvedError);
            };
            let var_size = var_type.size_of();
            if var_size == 0 {
                return Ok(());
            }
            instructions.push(Casm::Access(Access::Runtime {
                size: Some(var_size),
            }));
            Ok(())
        } else {
            let (var, address, level) =
                crate::arw_read!(scope, CodeGenerationError::ConcurrencyError)?
                    .access_var(&self.id)?;
            // dbg!((&var, &address, &level));
            let var_type = &arw_read!(var, CodeGenerationError::ConcurrencyError)?.type_sig;
            let var_size = var_type.size_of();
            if var_size == 0 {
                return Ok(());
            }
            instructions.push(Casm::Access(Access::Static {
                address: MemoryAddress::Stack {
                    offset: address,
                    level,
                },
                size: var_size,
            }));

            Ok(())
        }
    }
}

impl GenerateCode for Slice {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(signature) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let _item_size = {
            let Some(item_type) = signature.get_item() else {
                return Err(CodeGenerationError::UnresolvedError);
            };
            item_type.size_of()
        };
        // // Push the size of the slice
        // let bytes = (self.value.len() as u64).to_le_bytes().as_slice().to_vec();
        // offset += bytes.len();
        // instructions.push(Casm::Data(Data::Serialized { data: bytes }));

        for element in &self.value {
            let _ = element.gencode(scope, instructions)?;
        }

        Ok(())
    }
}

impl GenerateCode for StrSlice {
    fn gencode(
        &self,
        _scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let str_bytes: Box<[u8]> = self.value.as_bytes().into();
        let size = (&str_bytes).len() as u64;
        instructions.push(Casm::Data(data::Data::Serialized { data: str_bytes }));
        let padding = self.padding;
        if padding > 0 {
            instructions.push(Casm::Data(data::Data::Serialized {
                data: vec![0; padding].into(),
            }));
        }
        instructions.push(Casm::Data(data::Data::Serialized {
            data: (size + padding as u64).to_le_bytes().into(),
        }));
        Ok(())
    }
}

impl GenerateCode for Vector {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(signature) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let item_size = {
            let Some(item_type) = signature.get_item() else {
                return Err(CodeGenerationError::UnresolvedError);
            };
            item_type.size_of()
        };

        let len_bytes = (self.length as u64).to_le_bytes().as_slice().into();
        let cap_bytes = (self.capacity as u64).to_le_bytes().as_slice().into();

        // Push Length on stack
        instructions.push(Casm::Data(data::Data::Serialized { data: len_bytes }));
        // Push Capacity on stack
        instructions.push(Casm::Data(data::Data::Serialized { data: cap_bytes }));

        for element in &self.value {
            let _ = element.gencode(scope, instructions)?;
        }

        // Copy data on stack to heap at address

        // Alloc and push heap address on stack
        instructions.push(Casm::Alloc(Alloc::Heap {
            size: Some(item_size * self.capacity + 16),
        }));
        // Take the address on the top of the stack
        // and copy the data on the stack in the heap at given address and given offset
        // ( removing the data from the stack )
        instructions.push(Casm::Mem(Mem::TakeToHeap {
            //offset: vec_stack_address + 8,
            size: item_size * self.value.len() + 16,
        }));

        Ok(())
    }
}

impl GenerateCode for Tuple {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(signature) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        for (idx, element) in self.value.iter().enumerate() {
            let _item_size = {
                let Some(item_type) =
                    // <EType as GetSubTypes>::get_nth(signature, &idx)
                    signature.get_nth(&idx)
                else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                item_type.size_of()
            };
            let _ = element.gencode(scope, instructions)?;
        }
        Ok(())
    }
}

impl GenerateCode for ExprScope {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            ExprScope::Scope(value) => value.gencode(scope, instructions),
            ExprScope::Expr(value) => value.gencode(scope, instructions),
        }
    }
}

impl GenerateCode for Closure {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let end_closure = Label::gen();

        instructions.push(Casm::Goto(Goto {
            label: Some(end_closure),
        }));

        let closure_label = instructions.push_label("fn_closure".into());
        let _ = self.scope.gencode(scope, instructions);
        instructions.push_label_id(end_closure, "end_closure".into());

        instructions.push(Casm::Mem(Mem::LabelOffset(closure_label)));

        if self.closed {
            /* Load env and store in the heap */
            let mut alloc_size = 16;
            let mut env_size = 0;

            let binding = self
                .scope
                .scope()
                .map_err(|_| CodeGenerationError::UnresolvedError)?;
            let inner_scope = binding.as_ref();

            for var in inner_scope
                .try_read()
                .map_err(|_| CodeGenerationError::ConcurrencyError)?
                .env_vars()
                .map_err(|_| CodeGenerationError::ConcurrencyError)?
            {
                let var_type = &var.type_sig;
                let var_size = var_type.size_of();
                alloc_size += var_size;
                env_size += var_size;
            }

            // Load Env Size
            instructions.push(Casm::Data(data::Data::Serialized {
                data: env_size.to_le_bytes().into(),
            }));
            let outer_scope = crate::arw_read!(scope, CodeGenerationError::ConcurrencyError)?;
            // Load Env variables
            for var in inner_scope
                .try_read()
                .map_err(|_| CodeGenerationError::ConcurrencyError)?
                .env_vars()
                .map_err(|_| CodeGenerationError::ConcurrencyError)?
            {
                let (var, address, level) = outer_scope.access_var(&var.id)?;
                let var_type = &arw_read!(var, CodeGenerationError::ConcurrencyError)?.type_sig;
                let var_size = var_type.size_of();
                instructions.push(Casm::Access(Access::Static {
                    address: MemoryAddress::Stack {
                        offset: address,
                        level: level,
                    },
                    size: var_size,
                }));
            }
            instructions.push(Casm::Alloc(Alloc::Heap {
                size: Some(alloc_size),
            }));
            instructions.push(Casm::Mem(Mem::TakeToHeap { size: alloc_size }));
        }

        Ok(())
    }
}

impl GenerateCode for Address {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        // let _ = rec_addr_gencode(&self.value, scope, instructions)?;
        let _ = self.value.locate(scope, instructions)?;
        Ok(())
    }
}

impl GenerateCode for PtrAccess {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let _ = self.value.gencode(scope, instructions)?;
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
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(signature) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Either::User(struct_type) = signature else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let UserType::Struct(struct_type) = struct_type.as_ref() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
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

            let _ = field_expr.gencode(scope, instructions)?;
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
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(signature) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Either::User(union_type) = signature else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let UserType::Union(union_type) = union_type.as_ref() else {
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

            let _ = field_expr.gencode(scope, instructions)?;
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
        _scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(signature) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Either::User(enum_type) = signature else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let UserType::Enum(enum_type) = enum_type.as_ref() else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let Some(index) = enum_type
            .values
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
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let cap = align(self.fields.len());
        let Some(map_type) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(key_type) = map_type.get_key() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(value_type) = map_type.get_item() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let key_size = key_type.size_of();
        let value_size = value_type.size_of();
        instructions.push(Casm::Data(data::Data::Serialized {
            data: cap.to_le_bytes().into(),
        }));
        instructions.push(Casm::Platform(LibCasm::Core(CoreCasm::Alloc(
            AllocCasm::MapWithCapacity {
                key_size,
                value_size,
            },
        ))));

        for (key, value) in &self.fields {
            let _ = key.gencode(scope, instructions)?;
            let _ = value.gencode(scope, instructions)?;

            instructions.push(Casm::Platform(LibCasm::Core(CoreCasm::Alloc(
                AllocCasm::InsertAndForward {
                    key_size,
                    value_size,
                    ref_access: DerefHashing::from(&map_type),
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
        compile_expression_with_type, compile_statement, e_static, p_num,
        semantic::{
            scope::{
                scope::Scope,
                static_types::{PrimitiveType, SliceType, StrSliceType, TupleType, VecType},
                user_type_impl,
            },
            Resolve,
        },
        v_num,
        vm::vm::{DeserializeFrom, Executable, Runtime},
    };
    use crate::{clear_stack, compile_expression};

    #[test]
    fn valid_number() {
        let (expr_i64, data_i64) = compile_expression!(Primitive, "420");
        assert_number!(expr_i64, data_i64, I64);

        let (expr_u128, data_u128) = compile_expression!(Primitive, "420u128");
        assert_number!(expr_u128, data_u128, U128);

        let (expr_u64, data_u64) = compile_expression!(Primitive, "420u64");
        assert_number!(expr_u64, data_u64, U64);

        let (expr_u32, data_u32) = compile_expression!(Primitive, "420u32");
        assert_number!(expr_u32, data_u32, U32);

        let (expr_u16, data_u16) = compile_expression!(Primitive, "420u16");
        assert_number!(expr_u16, data_u16, U16);

        let (expr_u8, data_u8) = compile_expression!(Primitive, "20u8");
        assert_number!(expr_u8, data_u8, U8);

        let (expr_i128, data_i128) = compile_expression!(Primitive, "420i128");
        assert_number!(expr_i128, data_i128, I128);

        let (expr_i64, data_i64) = compile_expression!(Primitive, "420i64");
        assert_number!(expr_i64, data_i64, I64);

        let (expr_i32, data_i32) = compile_expression!(Primitive, "420i32");
        assert_number!(expr_i32, data_i32, I32);

        let (expr_i16, data_i16) = compile_expression!(Primitive, "420i16");
        assert_number!(expr_i16, data_i16, I16);

        let (expr_i8, data_i8) = compile_expression!(Primitive, "20i8");
        assert_number!(expr_i8, data_i8, I8);
    }

    #[test]
    fn valid_primitive() {
        let (expr, data) = compile_expression!(Primitive, "'a'");
        let result =
            <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
                .expect("Deserialization should have succeeded");
        assert_eq!(result, expr);
        let (expr, data) = compile_expression!(Primitive, "true");
        let result =
            <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Bool, &data)
                .expect("Deserialization should have succeeded");
        assert_eq!(result, expr);

        let (expr, data) = compile_expression!(Primitive, "420.69");
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, expr);
    }

    #[test]
    fn valid_tuple() {
        let (expr, data) = compile_expression!(Tuple, "(true,17)");
        let result: Tuple = TupleType(vec![
            e_static!(crate::semantic::scope::static_types::StaticType::Primitive(
                PrimitiveType::Bool
            )),
            p_num!(I64),
        ])
        .deserialize_from(&data)
        .expect("Deserialization should have succeeded");
        // assert_eq!(result.value, expr.value);
        for (expected, res) in expr.value.into_iter().zip(result.value) {
            match (expected, res) {
                (
                    Expression::Atomic(Atomic::Data(Data::Primitive(expected))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(res))),
                ) => {
                    assert_eq!(expected, res);
                }
                _ => assert!(false, "Expected boolean or u64"),
            }
        }
        let (expr, data) = compile_expression!(Tuple, "(420i128,true,17,'a')");
        let result: Tuple = TupleType(vec![
            p_num!(I128),
            e_static!(crate::semantic::scope::static_types::StaticType::Primitive(
                PrimitiveType::Bool
            )),
            p_num!(I64),
            e_static!(crate::semantic::scope::static_types::StaticType::Primitive(
                PrimitiveType::Char
            )),
        ])
        .deserialize_from(&data)
        .expect("Deserialization should have succeeded");
        // assert_eq!(result.value, expr.value);
        for (expected, res) in expr.value.into_iter().zip(result.value) {
            match (expected, res) {
                (
                    Expression::Atomic(Atomic::Data(Data::Primitive(expected))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(res))),
                ) => {
                    assert_eq!(expected, res);
                }
                _ => assert!(false, "Expected boolean or u64"),
            }
        }
    }

    #[test]
    fn valid_struct() {
        let user_type = user_type_impl::Struct {
            id: "Point".to_string().into(),
            fields: {
                let mut res = Vec::new();
                res.push(("x".to_string().into(), p_num!(U64)));
                res.push(("y".to_string().into(), p_num!(I32)));
                res
            },
        };
        let (expr, data) = compile_expression_with_type!(
            Struct,
            r##"
        Point {
            x : 420u64,
            y : 69i32
        }
        "##,
            user_type
        );
        let result: Struct = user_type
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");
        for ((e_id, expected), (r_id, res)) in expr.fields.into_iter().zip(result.fields) {
            if e_id != r_id {
                assert!(false, "Expected matching field ids")
            }
            match (expected, res) {
                (
                    Expression::Atomic(Atomic::Data(Data::Primitive(expected))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(res))),
                ) => {
                    assert_eq!(expected, res);
                }
                _ => assert!(false, "Expected primitives"),
            }
        }
    }

    #[test]
    fn valid_struct_complex() {
        let user_type = user_type_impl::Struct {
            id: "Point".to_string().into(),
            fields: {
                let mut res = Vec::new();
                res.push(("x".to_string().into(), p_num!(I32)));
                res.push(("y".to_string().into(), p_num!(I64)));
                res
            },
        };
        let (expr, data) = compile_expression_with_type!(
            Struct,
            r##"
        Point {
            x : 420,
            y : 69
        }
        "##,
            user_type
        );
        let result: Struct = user_type
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");
        for ((e_id, expected), (r_id, res)) in expr.fields.into_iter().zip(result.fields) {
            if e_id != r_id {
                assert!(false, "Expected matching field ids")
            }
            match (expected, res) {
                (
                    Expression::Atomic(Atomic::Data(Data::Primitive(expected))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(res))),
                ) => {
                    assert_eq!(expected, res);
                }
                _ => assert!(false, "Expected primitives"),
            }
        }
    }

    #[test]
    fn valid_union() {
        let user_type = user_type_impl::Union {
            id: "Geo".to_string().into(),
            variants: {
                let mut res = Vec::new();
                res.push((
                    "Point".to_string().into(),
                    user_type_impl::Struct {
                        id: "Point".to_string().into(),
                        fields: vec![
                            ("x".to_string().into(), p_num!(I64)),
                            ("y".to_string().into(), p_num!(I64)),
                        ],
                    },
                ));
                res.push((
                    "Axe".to_string().into(),
                    user_type_impl::Struct {
                        id: "Axe".to_string().into(),
                        fields: {
                            let mut res = Vec::new();
                            res.push(("x".to_string().into(), p_num!(I64)));
                            res
                        },
                    },
                ));
                res
            },
        };
        let (expr, data) = compile_expression_with_type!(
            Union,
            r##"
            Geo::Point { x : 2, y : 8}
        "##,
            user_type
        );
        let result: Union = user_type
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");
        if expr.variant != result.variant {
            assert!(false, "Expected matching field ids")
        }
        for ((e_id, expected), (r_id, res)) in expr.fields.into_iter().zip(result.fields) {
            if e_id != r_id {
                assert!(false, "Expected matching field ids")
            }
            match (expected, res) {
                (
                    Expression::Atomic(Atomic::Data(Data::Primitive(expected))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(res))),
                ) => {
                    assert_eq!(expected, res);
                }
                _ => assert!(false, "Expected primitives"),
            }
        }
    }

    #[test]
    fn valid_enum() {
        let user_type = user_type_impl::Enum {
            id: "Geo".to_string().into(),
            values: {
                let mut res = Vec::new();
                res.push("Axe".to_string().into());
                res.push("Point".to_string().into());
                res.push("Space".to_string().into());
                res
            },
        };
        let (expr, data) = compile_expression_with_type!(
            Enum,
            r##"
            Geo::Point
        "##,
            user_type
        );
        let result: Enum =
            <user_type_impl::Enum as DeserializeFrom>::deserialize_from(&user_type, &data)
                .expect("Deserialization should have succeeded");
        assert_eq!(expr, result)
    }

    #[test]
    fn valid_slice() {
        let (expr, data) = compile_expression!(Slice, "[1,2,3]");
        let result: Slice = SliceType {
            size: 3,
            item_type: Box::new(p_num!(I64)),
        }
        .deserialize_from(&data)
        .expect("Deserialization should have succeeded");

        let result: Vec<Option<i64>> = result
            .value
            .iter()
            .map(|e| match e {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x {
                        Number::I64(n) => Some(*n),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect();
        assert_eq!(result, vec![Some(1), Some(2), Some(3)])
    }

    #[test]
    fn valid_vector() {
        // Parse the expression.
        let mut expr = Vector::parse("vec[1,2,3,4]".into())
            .expect("Parsing should have succeeded")
            .1;

        // Create a new block.
        let scope = Scope::new();
        // Perform semantic check.
        expr.resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions = CasmProgram::default();
        expr.gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);

        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        program.merge(instructions);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);
        let arr: [u8; 8] = data.try_into().expect("");
        let heap_address = u64::from_le_bytes(arr);

        let data = heap
            .read(heap_address as usize, 8 * 4 + 16)
            .expect("Heap Read should have succeeded");

        let result: Vector = VecType(Box::new(p_num!(I64)))
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");

        let result: Vec<Option<i64>> = result
            .value
            .iter()
            .map(|e| match e {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x {
                        Number::I64(n) => Some(*n),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect();
        assert_eq!(result, vec![Some(1), Some(2), Some(3), Some(4)])
    }

    #[test]
    fn valid_string() {
        let (expr, data) = compile_expression!(StrSlice, "\"Hello World\"");

        let result: StrSlice = <StrSliceType as DeserializeFrom>::deserialize_from(
            &&StrSliceType {
                size: "Hello World".len(),
            },
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "Hello World")
    }

    #[test]
    fn valid_string_complex() {
        let (expr, data) = compile_expression!(StrSlice, "\"你好世界\"");

        let result: StrSlice = <StrSliceType as DeserializeFrom>::deserialize_from(
            &StrSliceType {
                size: "你好世界".len(),
            },
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "你好世界")
    }

    #[test]
    fn valid_array_access() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let arr = [1,2,3,4];
            return arr[2]; 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 3));
    }

    #[test]
    fn valid_vec_access() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let arr = vec[1,2,3,4];
            return arr[2]; 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 3));
    }

    #[test]
    fn valid_str_slice_access() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let arr = "Hello World";
            return arr[2]; 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result =
            <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
                .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Char('l'));
    }

    #[test]
    fn valid_string_access() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let arr = string("Hello World");
            return arr[2]; 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result =
            <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
                .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Char('l'));
    }

    #[test]
    fn valid_tuple_access() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let tuple = (420,69);
            return tuple.1; 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 69));
    }

    #[test]
    fn valid_field_access() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            struct Point {
                x : i64,
                y : i64,
            }
            let point = Point {
                x : 420,
                y : 69,
            };
            return point.y; 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 69));
    }

    #[test]
    fn valid_empty_struct() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            struct Phantom {}
            let var = Phantom{};
            let copy = (var,50);
            return var;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        assert_eq!(data.len(), 0)
    }
    #[test]
    fn valid_closure() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let f = (x:u64) ->{ 
                let y:u64;
                let u:u64;
                let i:u64;
                let o:u64;
                let e:u64;
                
                return x+1;
            };
            return f(68); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 69));
    }
    #[test]
    fn valid_closure_with_stack_env() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let env:u64 = 31;

            let f = move (x:u64) -> {
                return env + x;
            };
            env = 0;
            return f(38); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 69));
    }

    #[test]
    fn valid_closure_with_heap_env() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let env : Vec<u64> = vec[2,5];
            let f = move (x:u64) -> {
                return env[1] + x;
            };
            env[1] = 31;
            return f(38); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 69));
    }

    #[test]
    fn valid_closure_rec() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let blob = 5;
            let rec f : fn(u64) -> u64 = (x:u64) -> {
                if x >= 5 {
                    return 5u64;
                }
                return f(x+1);
            };
            return f(0); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 5));
    }

    #[test]
    fn valid_closure_rec_with_env() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let env = 1u64;
            let rec f : dyn fn(u64) -> u64 = move (x:u64) -> {
                let y:u64;
                let z:u64;
                let e:u64;
                let t:u64;
                let u:u64;
                if x >= 5 {
                    return 5u64;
                }
                return f(x+env);
            };
            return f(0); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 5));
    }

    #[test]
    fn valid_addr_complex() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let arr = vec[1,2,3,4];
            let len = *(arr as &u64);
            return len; 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 4));
    }

    #[test]
    fn valid_ptr_access_complex() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let arr = vec[1,2,3,4];
            let first = *(((arr as &Any) as u64 + 16) as &u64);
            return first; 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 1));
    }

    #[test]
    fn valid_map_init() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let hmap : Map<u64,u64> = map {
                5:1,
                8:2,
                9:3,
                7:4,
                1:5,
            };
            let (value,_) = get(&hmap,9);
            return value;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 3));
    }

    #[test]
    fn valid_map_init_no_typedef() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let hmap = map {
                "test":1,
                "test1":2,
                "test11":3,
                "test111":4,
                "test1111":5,
            };
            let (value,_) = get(&hmap,"test11");
            return value;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 3));
    }
}
