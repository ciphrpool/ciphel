use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    rc::Rc,
};

use num_traits::{sign, ToBytes};

use crate::{
    ast::utils::strings::ID,
    semantic::{
        scope::{
            static_types::{NumberType, StaticType},
            type_traits::GetSubTypes,
            user_type_impl::UserType,
            ScopeApi,
        },
        AccessLevel, EType, Either, MutRc, SizeOf,
    },
    vm::{
        allocator::{align, heap::HEAP_ADDRESS_SIZE, stack::Offset, MemoryAddress},
        casm::{
            alloc::{Access, Alloc},
            branch::{Goto, Label},
            locate::Locate,
            memcopy::MemCopy,
            operation::{Addition, Mult, OpPrimitive, Operation, OperationKind},
            serialize::Serialized,
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{
    Address, Closure, Data, Enum, ExprScope, FieldAccess, ListAccess, Map, NumAccess, Primitive,
    PtrAccess, Slice, StrSlice, Struct, Tuple, Union, VarID, Variable, Vector,
};

impl<Scope: ScopeApi> GenerateCode<Scope> for Data<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
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
            Data::Unit => todo!(),
            Data::Map(value) => value.gencode(scope, instructions),
            Data::Struct(value) => value.gencode(scope, instructions),
            Data::Union(value) => value.gencode(scope, instructions),
            Data::Enum(value) => value.gencode(scope, instructions),
            Data::StrSlice(value) => value.gencode(scope, instructions),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Primitive {
    fn gencode(
        &self,
        _scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let casm = match self {
            Primitive::Number(data) => match data.get() {
                super::Number::U8(data) => Serialized {
                    data: data.to_le_bytes().to_vec(),
                },
                super::Number::U16(data) => Serialized {
                    data: data.to_le_bytes().to_vec(),
                },
                super::Number::U32(data) => Serialized {
                    data: data.to_le_bytes().to_vec(),
                },
                super::Number::U64(data) => Serialized {
                    data: data.to_le_bytes().to_vec(),
                },
                super::Number::U128(data) => Serialized {
                    data: data.to_le_bytes().to_vec(),
                },
                super::Number::I8(data) => Serialized {
                    data: data.to_le_bytes().to_vec(),
                },
                super::Number::I16(data) => Serialized {
                    data: data.to_le_bytes().to_vec(),
                },
                super::Number::I32(data) => Serialized {
                    data: data.to_le_bytes().to_vec(),
                },
                super::Number::I64(data) => Serialized {
                    data: data.to_le_bytes().to_vec(),
                },
                super::Number::I128(data) => Serialized {
                    data: data.to_le_bytes().to_vec(),
                },
                super::Number::F64(data) => Serialized {
                    data: data.to_le_bytes().to_vec(),
                },
                super::Number::Unresolved(_) => return Err(CodeGenerationError::UnresolvedError),
            },
            Primitive::Bool(data) => Serialized {
                data: [*data as u8].to_vec(),
            },
            Primitive::Char(data) => Serialized {
                data: [*data as u8].to_vec(),
            },
        };
        instructions.push(Casm::Serialize(casm));
        Ok(())
    }
}

// impl<
//         Scope: ScopeApi<
//             UserType = UserType,
//             StaticType = StaticType,
//             Var = Var,
//             Chan = Chan,
//             Event = Event,
//         >,
//     > Variable<Scope>
// {
//     fn get_offset(
//         &self,
//
//         address: usize,
//         scope: &MutRc<Scope>,
//         instructions: &MutRc<CasmProgram>,
//         signature: &EType,
//     ) -> Result<
//         (
//             (usize, usize),
//             EType,
//         ),
//         CodeGenerationError,
//     > {
//         match self {
//             Variable::Var(VarID(id)) => {
//                 let Either::User(struct_type) = signature else {
//                     return Err(CodeGenerationError::UnresolvedError);
//                 };
//                 let UserType::Struct(struct_type) = struct_type.as_ref() else {
//                     return Err(CodeGenerationError::UnresolvedError);
//                 };
//                 let mut offset = address;
//                 let mut var_size = None;
//                 let mut var_type = None;
//                 for (field_id, field) in &struct_type.fields {
//                     let field_size = field.size_of();
//                     if field_id == id {
//                         var_size = Some(field_size);
//                         var_type = Some(field);
//                         break;
//                     }
//                     offset += field_size;
//                 }
//                 let Some(var_size) = var_size else {
//                     return Err(CodeGenerationError::UnresolvedError);
//                 };
//                 let Some(var_type) = var_type else {
//                     return Err(CodeGenerationError::UnresolvedError);
//                 };

//                 {
//                     let mut borrowed = instructions.as_ref()
// .try_borrow_mut()
// .map_err(|_| CodeGenerationError::Default)?;
//                     instructions.push(Casm::Access(Access::Variable {
//                         address: MemoryAddress::Stack { offset },
//                         size: var_size,
//                     }))
//                 }

//                 Ok(((offset, var_size), var_type.clone()))
//             }
//             Variable::FieldAccess(FieldAccess { var, field }) => {
//                 let ((var_offset, var_size), var_type) =
//                     var.as_ref()
//                         .get_offset(offset, address, scope, instructions, signature)?;
//                 let ((field_offset, field_size), field_type) =
//                     field
//                         .as_ref()
//                         .get_offset(offset, var_offset, scope, instructions, &var_type)?;

//                 {
//                     let mut borrowed = instructions.as_ref()
// .try_borrow_mut()
// .map_err(|_| CodeGenerationError::Default)?;
//                     instructions.push(Casm::Access(Access::Variable {
//                         address: MemoryAddress::Stack {
//                             offset: field_offset,
//                         },
//                         size: field_size,
//                     }))
//                 }
//                 Ok(((field_offset, field_size), field_type))
//             }
//             Variable::NumAccess(NumAccess { var, index }) => {
//                 let ((var_offset, var_size), var_type) =
//                     var.as_ref()
//                         .get_offset(offset, address, scope, instructions, signature)?;
//                 let Either::Static(tuple_type) = signature else {
//                     return Err(CodeGenerationError::UnresolvedError);
//                 };
//                 let StaticType::Tuple(TupleType(tuple_type)) = tuple_type.as_ref() else {
//                     return Err(CodeGenerationError::UnresolvedError);
//                 };
//                 let mut offset = var_offset;
//                 let mut var_size = None;
//                 let mut var_type = None;
//                 for (idx, field) in tuple_type.iter().enumerate() {
//                     let field_size = field.size_of();
//                     if idx == *index {
//                         var_size = Some(field_size);
//                         var_type = Some(field);
//                         break;
//                     }
//                     offset += field_size;
//                 }
//                 let Some(var_size) = var_size else {
//                     return Err(CodeGenerationError::UnresolvedError);
//                 };
//                 let Some(var_type) = var_type else {
//                     return Err(CodeGenerationError::UnresolvedError);
//                 };

//                 {
//                     let mut borrowed = instructions.as_ref()
// .try_borrow_mut()
// .map_err(|_| CodeGenerationError::Default)?;
//                     instructions.push(Casm::Access(Access::Variable {
//                         address: MemoryAddress::Stack { offset },
//                         size: var_size,
//                     }))
//                 }
//                 Ok(((offset, var_size), var_type.clone()))
//             }
//             Variable::ListAccess(ListAccess { var, index }) => {
//                 let ((var_offset, var_size), var_type) =
//                     var.as_ref()
//                         .get_offset(offset, address, scope, instructions, signature)?;
//                 let (item_size, item_type) = {
//                     let Some(item_type) = <EType as GetSubTypes<
//                         Scope,
//                     >>::get_item(&var_type) else {
//                         return Err(CodeGenerationError::UnresolvedError);
//                     };
//                     (item_type.size_of(), item_type)
//                 };
//                 let Either::Static(list_type) = var_type else {
//                     return Err(CodeGenerationError::UnresolvedError);
//                 };
//                 match list_type.as_ref() {
//                     StaticType::Slice(_) => {
//                         let _ = index.gencode(scope, instructions, signature)?;
//                         {
//                             let mut borrowed = instructions.as_ref()
// .try_borrow_mut()
// .map_err(|_| CodeGenerationError::Default)?;
//                             instructions.push(Casm::Access(Access::List {
//                                 address: MemoryAddress::Stack {
//                                     offset: var_offset + 8,
//                                 },
//                                 size: item_size,
//                             }));
//                         }
//                     }
//                     StaticType::Vec(_) => {
//                         {
//                             let mut borrowed = instructions.as_ref()
// .try_borrow_mut()
// .map_err(|_| CodeGenerationError::Default)?;
//                             instructions.push(Casm::Access(Access::Variable {
//                                 address: MemoryAddress::Stack {
//                                     offset: var_offset + 16, /* LENGTH + CAPACITY */
//                                 },
//                                 size: var_size,
//                             }));
//                         }
//                         let _ = index.gencode(scope, instructions, signature)?;
//                         {
//                             let mut borrowed = instructions.as_ref()
// .try_borrow_mut()
// .map_err(|_| CodeGenerationError::Default)?;
//                             instructions.push(Casm::Access(Access::List {
//                                 address: MemoryAddress::Heap,
//                                 size: item_size,
//                             }));
//                         }
//                     }
//                     _ => {}
//                 }
//                 Ok(((var_offset, var_size), var_type.clone()))
//             }
//         }
//     }
// }

// impl<
//         Scope: ScopeApi<
//             UserType = UserType,
//             StaticType = StaticType,
//             Var = Var,
//             Chan = Chan,
//             Event = Event,
//         >,
//     > GenerateCode<Scope> for Variable<Scope>
// {
//
//     fn gencode(
//         &self,
//         scope: &MutRc<Scope>,
//         instructions: &MutRc<CasmProgram>,
//
//
//     ) -> Result<(), CodeGenerationError> {
//         match self {
//             Variable::Var(VarID(id)) => {
//                 let var = scope
//                     .borrow()
//                     .find_var(id)
//                     .map_err(|_| CodeGenerationError::UnresolvedError)?;

//                 let Some(address) = &var.as_ref().address else {
//                     return Err(CodeGenerationError::UnresolvedError);
//                 };
//                 let var_type = &var.as_ref().type_sig;
//                 let var_size = var_type.size_of();

//                 let mut borrowed = instructions.as_ref()
// .try_borrow_mut()
// .map_err(|_| CodeGenerationError::Default)?;
//                 instructions.push(Casm::Access(Access::Variable {
//                     address: MemoryAddress::Stack {
//                         offset: address.as_ref().get(),
//                     },
//                     size: var_size,
//                 }));

//                 Ok(())
//             }
//             Variable::FieldAccess(FieldAccess { var, field }) => {
//                 // let ((var_offset, var_size), var_type) =
//                 //     var.get_offset(offset, scope, instructions, signature)?;
//                 // let _ = field.get_offset(offset, scope, instructions, &var_type)?;
//                 todo!()
//             }
//             Variable::NumAccess(_) => todo!(),
//             Variable::ListAccess(_) => todo!(),
//         }
//     }
// }

impl<Scope: ScopeApi> GenerateCode<Scope> for Variable<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Variable::Var(VarID { id, metadata: _ }) => {
                let (var, address, level) = scope.as_ref().borrow().access_var(id)?;
                // dbg!((&var, &address, &level));
                let var_type = &var.as_ref().type_sig;
                let var_size = var_type.size_of();

                instructions.push(Casm::Access(Access::Static {
                    address: MemoryAddress::Stack {
                        offset: address,
                        level,
                    },
                    size: var_size,
                }));

                Ok(())
            }
            Variable::FieldAccess(FieldAccess {
                var,
                field,
                metadata: _,
            }) => {
                // Locate the variable
                let _ = var.locate(scope, instructions)?;
                // Access the field
                let Some(from_type) = var.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(offset) = from_type.get_field_offset(field.name()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    instructions.push(Casm::Serialize(Serialized {
                        data: (offset as u64).to_le_bytes().to_vec(),
                    }));
                    instructions.push(Casm::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        // result: OpPrimitive::Number(NumberType::U64),
                    }));
                }
                field.access_from(from_type, scope, instructions)
            }
            Variable::NumAccess(NumAccess {
                var,
                index,
                metadata,
            }) => {
                // Locate the variable
                let _ = var.locate(scope, instructions)?;

                // Access the field
                let Some(from_type) = var.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(offset) = from_type.get_inline_field_offset(*index) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(size) = metadata.signature().map(|sig| sig.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    instructions.push(Casm::Serialize(Serialized {
                        data: (offset as u64).to_le_bytes().to_vec(),
                    }));
                    instructions.push(Casm::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        // result: OpPrimitive::Number(NumberType::U64),
                    }));
                    instructions.push(Casm::Access(Access::Runtime { size: Some(size) }));
                }
                Ok(())
            }
            Variable::ListAccess(ListAccess {
                var,
                index,
                metadata,
            }) => {
                // Locate the variable
                let _ = var.locate(scope, instructions)?;

                if let Some(signature) = var.signature() {
                    match signature {
                        Either::Static(signature) => match signature.as_ref() {
                            StaticType::String(_) | StaticType::Vec(_) => {
                                instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
                                instructions.push(Casm::Serialize(Serialized {
                                    data: (16u64).to_le_bytes().to_vec(),
                                }));
                                instructions.push(Casm::Operation(Operation {
                                    kind: OperationKind::Addition(Addition {
                                        left: OpPrimitive::Number(NumberType::U64),
                                        right: OpPrimitive::Number(NumberType::U64),
                                    }),
                                }));
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }

                // Access the field
                let _ = index.gencode(scope, instructions)?;
                let Some(size) = metadata.signature().map(|sig| sig.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                instructions.push(Casm::Serialize(Serialized {
                    data: (size as u64).to_le_bytes().to_vec(),
                }));
                instructions.push(Casm::Operation(Operation {
                    kind: OperationKind::Mult(Mult {
                        left: OpPrimitive::Number(NumberType::U64),
                        right: OpPrimitive::Number(NumberType::U64),
                    }),
                }));
                instructions.push(Casm::Operation(Operation {
                    kind: OperationKind::Addition(Addition {
                        left: OpPrimitive::Number(NumberType::U64),
                        right: OpPrimitive::Number(NumberType::U64),
                    }),
                }));
                instructions.push(Casm::Access(Access::Runtime { size: Some(size) }));

                Ok(())
            }
        }
    }
}

impl<Scope: ScopeApi> Variable<Scope> {
    pub fn signature(&self) -> Option<EType> {
        match self {
            Variable::Var(VarID { id: _, metadata }) => metadata.signature(),
            Variable::FieldAccess(FieldAccess {
                var: _,
                field,
                metadata,
            }) => metadata.signature(),
            Variable::NumAccess(NumAccess {
                var: _,
                index: _,
                metadata,
            }) => metadata.signature(),
            Variable::ListAccess(ListAccess {
                var: _,
                index: _,
                metadata,
            }) => metadata.signature(),
        }
    }

    fn name(&self) -> &ID {
        match self {
            Variable::Var(VarID { id, metadata: _ }) => id,
            Variable::FieldAccess(FieldAccess {
                var,
                field: _,
                metadata: _,
            }) => var.name(),
            Variable::NumAccess(NumAccess {
                var,
                index: _,
                metadata: _,
            }) => var.name(),
            Variable::ListAccess(ListAccess {
                var,
                index: _,
                metadata: _,
            }) => var.name(),
        }
    }

    fn access_from(
        &self,
        from_type: EType,

        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Variable::Var(VarID { id: _, metadata }) => {
                let Some(size) = metadata.signature().map(|sig| sig.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                instructions.push(Casm::Access(Access::Runtime { size: Some(size) }));
                Ok(())
            }
            Variable::FieldAccess(FieldAccess {
                var,
                field,
                metadata: _,
            }) => {
                // Access the field
                let Some(from_type) = var.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(offset) = from_type.get_field_offset(field.name()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    instructions.push(Casm::Serialize(Serialized {
                        data: (offset as u64).to_le_bytes().to_vec(),
                    }));
                    instructions.push(Casm::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        // result: OpPrimitive::Number(NumberType::U64),
                    }));
                }
                field.access_from(from_type, scope, instructions)
            }
            Variable::NumAccess(NumAccess {
                var,
                index,
                metadata,
            }) => {
                let _ = var.locate_from(from_type, scope, instructions)?;
                let Some(from_type) = var.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(offset) = from_type.get_inline_field_offset(*index) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(size) = metadata.signature().map(|sig| sig.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    instructions.push(Casm::Serialize(Serialized {
                        data: (offset as u64).to_le_bytes().to_vec(),
                    }));
                    instructions.push(Casm::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        // result: OpPrimitive::Number(NumberType::U64),
                    }));
                    instructions.push(Casm::Access(Access::Runtime { size: Some(size) }));
                }
                Ok(())
            }
            Variable::ListAccess(ListAccess {
                var,
                index,
                metadata,
            }) => {
                let _ = var.locate_from(from_type, scope, instructions)?;
                if let Some(signature) = var.signature() {
                    match signature {
                        Either::Static(signature) => match signature.as_ref() {
                            StaticType::String(_) | StaticType::Vec(_) => {
                                instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
                                instructions.push(Casm::Serialize(Serialized {
                                    data: (16u64).to_le_bytes().to_vec(),
                                }));
                                instructions.push(Casm::Operation(Operation {
                                    kind: OperationKind::Addition(Addition {
                                        left: OpPrimitive::Number(NumberType::U64),
                                        right: OpPrimitive::Number(NumberType::U64),
                                    }),
                                }));
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                let _ = index.gencode(scope, instructions)?;
                let Some(size) = metadata.signature().map(|sig| sig.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };

                instructions.push(Casm::Serialize(Serialized {
                    data: (size as u64).to_le_bytes().to_vec(),
                }));
                instructions.push(Casm::Operation(Operation {
                    kind: OperationKind::Mult(Mult {
                        left: OpPrimitive::Number(NumberType::U64),
                        right: OpPrimitive::Number(NumberType::U64),
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                instructions.push(Casm::Operation(Operation {
                    kind: OperationKind::Addition(Addition {
                        left: OpPrimitive::Number(NumberType::U64),
                        right: OpPrimitive::Number(NumberType::U64),
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                instructions.push(Casm::Access(Access::Runtime { size: Some(size) }));

                Ok(())
            }
        }
    }

    pub fn locate(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Variable::Var(VarID { id, metadata: _ }) => {
                let (var, address, level) = scope.as_ref().borrow().access_var(id)?;

                let var_type = &var.as_ref().type_sig;
                let _var_size = var_type.size_of();

                instructions.push(Casm::Locate(Locate {
                    address: MemoryAddress::Stack {
                        offset: address,
                        level,
                    },
                }));

                Ok(())
            }
            Variable::FieldAccess(FieldAccess {
                var,
                field,
                metadata,
            }) => {
                let _ = var.locate(scope, instructions)?;
                // Locate the field
                let Some(from_type) = var.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(offset) = from_type.get_field_offset(field.name()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    instructions.push(Casm::Serialize(Serialized {
                        data: (offset as u64).to_le_bytes().to_vec(),
                    }));
                    instructions.push(Casm::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        // result: OpPrimitive::Number(NumberType::U64),
                    }));
                }
                field.locate_from(from_type, scope, instructions)
            }
            Variable::NumAccess(NumAccess {
                var,
                index,
                metadata: _,
            }) => {
                let _ = var.locate(scope, instructions)?;
                let Some(from_type) = var.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(offset) = from_type.get_inline_field_offset(*index) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    instructions.push(Casm::Serialize(Serialized {
                        data: (offset as u64).to_le_bytes().to_vec(),
                    }));
                    instructions.push(Casm::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        // result: OpPrimitive::Number(NumberType::U64),
                    }));
                }
                Ok(())
            }
            Variable::ListAccess(ListAccess {
                var,
                index,
                metadata,
            }) => {
                let _ = var.locate(scope, instructions)?;
                if let Some(signature) = var.signature() {
                    match signature {
                        Either::Static(signature) => match signature.as_ref() {
                            StaticType::String(_) | StaticType::Vec(_) => {
                                instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
                                instructions.push(Casm::Serialize(Serialized {
                                    data: (16u64).to_le_bytes().to_vec(),
                                }));
                                instructions.push(Casm::Operation(Operation {
                                    kind: OperationKind::Addition(Addition {
                                        left: OpPrimitive::Number(NumberType::U64),
                                        right: OpPrimitive::Number(NumberType::U64),
                                    }),
                                }));
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                let _ = index.gencode(scope, instructions)?;
                let Some(item_size) = metadata.signature().map(|sig| sig.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    instructions.push(Casm::Serialize(Serialized {
                        data: (item_size as u64).to_le_bytes().to_vec(),
                    }));
                    instructions.push(Casm::Operation(Operation {
                        kind: OperationKind::Mult(Mult {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        // result: OpPrimitive::Number(NumberType::U64),
                    }));
                    instructions.push(Casm::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        // result: OpPrimitive::Number(NumberType::U64),
                    }));
                }
                Ok(())
            }
        }
    }

    fn locate_from(
        &self,
        from_type: EType,

        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Variable::Var(VarID { id: _, metadata }) => Ok(()),
            Variable::FieldAccess(FieldAccess {
                var,
                field,
                metadata: _,
            }) => {
                // Access the field
                let Some(from_type) = var.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(offset) = from_type.get_field_offset(field.name()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    instructions.push(Casm::Serialize(Serialized {
                        data: (offset as u64).to_le_bytes().to_vec(),
                    }));
                    instructions.push(Casm::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        // result: OpPrimitive::Number(NumberType::U64),
                    }));
                }
                field.locate_from(from_type, scope, instructions)
            }
            Variable::NumAccess(NumAccess {
                var,
                index,
                metadata,
            }) => {
                let _ = var.locate_from(from_type, scope, instructions)?;
                let Some(from_type) = var.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(offset) = from_type.get_inline_field_offset(*index) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    instructions.push(Casm::Serialize(Serialized {
                        data: (offset as u64).to_le_bytes().to_vec(),
                    }));
                    instructions.push(Casm::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        // result: OpPrimitive::Number(NumberType::U64),
                    }));
                }
                Ok(())
            }
            Variable::ListAccess(ListAccess {
                var,
                index,
                metadata,
            }) => {
                let _ = var.locate_from(from_type, scope, instructions)?;
                if let Some(signature) = var.signature() {
                    match signature {
                        Either::Static(signature) => match signature.as_ref() {
                            StaticType::String(_) | StaticType::Vec(_) => {
                                instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
                                instructions.push(Casm::Serialize(Serialized {
                                    data: (16u64).to_le_bytes().to_vec(),
                                }));
                                instructions.push(Casm::Operation(Operation {
                                    kind: OperationKind::Addition(Addition {
                                        left: OpPrimitive::Number(NumberType::U64),
                                        right: OpPrimitive::Number(NumberType::U64),
                                    }),
                                }));
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }

                let _ = index.gencode(scope, instructions)?;
                let Some(item_size) = metadata.signature().map(|sig| sig.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    instructions.push(Casm::Serialize(Serialized {
                        data: (item_size as u64).to_le_bytes().to_vec(),
                    }));
                    instructions.push(Casm::Operation(Operation {
                        kind: OperationKind::Mult(Mult {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        // result: OpPrimitive::Number(NumberType::U64),
                    }));
                    instructions.push(Casm::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        // result: OpPrimitive::Number(NumberType::U64),
                    }));
                }
                Ok(())
            }
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Slice<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
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
        // // Push the size of the slice
        // let bytes = (self.value.len() as u64).to_le_bytes().as_slice().to_vec();
        // offset += bytes.len();
        // instructions.push(Casm::Serialize(Serialized { data: bytes }));

        for element in &self.value {
            let _ = element.gencode(scope, instructions)?;
        }

        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for StrSlice {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let str_bytes = self.value.clone().into_bytes();
        let bytes_len = str_bytes.len();
        instructions.push(Casm::Serialize(Serialized { data: str_bytes }));
        Ok(())
        // let mut borrowed = instructions
        //     .as_ref()
        //     .try_borrow_mut()
        //     .map_err(|_| CodeGenerationError::Default)?;

        // // Alloc and push heap address on stack
        // instructions.push(Casm::Alloc(Alloc::Heap {
        //     size: align(self.value.len()) + 16,
        // }));

        // let len_bytes = (self.value.len() as u64).to_le_bytes().as_slice().to_vec();
        // let cap_bytes = (align(self.value.len()) as u64)
        //     .to_le_bytes()
        //     .as_slice()
        //     .to_vec();

        // // Push Length on stack
        // instructions.push(Casm::Serialize(Serialized { data: len_bytes }));
        // // Push Capacity on stack
        // instructions.push(Casm::Serialize(Serialized { data: cap_bytes }));
        // let str_bytes = self.value.clone().into_bytes();
        // let bytes_len = str_bytes.len();
        // instructions.push(Casm::Serialize(Serialized { data: str_bytes }));

        // drop(borrowed);
        // // Copy data on stack to heap at address
        // let mut borrowed = instructions
        //     .as_ref()
        //     .try_borrow_mut()
        //     .map_err(|_| CodeGenerationError::Default)?;
        // // Copy heap address on top of the stack
        // instructions.push(Casm::Access(Access::Static {
        //     address: MemoryAddress::Stack {
        //         offset: Offset::ST(-((bytes_len + 16 + 8) as isize)),
        //         level: AccessLevel::Direct,
        //     },
        //     size: 8,
        // }));

        // // Take the address on the top of the stack
        // // and copy the data on the stack in the heap at given address and given offset
        // // ( removing the data from the stack )
        // instructions.push(Casm::MemCopy(MemCopy::TakeToHeap {
        //     //offset: vec_stack_address + 8,
        //     size: self.value.len() + 16,
        // }));

        // Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Vector<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
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

        let len_bytes = (self.length as u64).to_le_bytes().as_slice().to_vec();
        let cap_bytes = (self.capacity as u64).to_le_bytes().as_slice().to_vec();

        // Push Length on stack
        instructions.push(Casm::Serialize(Serialized { data: len_bytes }));
        // Push Capacity on stack
        instructions.push(Casm::Serialize(Serialized { data: cap_bytes }));

        for element in &self.value {
            let _ = element.gencode(scope, instructions)?;
        }

        // Copy data on stack to heap at address

        // Alloc and push heap address on stack
        instructions.push(Casm::Alloc(Alloc::Heap {
            size: item_size * self.capacity + 16,
        }));
        // Take the address on the top of the stack
        // and copy the data on the stack in the heap at given address and given offset
        // ( removing the data from the stack )
        instructions.push(Casm::MemCopy(MemCopy::TakeToHeap {
            //offset: vec_stack_address + 8,
            size: item_size * self.value.len() + 16,
        }));

        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Tuple<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(signature) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        for (idx, element) in self.value.iter().enumerate() {
            let item_size = {
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

impl<Scope: ScopeApi> GenerateCode<Scope> for ExprScope<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            ExprScope::Scope(value) => value.gencode(scope, instructions),
            ExprScope::Expr(value) => value.gencode(scope, instructions),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Closure<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let end_closure = Label::gen();

        instructions.push(Casm::Goto(Goto {
            label: Some(end_closure),
        }));

        let closure_label = instructions.push_label("fn_closure".into());
        let _ = self.scope.gencode(scope, instructions);
        instructions.push_label_id(end_closure, "end_closure".into());

        instructions.push(Casm::MemCopy(MemCopy::LabelOffset(closure_label)));

        if self.closed {
            /* Load env and store in the heap */
            let mut alloc_size = 16;
            let mut env_size = 0;

            let binding = self
                .scope
                .scope()
                .map_err(|_| CodeGenerationError::UnresolvedError)?;
            let inner_scope = binding.as_ref().borrow();

            for var in inner_scope.env_vars() {
                let var_type = &var.as_ref().type_sig;
                let var_size = var_type.size_of();
                alloc_size += var_size;
                env_size += var_size;
            }

            // Load Env Size
            instructions.push(Casm::Serialize(Serialized {
                data: env_size.to_le_bytes().to_vec(),
            }));
            let outer_scope = scope.as_ref().borrow();
            // Load Env variables
            for var in inner_scope.env_vars() {
                let (var, address, level) = outer_scope.access_var(&var.id)?;
                let var_type = &var.as_ref().type_sig;
                let var_size = var_type.size_of();
                instructions.push(Casm::Access(Access::Static {
                    address: MemoryAddress::Stack {
                        offset: address,
                        level: level,
                    },
                    size: var_size,
                }));
            }
            instructions.push(Casm::Alloc(Alloc::Heap { size: alloc_size }));
            instructions.push(Casm::MemCopy(MemCopy::TakeToHeap { size: alloc_size }));
        }

        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Address<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        self.value.locate(scope, instructions)
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for PtrAccess<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Struct<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
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
                instructions.push(Casm::Serialize(Serialized {
                    data: vec![0; padding],
                }));
            }
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Union<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
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
        let Some((idx, struct_type)) = union_type
            .variants
            .iter()
            .enumerate()
            .find(|(_idx, (id, _))| id == &self.variant)
            .map(|(idx, (_, st))| (idx, st))
        else {
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
                instructions.push(Casm::Serialize(Serialized {
                    data: vec![0; padding],
                }));
            }
        }

        instructions.push(Casm::Serialize(Serialized {
            data: (idx as u64).to_le_bytes().to_vec(),
        }));

        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Enum {
    fn gencode(
        &self,
        _scope: &MutRc<Scope>,
        instructions: &CasmProgram,
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

        instructions.push(Casm::Serialize(Serialized {
            data: (index as u64).to_le_bytes().to_vec(),
        }));
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Map<Scope> {
    fn gencode(
        &self,
        _scope: &MutRc<Scope>,
        _instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, mem};

    use num_traits::Zero;

    use super::*;
    use crate::{
        ast::{
            expressions::{
                data::{Data, Number},
                Atomic, Expression,
            },
            statements::{self, Statement},
            TryParse,
        },
        semantic::{
            scope::{
                scope_impl::Scope,
                static_types::{
                    PrimitiveType, SliceType, StrSliceType, StringType, TupleType, VecType,
                },
                user_type_impl, ScopeApi,
            },
            Resolve, TypeOf,
        },
        vm::{
            allocator::Memory,
            casm::branch::BranchTable,
            vm::{DeserializeFrom, Executable, Runtime},
        },
    };
    #[macro_export]
    macro_rules! assert_number {
        ($expr:ident,$data:ident,$num_type:ident) => {
            let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
                &PrimitiveType::Number(NumberType::$num_type),
                &$data,
            )
            .expect("Deserialization should have succeeded");

            // Assert and return the result.
            assert_eq!(result, $expr);
        };
    }

    #[macro_export]
    macro_rules! compile_expression {
        ($expr_type:ident,$expr_str:expr) => {{
            // Parse the expression.
            let expr = $expr_type::parse($expr_str.into())
                .expect("Parsing should have succeeded")
                .1;

            // Create a new scope.
            let scope = Scope::new();
            // Perform semantic check.
            expr.resolve(&scope, &None, &())
                .expect("Semantic resolution should have succeeded");

            // Code generation.
            let instructions = CasmProgram::default();
            expr.gencode(&scope, &instructions)
                .expect("Code generation should have succeeded");
            assert!(instructions.len() > 0);

            // Execute the instructions.
            let mut runtime = Runtime::new();
            let tid = runtime
                .spawn()
                .expect("Thread spawning should have succeeded");
            let thread = runtime.get(tid).expect("Thread should exist");
            thread.push_instr(instructions);
            thread.run().expect("Execution should have succeeded");
            let memory = &thread.memory();
            let data = clear_stack!(memory);
            (expr, data)
        }};
    }

    #[macro_export]
    macro_rules! compile_expression_with_type {
        ($expr_type:ident,$expr_str:expr,$user_type:ident) => {{
            // Parse the expression.
            let expr = $expr_type::parse($expr_str.into())
                .expect("Parsing should have succeeded")
                .1;

            // Create a new scope.
            let scope = Scope::new();
            let _ = scope
                .as_ref()
                .borrow_mut()
                .register_type(
                    &$user_type.id.clone(),
                    UserType::$expr_type($user_type.clone().into()),
                )
                .expect("Type registering should have succeeded");
            // Perform semantic check.
            expr.resolve(&scope, &None, &())
                .expect("Semantic resolution should have succeeded");

            // Code generation.
            let instructions = CasmProgram::default();
            expr.gencode(&scope, &instructions)
                .expect("Code generation should have succeeded");
            assert!(instructions.len() > 0);

            // Execute the instructions.
            let mut runtime = Runtime::new();
            let tid = runtime
                .spawn()
                .expect("Thread spawning should have succeeded");
            let thread = runtime.get(tid).expect("Thread should exist");
            thread.push_instr(instructions);
            thread.run().expect("Execution should have succeeded");
            let memory = &thread.memory();
            let data = clear_stack!(memory);
            (expr, data)
        }};
    }

    #[macro_export]
    macro_rules! clear_stack {
        ($memory:ident) => {{
            use num_traits::Zero;
            let top = $memory.stack.top();
            let data = $memory.stack.pop(top).expect("Read should have succeeded");
            assert!($memory.stack.top().is_zero());
            data
        }};
    }

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
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Char,
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, expr);
        let (expr, data) = compile_expression!(Primitive, "true");
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Bool,
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, expr);

        let (expr, data) = compile_expression!(Primitive, "420.69");
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, expr);
    }

    #[test]
    fn valid_tuple() {
        let (expr, data) = compile_expression!(Tuple, "(true,17)");
        let result: Tuple<Scope> = TupleType(vec![
            Either::Static(StaticType::Primitive(PrimitiveType::Bool).into()),
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into()),
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
        let result: Tuple<Scope> = TupleType(vec![
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::I128)).into()),
            Either::Static(StaticType::Primitive(PrimitiveType::Bool).into()),
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into()),
            Either::Static(StaticType::Primitive(PrimitiveType::Char).into()),
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
            id: "Point".into(),
            fields: {
                let mut res = Vec::new();
                res.push((
                    "x".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    ),
                ));
                res.push((
                    "y".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I32)).into(),
                    ),
                ));
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
        let result: Struct<Scope> = user_type
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
            id: "Point".into(),
            fields: {
                let mut res = Vec::new();
                res.push((
                    "x".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I32)).into(),
                    ),
                ));
                res.push((
                    "y".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                ));
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
        let result: Struct<Scope> = user_type
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
            id: "Geo".into(),
            variants: {
                let mut res = Vec::new();
                res.push((
                    "Point".into(),
                    user_type_impl::Struct {
                        id: "Point".into(),
                        fields: vec![
                            (
                                "x".into(),
                                Either::Static(
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
                                        .into(),
                                ),
                            ),
                            (
                                "y".into(),
                                Either::Static(
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
                                        .into(),
                                ),
                            ),
                        ],
                    },
                ));
                res.push((
                    "Axe".into(),
                    user_type_impl::Struct {
                        id: "Axe".into(),
                        fields: {
                            let mut res = Vec::new();
                            res.push((
                                "x".into(),
                                Either::Static(
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
                                        .into(),
                                ),
                            ));
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
        let result: Union<Scope> = user_type
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
            id: "Geo".into(),
            values: {
                let mut res = Vec::new();
                res.push("Axe".into());
                res.push("Point".into());
                res.push("Space".into());
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
            <user_type_impl::Enum as DeserializeFrom<Scope>>::deserialize_from(&user_type, &data)
                .expect("Deserialization should have succeeded");
        assert_eq!(expr, result)
    }

    #[test]
    fn valid_slice() {
        let (expr, data) = compile_expression!(Slice, "[1,2,3]");
        let result: Slice<Scope> = SliceType {
            size: 3,
            item_type: Box::new(Either::Static(
                StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
            )),
        }
        .deserialize_from(&data)
        .expect("Deserialization should have succeeded");

        let result: Vec<Option<i64>> = result
            .value
            .iter()
            .map(|e| match e {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x.get() {
                        Number::I64(n) => Some(n),
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
        let expr = Vector::parse("vec[1,2,3,4]".into())
            .expect("Parsing should have succeeded")
            .1;

        // Create a new scope.
        let scope = Scope::new();
        // Perform semantic check.
        expr.resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        expr.gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);

        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);
        let arr: [u8; 8] = data.try_into().expect("");
        let heap_address = u64::from_le_bytes(arr);

        let data = memory
            .heap
            .read(heap_address as usize, 8 * 4 + 16)
            .expect("Heap Read should have succeeded");

        let result: Vector<Scope> = VecType(Box::new(Either::Static(
            StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
        )))
        .deserialize_from(&data)
        .expect("Deserialization should have succeeded");

        let result: Vec<Option<i64>> = result
            .value
            .iter()
            .map(|e| match e {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x.get() {
                        Number::I64(n) => Some(n),
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

        let result: StrSlice = <StrSliceType as DeserializeFrom<Scope>>::deserialize_from(
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

        let result: StrSlice = <StrSliceType as DeserializeFrom<Scope>>::deserialize_from(
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
        let statement = Statement::parse(
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

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(3))));
    }

    #[test]
    fn valid_vec_access() {
        let statement = Statement::parse(
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

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(3))));
    }

    #[test]
    fn valid_str_slice_access() {
        let statement = Statement::parse(
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

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Char,
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Char('l'));
    }

    #[test]
    fn valid_string_access() {
        let statement = Statement::parse(
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

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Char,
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Char('l'));
    }

    #[test]
    fn valid_tuple_access() {
        let statement = Statement::parse(
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

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(69))));
    }

    #[test]
    fn valid_field_access() {
        let statement = Statement::parse(
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

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(69))));
    }

    #[test]
    fn valid_closure() {
        let statement = Statement::parse(
            r##"
        let x = {
            let f = (x:u64) -> x+1u64;
            return f(68); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::U64(69))));
    }
    #[test]
    fn valid_closure_with_stack_env() {
        let statement = Statement::parse(
            r##"
        let x = {
            let env:u64 = 31;

            let f = move (x:u64) -> {
                return env + x;
            };
            env = 50;
            return f(38); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::U64(69))));
    }

    #[test]
    fn valid_closure_with_heap_env() {
        let statement = Statement::parse(
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

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::U64(69))));
    }

    #[test]
    fn valid_closure_rec() {
        let statement = Statement::parse(
            r##"
        let x = {
            let blob = 5;
            let rec f : fn(u64) -> u64 = (x:u64) -> {
                if x >= 5u64 {
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

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::U64(5))));
    }

    #[test]
    fn valid_closure_rec_with_env() {
        let statement = Statement::parse(
            r##"
        let x = {
            let env = 1u64;
            let rec f : dyn fn(u64) -> u64 = move (x:u64) -> {
                if x >= 5u64 {
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

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::U64(5))));
    }
}
