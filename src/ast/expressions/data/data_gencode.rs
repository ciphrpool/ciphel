use std::{
    borrow::BorrowMut,
    cell::{Cell, RefCell},
    rc::Rc,
};

use nom::bytes;
use num_traits::ToBytes;

use crate::{
    ast::utils::strings::ID,
    semantic::{
        scope::{
            chan_impl::Chan,
            event_impl::Event,
            static_types::{NumberType, StaticType, TupleType},
            type_traits::GetSubTypes,
            user_type_impl::UserType,
            var_impl::Var,
            ScopeApi,
        },
        Either, SizeOf,
    },
    vm::{
        allocator::{heap::HEAP_ADDRESS_SIZE, MemoryAddress},
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
    Address, Channel, Closure, Enum, FieldAccess, ListAccess, Map, NumAccess, Primitive, PtrAccess,
    Slice, Struct, Tuple, Union, VarID, Variable, Vector,
};

impl<Scope: ScopeApi> GenerateCode<Scope> for Primitive {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        codes: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        let mut borrowed = codes.as_ref().borrow_mut();
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
//         codes: &Rc<RefCell<Vec<Strip>>>,
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
//                     let mut borrowed = codes.as_ref().borrow_mut();
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
//                         .get_offset(start_offset, address, scope, codes, signature)?;
//                 let ((field_offset, field_size), field_type) =
//                     field
//                         .as_ref()
//                         .get_offset(start_offset, var_offset, scope, codes, &var_type)?;

//                 {
//                     let mut borrowed = codes.as_ref().borrow_mut();
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
//                         .get_offset(start_offset, address, scope, codes, signature)?;
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
//                     let mut borrowed = codes.as_ref().borrow_mut();
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
//                         .get_offset(start_offset, address, scope, codes, signature)?;
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
//                         let _ = index.gencode(scope, codes, start_offset, signature)?;
//                         {
//                             let mut borrowed = codes.as_ref().borrow_mut();
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
//                             let mut borrowed = codes.as_ref().borrow_mut();
//                             borrowed.push(Strip::Access(Access::Variable {
//                                 address: MemoryAddress::Stack {
//                                     offset: var_offset + 16, /* LENGTH + CAPACITY */
//                                 },
//                                 size: var_size,
//                             }));
//                         }
//                         let _ = index.gencode(scope, codes, start_offset, signature)?;
//                         {
//                             let mut borrowed = codes.as_ref().borrow_mut();
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
//         codes: &Rc<RefCell<Vec<Strip>>>,
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

//                 let mut borrowed = codes.as_ref().borrow_mut();
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
//                 //     var.get_offset(offset, offset, scope, codes, signature)?;
//                 // let _ = field.get_offset(offset, offset, scope, codes, &var_type)?;
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
        codes: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Variable::Var(VarID { id, metadata }) => {
                let var = scope
                    .borrow()
                    .find_var(id)
                    .map_err(|_| CodeGenerationError::UnresolvedError)?;

                let Some(address) = &var.as_ref().address else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let var_type = &var.as_ref().type_sig;
                let var_size = var_type.size_of();

                let mut borrowed = codes.as_ref().borrow_mut();
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
                metadata,
            }) => {
                // Locate the variable
                let _ = var.locate(scope, codes, offset)?;
                // Access the field
                let Some(from_type) = var.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(offset) = from_type.get_field_offset(field.name()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    let mut borrowed = codes.as_ref().borrow_mut();
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
                field.access_from(from_type, offset, scope, codes, offset + 8)
            }
            Variable::NumAccess(NumAccess {
                var,
                index,
                metadata,
            }) => {
                // Locate the variable
                let _ = var.locate(scope, codes, offset)?;

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
                    let mut borrowed = codes.as_ref().borrow_mut();
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
                let _ = var.locate(scope, codes, offset)?;

                // Access the field
                let _ = index.gencode(scope, codes, offset + 8)?;
                let Some(size) = metadata.signature().map(|sig| sig.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    let mut borrowed = codes.as_ref().borrow_mut();
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
            Variable::Var(VarID { id, metadata }) => metadata.signature(),
            Variable::FieldAccess(FieldAccess {
                var,
                field,
                metadata,
            }) => metadata.signature(),
            Variable::NumAccess(NumAccess {
                var,
                index,
                metadata,
            }) => metadata.signature(),
            Variable::ListAccess(ListAccess {
                var,
                index,
                metadata,
            }) => metadata.signature(),
        }
    }

    fn name(&self) -> &ID {
        match self {
            Variable::Var(VarID { id, metadata }) => id,
            Variable::FieldAccess(FieldAccess {
                var,
                field,
                metadata,
            }) => var.name(),
            Variable::NumAccess(NumAccess {
                var,
                index,
                metadata,
            }) => var.name(),
            Variable::ListAccess(ListAccess {
                var,
                index,
                metadata,
            }) => var.name(),
        }
    }

    fn access_from(
        &self,
        from_type: Either<UserType, StaticType>,
        offset: usize,
        scope: &Rc<RefCell<Scope>>,
        codes: &Rc<RefCell<Vec<Strip>>>,
        start_offset: usize,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Variable::Var(VarID { id, metadata }) => {
                let mut borrowed = codes.as_ref().borrow_mut();
                let Some(size) = metadata.signature().map(|sig| sig.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                borrowed.push(Strip::Access(Access::Runtime { size }));
                Ok(())
            }
            Variable::FieldAccess(FieldAccess {
                var,
                field,
                metadata,
            }) => {
                // Access the field
                let Some(from_type) = var.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(offset) = from_type.get_field_offset(field.name()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    let mut borrowed = codes.as_ref().borrow_mut();
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
                field.access_from(from_type, offset, scope, codes, start_offset)
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
                    let mut borrowed = codes.as_ref().borrow_mut();
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
                let _ = index.gencode(scope, codes, start_offset)?;
                let Some(size) = metadata.signature().map(|sig| sig.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    let mut borrowed = codes.as_ref().borrow_mut();
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
        codes: &Rc<RefCell<Vec<Strip>>>,
        start_offset: usize,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Variable::Var(VarID { id, metadata }) => {
                let var = scope
                    .borrow()
                    .find_var(id)
                    .map_err(|_| CodeGenerationError::UnresolvedError)?;

                let Some(address) = &var.as_ref().address else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let var_type = &var.as_ref().type_sig;
                let var_size = var_type.size_of();

                let mut borrowed = codes.as_ref().borrow_mut();
                borrowed.push(Strip::Locate(Locate {
                    address: MemoryAddress::Stack {
                        offset: address.as_ref().get(),
                    },
                }));

                Ok(())
            }
            Variable::FieldAccess(FieldAccess {
                var,
                field,
                metadata,
            }) => unreachable!(),
            Variable::NumAccess(NumAccess {
                var,
                index,
                metadata,
            }) => {
                let _ = var.locate(scope, codes, start_offset)?;
                let Some(from_type) = var.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(offset) = from_type.get_inline_field_offset(*index) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                {
                    let mut borrowed = codes.as_ref().borrow_mut();
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
                metadata,
            }) => {
                let _ = var.locate(scope, codes, start_offset)?;
                let _ = index.gencode(scope, codes, start_offset)?;
                {
                    let mut borrowed = codes.as_ref().borrow_mut();
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
        codes: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        let mut borrowed = codes.as_ref().borrow_mut();

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
                let mut bytes = (data.len() as u64).to_le_bytes().as_slice().to_vec();

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
                    let _ = element.gencode(scope, codes, offset)?;
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
        codes: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Vector::Init {
                value: data,
                metadata,
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
                let mut borrowed = codes.as_ref().borrow_mut();
                let mut offset = offset;

                let vec_stack_address = offset;
                // Alloc and push heap address on stack
                borrowed.push(Strip::Alloc(Alloc {
                    size: item_size * data.len(),
                }));
                offset += HEAP_ADDRESS_SIZE;

                let bytes = (data.len() as u64).to_le_bytes().as_slice().to_vec();

                // Start of the Vec data
                let data_offset = offset;
                // Push Length on stack
                offset += bytes.len();
                borrowed.push(Strip::Serialize(Serialized {
                    data: bytes.clone(),
                }));
                // Push Capacity on stack
                offset += bytes.len();
                borrowed.push(Strip::Serialize(Serialized { data: bytes }));

                drop(borrowed);
                for element in data {
                    let _ = element.gencode(scope, codes, offset)?;
                    offset += item_size;
                }

                // Copy data on stack to heap at address
                let mut borrowed = codes.as_ref().borrow_mut();
                // Push offset ( where the vec data starts) on top of the stack
                borrowed.push(Strip::Serialize(Serialized {
                    data: 0u64.to_le_bytes().to_vec(),
                }));

                // Push heap address on top of the stack
                borrowed.push(Strip::Access(Access::Static {
                    address: MemoryAddress::Stack {
                        offset: vec_stack_address,
                    },
                    size: HEAP_ADDRESS_SIZE,
                }));
                // Take the address and the offset on the top of the stack
                // and copy the data on the stack in the heap at given address and given offset
                // ( removing the data from the stack )
                borrowed.push(Strip::MemCopy(MemCopy::Take {
                    offset: data_offset,
                    size: item_size * data.len(),
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
                let mut borrowed = codes.as_ref().borrow_mut();
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
        codes: &Rc<RefCell<Vec<Strip>>>,
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
            let _ = element.gencode(scope, codes, offset)?;
            offset += item_size;
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Closure<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        codes: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Address<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        codes: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for PtrAccess<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        codes: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Channel<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        codes: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Struct<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        codes: &Rc<RefCell<Vec<Strip>>>,
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

            let _ = field_expr.gencode(scope, codes, offset)?;
            offset += field_size;
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Union<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        codes: &Rc<RefCell<Vec<Strip>>>,
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
            .find(|(idx, (id, _))| id == &self.variant)
            .map(|(idx, (_, st))| (idx, st))
        else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        {
            let mut borrowed = codes.as_ref().borrow_mut();
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

            let _ = field_expr.gencode(scope, codes, offset)?;
            offset += field_size;
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Enum {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        codes: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
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
        let mut borrowed = codes.as_ref().borrow_mut();
        borrowed.push(Strip::Serialize(Serialized {
            data: (index as u64).to_le_bytes().to_vec(),
        }));
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Map<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        codes: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}
