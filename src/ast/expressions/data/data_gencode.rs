use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

use num_traits::ToBytes;

use crate::{
    ast::utils::strings::ID,
    semantic::{
        scope::{
            static_types::{NumberType, StaticType},
            type_traits::GetSubTypes,
            user_type_impl::UserType,
            ScopeApi,
        },
        Either, SizeOf,
    },
    vm::{
        allocator::{align, heap::HEAP_ADDRESS_SIZE, MemoryAddress},
        strips::{
            access::Access,
            alloc::Alloc,
            locate::Locate,
            memcopy::MemCopy,
            operation::{Addition, OpPrimitive, Operation, OperationKind},
            serialize::Serialized,
            Strip,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{
    Address, Channel, Closure, Data, Enum, FieldAccess, ListAccess, Map, NumAccess, Primitive,
    PtrAccess, Slice, Struct, Tuple, Union, VarID, Variable, Vector,
};

impl<Scope: ScopeApi> GenerateCode<Scope> for Data<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Data::Primitive(value) => value.gencode(scope, instructions, offset),
            Data::Slice(value) => value.gencode(scope, instructions, offset),
            Data::Vec(value) => value.gencode(scope, instructions, offset),
            Data::Closure(value) => value.gencode(scope, instructions, offset),
            Data::Chan(value) => value.gencode(scope, instructions, offset),
            Data::Tuple(value) => value.gencode(scope, instructions, offset),
            Data::Address(value) => value.gencode(scope, instructions, offset),
            Data::PtrAccess(value) => value.gencode(scope, instructions, offset),
            Data::Variable(value) => value.gencode(scope, instructions, offset),
            Data::Unit => todo!(),
            Data::Map(value) => value.gencode(scope, instructions, offset),
            Data::Struct(value) => value.gencode(scope, instructions, offset),
            Data::Union(value) => value.gencode(scope, instructions, offset),
            Data::Enum(value) => value.gencode(scope, instructions, offset),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Primitive {
    fn gencode(
        &self,
        _scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        _offset: usize,
    ) -> Result<(), CodeGenerationError> {
        let mut borrowed = instructions.as_ref().borrow_mut();
        let strip = match self {
            Primitive::Number(data) => match data {
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
            },
            Primitive::Float(data) => Serialized {
                data: data.to_le_bytes().to_vec(),
            },
            Primitive::Bool(data) => Serialized {
                data: [*data as u8].to_vec(),
            },
            Primitive::Char(data) => Serialized {
                data: [*data as u8].to_vec(),
            },
        };
        borrowed.push(Strip::Serialize(strip));
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
//         start_offset: usize,
//         address: usize,
//         scope: &Rc<RefCell<Scope>>,
//         instructions: &Rc<RefCell<Vec<Strip>>>,
//         signature: &Either<UserType, StaticType>,
//     ) -> Result<
//         (
//             (usize, usize),
//             Either<UserType, StaticType>,
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
//                     let mut borrowed = instructions.as_ref().borrow_mut();
//                     borrowed.push(Strip::Access(Access::Variable {
//                         address: MemoryAddress::Stack { offset },
//                         size: var_size,
//                     }))
//                 }

//                 Ok(((offset, var_size), var_type.clone()))
//             }
//             Variable::FieldAccess(FieldAccess { var, field }) => {
//                 let ((var_offset, var_size), var_type) =
//                     var.as_ref()
//                         .get_offset(start_offset, address, scope, instructions, signature)?;
//                 let ((field_offset, field_size), field_type) =
//                     field
//                         .as_ref()
//                         .get_offset(start_offset, var_offset, scope, instructions, &var_type)?;

//                 {
//                     let mut borrowed = instructions.as_ref().borrow_mut();
//                     borrowed.push(Strip::Access(Access::Variable {
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
//                         .get_offset(start_offset, address, scope, instructions, signature)?;
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
//                     let mut borrowed = instructions.as_ref().borrow_mut();
//                     borrowed.push(Strip::Access(Access::Variable {
//                         address: MemoryAddress::Stack { offset },
//                         size: var_size,
//                     }))
//                 }
//                 Ok(((offset, var_size), var_type.clone()))
//             }
//             Variable::ListAccess(ListAccess { var, index }) => {
//                 let ((var_offset, var_size), var_type) =
//                     var.as_ref()
//                         .get_offset(start_offset, address, scope, instructions, signature)?;
//                 let (item_size, item_type) = {
//                     let Some(item_type) = <Either<UserType, StaticType> as GetSubTypes<
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
//                         let _ = index.gencode(scope, instructions, start_offset, signature)?;
//                         {
//                             let mut borrowed = instructions.as_ref().borrow_mut();
//                             borrowed.push(Strip::Access(Access::List {
//                                 address: MemoryAddress::Stack {
//                                     offset: var_offset + 8,
//                                 },
//                                 size: item_size,
//                             }));
//                         }
//                     }
//                     StaticType::Vec(_) => {
//                         {
//                             let mut borrowed = instructions.as_ref().borrow_mut();
//                             borrowed.push(Strip::Access(Access::Variable {
//                                 address: MemoryAddress::Stack {
//                                     offset: var_offset + 16, /* LENGTH + CAPACITY */
//                                 },
//                                 size: var_size,
//                             }));
//                         }
//                         let _ = index.gencode(scope, instructions, start_offset, signature)?;
//                         {
//                             let mut borrowed = instructions.as_ref().borrow_mut();
//                             borrowed.push(Strip::Access(Access::List {
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
//         scope: &Rc<RefCell<Scope>>,
//         instructions: &Rc<RefCell<Vec<Strip>>>,
//         offset: usize,
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

//                 let mut borrowed = instructions.as_ref().borrow_mut();
//                 borrowed.push(Strip::Access(Access::Variable {
//                     address: MemoryAddress::Stack {
//                         offset: address.as_ref().get(),
//                     },
//                     size: var_size,
//                 }));

//                 Ok(())
//             }
//             Variable::FieldAccess(FieldAccess { var, field }) => {
//                 // let ((var_offset, var_size), var_type) =
//                 //     var.get_offset(offset, offset, scope, instructions, signature)?;
//                 // let _ = field.get_offset(offset, offset, scope, instructions, &var_type)?;
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
        scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Variable::Var(VarID { id, metadata: _ }) => {
                let var = scope
                    .borrow()
                    .find_var(id)
                    .map_err(|_| CodeGenerationError::UnresolvedError)?;

                let Some(address) = &var.as_ref().address else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let var_type = &var.as_ref().type_sig;
                let var_size = var_type.size_of();

                let mut borrowed = instructions.as_ref().borrow_mut();
                borrowed.push(Strip::Access(Access::Static {
                    address: MemoryAddress::Stack {
                        offset: address.as_ref().get(),
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
                let _ = var.locate(scope, instructions, offset)?;
                // Access the field
                let Some(from_type) = var.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(offset) = from_type.get_field_offset(field.name()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    let mut borrowed = instructions.as_ref().borrow_mut();
                    borrowed.push(Strip::Serialize(Serialized {
                        data: (offset as u64).to_le_bytes().to_vec(),
                    }));
                    borrowed.push(Strip::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        result: OpPrimitive::Number(NumberType::U64),
                    }));
                }
                field.access_from(from_type, offset, scope, instructions, offset + 8)
            }
            Variable::NumAccess(NumAccess {
                var,
                index,
                metadata,
            }) => {
                // Locate the variable
                let _ = var.locate(scope, instructions, offset)?;

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
                    let mut borrowed = instructions.as_ref().borrow_mut();
                    borrowed.push(Strip::Serialize(Serialized {
                        data: (offset as u64).to_le_bytes().to_vec(),
                    }));
                    borrowed.push(Strip::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        result: OpPrimitive::Number(NumberType::U64),
                    }));
                    borrowed.push(Strip::Access(Access::Runtime { size }));
                }
                Ok(())
            }
            Variable::ListAccess(ListAccess {
                var,
                index,
                metadata,
            }) => {
                // Locate the variable
                let _ = var.locate(scope, instructions, offset)?;

                // Access the field
                let _ = index.gencode(scope, instructions, offset + 8)?;
                let Some(size) = metadata.signature().map(|sig| sig.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    let mut borrowed = instructions.as_ref().borrow_mut();
                    borrowed.push(Strip::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        result: OpPrimitive::Number(NumberType::U64),
                    }));
                    borrowed.push(Strip::Access(Access::Runtime { size }));
                }
                Ok(())
            }
        }
    }
}

impl<Scope: ScopeApi> Variable<Scope> {
    fn signature(&self) -> Option<Either<UserType, StaticType>> {
        match self {
            Variable::Var(VarID { id: _, metadata }) => metadata.signature(),
            Variable::FieldAccess(FieldAccess {
                var: _,
                field: _,
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
        _from_type: Either<UserType, StaticType>,
        _offset: usize,
        scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        start_offset: usize,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Variable::Var(VarID { id: _, metadata }) => {
                let mut borrowed = instructions.as_ref().borrow_mut();
                let Some(size) = metadata.signature().map(|sig| sig.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                borrowed.push(Strip::Access(Access::Runtime { size }));
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
                    let mut borrowed = instructions.as_ref().borrow_mut();
                    borrowed.push(Strip::Serialize(Serialized {
                        data: (offset as u64).to_le_bytes().to_vec(),
                    }));
                    borrowed.push(Strip::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        result: OpPrimitive::Number(NumberType::U64),
                    }));
                }
                field.access_from(from_type, offset, scope, instructions, start_offset)
            }
            Variable::NumAccess(NumAccess {
                var,
                index,
                metadata,
            }) => {
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
                    let mut borrowed = instructions.as_ref().borrow_mut();
                    borrowed.push(Strip::Serialize(Serialized {
                        data: (offset as u64).to_le_bytes().to_vec(),
                    }));
                    borrowed.push(Strip::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        result: OpPrimitive::Number(NumberType::U64),
                    }));
                    borrowed.push(Strip::Access(Access::Runtime { size }));
                }
                Ok(())
            }
            Variable::ListAccess(ListAccess {
                var: _,
                index,
                metadata,
            }) => {
                let _ = index.gencode(scope, instructions, start_offset)?;
                let Some(size) = metadata.signature().map(|sig| sig.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    let mut borrowed = instructions.as_ref().borrow_mut();
                    borrowed.push(Strip::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        result: OpPrimitive::Number(NumberType::U64),
                    }));
                    borrowed.push(Strip::Access(Access::Runtime { size }));
                }
                Ok(())
            }
        }
    }

    fn locate(
        &self,
        scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        start_offset: usize,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Variable::Var(VarID { id, metadata: _ }) => {
                let var = scope
                    .borrow()
                    .find_var(id)
                    .map_err(|_| CodeGenerationError::UnresolvedError)?;

                let Some(address) = &var.as_ref().address else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let var_type = &var.as_ref().type_sig;
                let _var_size = var_type.size_of();

                let mut borrowed = instructions.as_ref().borrow_mut();
                borrowed.push(Strip::Locate(Locate {
                    address: MemoryAddress::Stack {
                        offset: address.as_ref().get(),
                    },
                }));

                Ok(())
            }
            Variable::FieldAccess(FieldAccess {
                var: _,
                field: _,
                metadata: _,
            }) => unreachable!(),
            Variable::NumAccess(NumAccess {
                var,
                index,
                metadata: _,
            }) => {
                let _ = var.locate(scope, instructions, start_offset)?;
                let Some(from_type) = var.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(offset) = from_type.get_inline_field_offset(*index) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    let mut borrowed = instructions.as_ref().borrow_mut();
                    borrowed.push(Strip::Serialize(Serialized {
                        data: (offset as u64).to_le_bytes().to_vec(),
                    }));
                    borrowed.push(Strip::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        result: OpPrimitive::Number(NumberType::U64),
                    }));
                }
                Ok(())
            }
            Variable::ListAccess(ListAccess {
                var,
                index,
                metadata: _,
            }) => {
                let _ = var.locate(scope, instructions, start_offset)?;
                let _ = index.gencode(scope, instructions, start_offset)?;
                {
                    let mut borrowed = instructions.as_ref().borrow_mut();
                    borrowed.push(Strip::Operation(Operation {
                        kind: OperationKind::Addition(Addition {
                            left: OpPrimitive::Number(NumberType::U64),
                            right: OpPrimitive::Number(NumberType::U64),
                        }),
                        result: OpPrimitive::Number(NumberType::U64),
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
        scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        let mut borrowed = instructions.as_ref().borrow_mut();

        match self {
            Slice::String { value, .. } => {
                let mut bytes = (value.len() as u64).to_le_bytes().as_slice().to_vec();
                bytes.extend_from_slice(value.as_bytes());
                borrowed.push(Strip::Serialize(Serialized { data: bytes }));
            }
            Slice::List {
                value: data,
                metadata,
            } => {
                let Some(signature) = metadata.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let bytes = (data.len() as u64).to_le_bytes().as_slice().to_vec();

                let item_size = {
                    let Some(item_type) = signature.get_item() else {
                        return Err(CodeGenerationError::UnresolvedError);
                    };
                    item_type.size_of()
                };

                let mut offset = offset + bytes.len();
                borrowed.push(Strip::Serialize(Serialized { data: bytes }));

                drop(borrowed);
                for element in data {
                    let _ = element.gencode(scope, instructions, offset)?;
                    offset += item_size;
                }
            }
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Vector<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Vector::Init {
                value: data,
                metadata,
                length,
                capacity,
            } => {
                let Some(signature) = metadata.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let item_size = {
                    let Some(item_type) = signature.get_item() else {
                        return Err(CodeGenerationError::UnresolvedError);
                    };
                    item_type.size_of()
                };
                let mut borrowed = instructions.as_ref().borrow_mut();
                let mut offset = offset;

                let vec_stack_address = offset;
                // Alloc and push heap address on stack
                borrowed.push(Strip::Alloc(Alloc {
                    size: item_size * data.len() + 16,
                }));
                offset += HEAP_ADDRESS_SIZE;

                let len_bytes = (*length as u64).to_le_bytes().as_slice().to_vec();
                let cap_bytes = (*capacity as u64).to_le_bytes().as_slice().to_vec();

                // Start of the Vec data
                let _data_offset = offset;
                // Push Length on stack
                offset += len_bytes.len();
                borrowed.push(Strip::Serialize(Serialized { data: len_bytes }));
                // Push Capacity on stack
                offset += cap_bytes.len();
                borrowed.push(Strip::Serialize(Serialized { data: cap_bytes }));

                drop(borrowed);
                for element in data {
                    let _ = element.gencode(scope, instructions, offset)?;
                    offset += item_size;
                }

                // Copy data on stack to heap at address
                let mut borrowed = instructions.as_ref().borrow_mut();
                // Copy heap address on top of the stack
                borrowed.push(Strip::Access(Access::Static {
                    address: MemoryAddress::Stack {
                        offset: vec_stack_address,
                    },
                    size: 8,
                }));

                // Take the address on the top of the stack
                // and copy the data on the stack in the heap at given address and given offset
                // ( removing the data from the stack )
                borrowed.push(Strip::MemCopy(MemCopy::Take {
                    offset: vec_stack_address + 8,
                    size: item_size * data.len() + 16,
                }));
            }
            Vector::Def { capacity, metadata } => {
                let Some(signature) = metadata.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let item_size = {
                    let Some(item_type) = signature.get_item() else {
                        return Err(CodeGenerationError::UnresolvedError);
                    };
                    item_type.size_of()
                };
                let mut borrowed = instructions.as_ref().borrow_mut();
                // Alloc and push heap address on stack
                borrowed.push(Strip::Alloc(Alloc {
                    size: item_size * capacity,
                }));
            }
        }

        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Tuple<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        let Some(signature) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let mut offset = offset;
        for (idx, element) in self.value.iter().enumerate() {
            let item_size = {
                let Some(item_type) =
                    // <Either<UserType, StaticType> as GetSubTypes>::get_nth(signature, &idx)
                    signature.get_nth(&idx)
                else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                item_type.size_of()
            };
            let _ = element.gencode(scope, instructions, offset)?;
            offset += item_size;
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Closure<Scope> {
    fn gencode(
        &self,
        _scope: &Rc<RefCell<Scope>>,
        _instructions: &Rc<RefCell<Vec<Strip>>>,
        _offset: usize,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Address<Scope> {
    fn gencode(
        &self,
        _scope: &Rc<RefCell<Scope>>,
        _instructions: &Rc<RefCell<Vec<Strip>>>,
        _offset: usize,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for PtrAccess<Scope> {
    fn gencode(
        &self,
        _scope: &Rc<RefCell<Scope>>,
        _instructions: &Rc<RefCell<Vec<Strip>>>,
        _offset: usize,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Channel<Scope> {
    fn gencode(
        &self,
        _scope: &Rc<RefCell<Scope>>,
        _instructions: &Rc<RefCell<Vec<Strip>>>,
        _offset: usize,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Struct<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
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
        let mut offset = offset;

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

            let _ = field_expr.gencode(scope, instructions, offset)?;
            // Add padding
            let padding = align(field_size) - field_size;
            if padding > 0 {
                let mut borrowed = instructions.as_ref().borrow_mut();
                borrowed.push(Strip::Serialize(Serialized {
                    data: vec![0; padding],
                }));
            }
            offset += field_size;
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Union<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
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
        let mut offset = offset;
        let Some((idx, struct_type)) = union_type
            .variants
            .iter()
            .enumerate()
            .find(|(_idx, (id, _))| id == &self.variant)
            .map(|(idx, (_, st))| (idx, st))
        else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        {
            let mut borrowed = instructions.as_ref().borrow_mut();
            borrowed.push(Strip::Serialize(Serialized {
                data: (idx as u64).to_le_bytes().to_vec(),
            }));
        }
        offset += 8;

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

            let _ = field_expr.gencode(scope, instructions, offset)?;
            // Add padding
            let padding = align(field_size) - field_size;
            if padding > 0 {
                let mut borrowed = instructions.as_ref().borrow_mut();
                borrowed.push(Strip::Serialize(Serialized {
                    data: vec![0; padding],
                }));
            }
            offset += field_size;
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Enum {
    fn gencode(
        &self,
        _scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        _offset: usize,
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
        let mut borrowed = instructions.as_ref().borrow_mut();
        borrowed.push(Strip::Serialize(Serialized {
            data: (index as u64).to_le_bytes().to_vec(),
        }));
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Map<Scope> {
    fn gencode(
        &self,
        _scope: &Rc<RefCell<Scope>>,
        _instructions: &Rc<RefCell<Vec<Strip>>>,
        _offset: usize,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;

    use super::*;
    use crate::{
        ast::{
            expressions::{data::Data, Atomic, Expression},
            TryParse,
        },
        semantic::{
            scope::{
                scope_impl::Scope,
                static_types::{PrimitiveType, TupleType, VecType},
                user_type_impl, ScopeApi,
            },
            Resolve, TypeOf,
        },
        vm::{
            allocator::Memory,
            vm::{DeserializeFrom, Executable},
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
                .expect("Semantic check should have succeeded");

            // Code generation.
            let instructions = Rc::new(RefCell::new(Vec::default()));
            expr.gencode(&scope, &instructions, 0)
                .expect("Code generation should have succeeded");

            let instructions = instructions.as_ref().take();
            assert!(instructions.len() > 0);

            // Execute the instructions.
            let memory = Memory::new();
            for instruction in instructions {
                instruction
                    .execute(&memory)
                    .expect("Execution should have succeeded");
            }

            (expr, memory)
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
                .expect("Semantic check should have succeeded");

            // Code generation.
            let instructions = Rc::new(RefCell::new(Vec::default()));
            expr.gencode(&scope, &instructions, 0)
                .expect("Code generation should have succeeded");

            let instructions = instructions.as_ref().take();
            assert!(instructions.len() > 0);

            // Execute the instructions.
            let memory = Memory::new();
            for instruction in instructions {
                instruction
                    .execute(&memory)
                    .expect("Execution should have succeeded");
            }

            (expr, memory)
        }};
    }

    #[macro_export]
    macro_rules! clear_stack {
        ($memory:ident) => {{
            let top = $memory.stack.top();
            let data = $memory.stack.pop(top).expect("Read should have succeeded");
            assert!($memory.stack.top().is_zero());
            data
        }};
    }

    #[test]
    fn valid_number() {
        let (expr_u64, memory) = compile_expression!(Primitive, "420");
        let data_u64 = clear_stack!(memory);
        assert_number!(expr_u64, data_u64, U64);

        let (expr_u128, memory) = compile_expression!(Primitive, "420u128");
        let data_u128 = clear_stack!(memory);
        assert_number!(expr_u128, data_u128, U128);

        let (expr_u64, memory) = compile_expression!(Primitive, "420u64");
        let data_u64 = clear_stack!(memory);
        assert_number!(expr_u64, data_u64, U64);

        let (expr_u32, memory) = compile_expression!(Primitive, "420u32");
        let data_u32 = clear_stack!(memory);
        assert_number!(expr_u32, data_u32, U32);

        let (expr_u16, memory) = compile_expression!(Primitive, "420u16");
        let data_u16 = clear_stack!(memory);
        assert_number!(expr_u16, data_u16, U16);

        let (expr_u8, memory) = compile_expression!(Primitive, "20u8");
        let data_u8 = clear_stack!(memory);
        assert_number!(expr_u8, data_u8, U8);

        let (expr_i128, memory) = compile_expression!(Primitive, "420i128");
        let data_i128 = clear_stack!(memory);
        assert_number!(expr_i128, data_i128, I128);

        let (expr_i64, memory) = compile_expression!(Primitive, "420i64");
        let data_i64 = clear_stack!(memory);
        assert_number!(expr_i64, data_i64, I64);

        let (expr_i32, memory) = compile_expression!(Primitive, "420i32");
        let data_i32 = clear_stack!(memory);
        assert_number!(expr_i32, data_i32, I32);

        let (expr_i16, memory) = compile_expression!(Primitive, "420i16");
        let data_i16 = clear_stack!(memory);
        assert_number!(expr_i16, data_i16, I16);

        let (expr_i8, memory) = compile_expression!(Primitive, "20i8");
        let data_i8 = clear_stack!(memory);
        assert_number!(expr_i8, data_i8, I8);
    }

    #[test]
    fn valid_primitive() {
        let (expr, memory) = compile_expression!(Primitive, "'a'");
        let data = clear_stack!(memory);
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Char,
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, expr);
        let (expr, memory) = compile_expression!(Primitive, "true");
        let data = clear_stack!(memory);
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Bool,
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, expr);

        let (expr, memory) = compile_expression!(Primitive, "420.69");
        let data = clear_stack!(memory);
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Float,
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, expr);
    }

    #[test]
    fn valid_tuple() {
        let (expr, memory) = compile_expression!(Tuple, "(true,17)");
        let data = clear_stack!(memory);
        let result: Tuple<Scope> = TupleType(vec![
            Either::Static(StaticType::Primitive(PrimitiveType::Bool).into()),
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()),
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
        let (expr, memory) = compile_expression!(Tuple, "(420i128,true,17,'a')");
        let data = clear_stack!(memory);
        let result: Tuple<Scope> = TupleType(vec![
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::I128)).into()),
            Either::Static(StaticType::Primitive(PrimitiveType::Bool).into()),
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()),
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
        let (expr, memory) = compile_expression_with_type!(
            Struct,
            r##"
        Point {
            x : 420u64,
            y : 69i32
        }
        "##,
            user_type
        );
        let data = clear_stack!(memory);
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
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64))
                                        .into(),
                                ),
                            ),
                            (
                                "y".into(),
                                Either::Static(
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64))
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
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64))
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
        let (expr, memory) = compile_expression_with_type!(
            Union,
            r##"
            Geo::Point { x : 2, y : 8}
        "##,
            user_type
        );
        let data = clear_stack!(memory);
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
        let (expr, memory) = compile_expression_with_type!(
            Enum,
            r##"
            Geo::Point
        "##,
            user_type
        );
        let data = clear_stack!(memory);
        let result: Enum =
            <user_type_impl::Enum as DeserializeFrom<Scope>>::deserialize_from(&user_type, &data)
                .expect("Deserialization should have succeeded");
        assert_eq!(expr, result)
    }

    #[test]
    fn valid_vector() {
        let (expr, memory) = compile_expression!(Vector, "vec[1,2,3]");
        let data = clear_stack!(memory);

        let arr: [u8; 8] = data.try_into().expect("");
        let heap_address = u64::from_le_bytes(arr);

        let data = memory
            .heap
            .read(heap_address as usize, 8 * 3 + 16)
            .expect("Heap Read should have succeeded");

        let result: Vector<Scope> = VecType(Box::new(Either::Static(
            StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
        )))
        .deserialize_from(&data)
        .expect("Deserialization should have succeeded");

        dbg!(result);
    }
}
