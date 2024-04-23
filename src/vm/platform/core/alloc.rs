use std::{
    cell::{Cell, Ref},
    vec,
};

use crate::semantic::scope::scope_impl::Scope;

use num_traits::ToBytes;

use crate::{
    ast::expressions::{
        Expression,
    },
    e_static, p_num,
    semantic::{
        scope::{
            static_types::{
                AddrType, NumberType, PrimitiveType, StaticType, StringType, VecType,
            },
            type_traits::{GetSubTypes},
        },
        AccessLevel, EType, Either, Info, Metadata, MutRc, Resolve, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        allocator::{align, stack::Offset},
        casm::{
            alloc::{Access, Alloc, Free},
            mem::Mem,
            operation::{
                OpPrimitive,
            },
            serialize::Serialized,
            Casm, CasmProgram,
        },
        platform::{utils::lexem, LibCasm},
        scheduler::Thread,
        vm::{CodeGenerationError, Executable, GenerateCode, RuntimeError},
    },
};



#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AppendKind {
    Vec,
    StrSlice,
    Char,
    String,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum DeleteKind {
    Vec,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ClearKind {
    Vec,
    String,
    Map,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AllocFn {
    Append {
        item_size: Cell<usize>,
        append_kind: Cell<AppendKind>,
    },
    Insert,
    Delete {
        item_size: Cell<usize>,
        delete_kind: Cell<DeleteKind>,
    },
    Len,
    Cap,
    Free,
    Alloc,
    Vec {
        with_capacity: Cell<bool>,
        item_size: Cell<usize>,
        metadata: Metadata,
    },
    Map,
    Chan,
    String {
        len: Cell<usize>,
        from_char: Cell<bool>,
    },

    SizeOf {
        size: Cell<usize>,
    },

    MemCopy,
    Clear {
        item_size: Cell<usize>,
        clear_kind: Cell<ClearKind>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AllocCasm {
    AppendChar,
    AppendItem(usize),
    AppendStrSlice(usize),
    AppendString,
    Insert,
    DeleteVec(usize),

    ClearVec(usize),
    ClearString(usize),
    ClearMap(usize),

    Len,
    Cap,
    Vec {
        item_size: usize,
        with_capacity: bool,
    },
    Map,
    Chan,
    StringFromSlice {
        len: usize,
    },
    StringFromChar,
}

impl AllocFn {
    pub fn from(suffixe: &Option<String>, id: &String) -> Option<Self> {
        match suffixe {
            Some(suffixe) => {
                if suffixe != lexem::CORE {
                    return None;
                }
            }
            None => {}
        }

        match id.as_str() {
            lexem::APPEND => Some(AllocFn::Append {
                item_size: Cell::new(0),
                append_kind: Cell::new(AppendKind::Vec),
            }),
            lexem::INSERT => Some(AllocFn::Insert),
            lexem::DELETE => Some(AllocFn::Delete {
                item_size: Cell::new(0),
                delete_kind: Cell::new(DeleteKind::Vec),
            }),
            lexem::LEN => Some(AllocFn::Len),
            lexem::CAP => Some(AllocFn::Cap),
            lexem::FREE => Some(AllocFn::Free),
            lexem::VEC => Some(AllocFn::Vec {
                with_capacity: Cell::new(false),
                item_size: Cell::new(0),
                metadata: Metadata::default(),
            }),
            lexem::MAP => Some(AllocFn::Map),
            lexem::CHAN => Some(AllocFn::Chan),
            lexem::STRING => Some(AllocFn::String {
                len: Cell::new(0),
                from_char: Cell::new(false),
            }),
            lexem::ALLOC => Some(AllocFn::Alloc),
            lexem::MEMCPY => Some(AllocFn::MemCopy),
            lexem::CLEAR => Some(AllocFn::Clear {
                item_size: Cell::new(0),
                clear_kind: Cell::new(ClearKind::Vec),
            }),
            lexem::SIZEOF => Some(AllocFn::SizeOf { size: Cell::new(0) }),
            _ => None,
        }
    }
}

impl Resolve for AllocFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression>;
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            AllocFn::Append {
                item_size,
                append_kind,
            } => {
                if extra.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let vector = &extra[0];
                let item = &extra[1];

                let _ = vector.resolve(scope, &None, &())?;
                let mut vector_type = vector.type_of(&scope.borrow())?;
                match &vector_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Address(AddrType(sub)) => vector_type = sub.as_ref().clone(),
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                match &vector_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Vec(_) => {
                            let item_type = vector_type.get_item();
                            append_kind.set(AppendKind::Vec);
                            let _ = item.resolve(scope, &item_type, &())?;
                            let Some(item_type) = item_type else {
                                return Err(SemanticError::IncorrectArguments);
                            };
                            item_size.set(item_type.size_of());
                            Ok(())
                        }
                        StaticType::String(_) => {
                            let _ = item.resolve(scope, &None, &())?;
                            let item_type = item.type_of(&scope.borrow())?;
                            match &item_type {
                                Either::Static(value) => match value.as_ref() {
                                    StaticType::Primitive(PrimitiveType::Char) => {
                                        append_kind.set(AppendKind::Char);
                                    }
                                    StaticType::String(_) => {
                                        append_kind.set(AppendKind::String);
                                    }
                                    StaticType::StrSlice(_) => {
                                        append_kind.set(AppendKind::StrSlice);
                                    }
                                    _ => return Err(SemanticError::IncorrectArguments),
                                },
                                _ => return Err(SemanticError::IncorrectArguments),
                            }
                            item_size.set(item_type.size_of());
                            Ok(())
                        }
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
            }
            AllocFn::Insert => todo!(),
            AllocFn::Delete {
                delete_kind,
                item_size,
            } => {
                if extra.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let vector = &extra[0];
                let index = &extra[1];

                let _ = vector.resolve(scope, &None, &())?;
                let mut vector_type = vector.type_of(&scope.borrow())?;
                match &vector_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Address(AddrType(sub)) => vector_type = sub.as_ref().clone(),
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                match &vector_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Vec(_) => {
                            let _ = index.resolve(scope, &Some(p_num!(U64)), &())?;
                            let index_type = index.type_of(&scope.borrow())?;
                            match &index_type {
                                Either::Static(value) => match value.as_ref() {
                                    StaticType::Primitive(PrimitiveType::Number(
                                        NumberType::U64,
                                    )) => {}
                                    _ => return Err(SemanticError::IncorrectArguments),
                                },
                                _ => return Err(SemanticError::IncorrectArguments),
                            }
                            let item_type = vector_type.get_item();
                            delete_kind.set(DeleteKind::Vec);
                            let Some(item_type) = item_type else {
                                return Err(SemanticError::IncorrectArguments);
                            };
                            item_size.set(item_type.size_of());
                            Ok(())
                        }
                        StaticType::Map(_) => {
                            todo!();
                            Ok(())
                        }
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
            }
            AllocFn::Free => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let address = &extra[0];

                let _ = address.resolve(scope, &None, &())?;
                let address_type = address.type_of(&scope.borrow())?;
                match &address_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Address(AddrType(_)) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(())
            }
            AllocFn::Vec {
                with_capacity,
                item_size,
                metadata,
            } => {
                if extra.len() > 2 || extra.len() < 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                if extra.len() == 2 {
                    with_capacity.set(true);
                }
                for param in extra {
                    let _ = param.resolve(scope, &Some(p_num!(U64)), &())?;
                }
                if context.is_none() {
                    return Err(SemanticError::CantInferType);
                }
                match &context {
                    Some(value) => match value {
                        Either::Static(value) => match value.as_ref() {
                            StaticType::Vec(VecType(item)) => item_size.set(item.size_of()),
                            _ => return Err(SemanticError::IncompatibleTypes),
                        },
                        Either::User(_) => return Err(SemanticError::IncompatibleTypes),
                    },
                    None => unreachable!(),
                }
                let mut borrowed_metadata = metadata
                    .info
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| SemanticError::Default)?;
                *borrowed_metadata = Info::Resolved {
                    context: context.clone(),
                    signature: context.clone(),
                };
                Ok(())
            }
            AllocFn::Map => todo!(),
            AllocFn::Chan => todo!(),
            AllocFn::String { len, from_char } => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let param = extra.first().unwrap();
                let _ = param.resolve(scope, &None, &())?;
                let param_type = param.type_of(&scope.borrow())?;
                match param_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::StrSlice(slice) => {
                            from_char.set(false);
                            len.set(slice.size_of());
                        }
                        StaticType::Primitive(PrimitiveType::Char) => {
                            from_char.set(true);
                        }
                        _ => {
                            return Err(SemanticError::IncorrectArguments);
                        }
                    },
                    Either::User(_) => {
                        return Err(SemanticError::IncorrectArguments);
                    }
                }
                Ok(())
            }
            AllocFn::Alloc => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let size = &extra[0];

                let _ = size.resolve(scope, &Some(p_num!(U64)), &())?;
                let size_type = size.type_of(&scope.borrow())?;
                match &size_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(())
            }
            AllocFn::Len => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let address = &extra[0];

                let _ = address.resolve(scope, &None, &())?;
                let address_type = address.type_of(&scope.borrow())?;
                match &address_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::String(_) => {}
                        StaticType::Vec(_) => {}
                        StaticType::Map(_) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(())
            }
            AllocFn::Cap => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let address = &extra[0];

                let _ = address.resolve(scope, &None, &())?;
                let address_type = address.type_of(&scope.borrow())?;
                match &address_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::String(_) => {}
                        StaticType::Vec(_) => {}
                        StaticType::Map(_) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(())
            }
            AllocFn::SizeOf { size } => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let param = &extra[0];

                let _ = param.resolve(scope, &None, &())?;
                let param_type = param.type_of(&scope.borrow())?;

                size.set(param_type.size_of());

                Ok(())
            }
            AllocFn::MemCopy => {
                if extra.len() != 3 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let dest = &extra[0];
                let src = &extra[1];
                let size = &extra[2];

                let _ = dest.resolve(scope, &None, &())?;
                let _ = src.resolve(scope, &None, &())?;
                let dest_type = dest.type_of(&scope.borrow())?;
                let src_type = src.type_of(&scope.borrow())?;
                match &dest_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Address(_) => {}
                        StaticType::String(_) => {}
                        StaticType::Vec(_) => {}
                        StaticType::Map(_) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                match &src_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Address(_) => {}
                        StaticType::String(_) => {}
                        StaticType::Vec(_) => {}
                        StaticType::Map(_) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                let _ = size.resolve(scope, &Some(p_num!(U64)), &())?;
                let size_type = size.type_of(&scope.borrow())?;
                match &size_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(())
            }
            AllocFn::Clear {
                clear_kind,
                item_size,
            } => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let src = &extra[0];

                let _ = src.resolve(scope, &None, &())?;
                let src_type = src.type_of(&scope.borrow())?;

                match &src_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Address(AddrType(inner)) => match inner.as_ref() {
                            Either::Static(value) => match value.as_ref() {
                                StaticType::String(_) => {
                                    clear_kind.set(ClearKind::String);
                                    item_size.set(1);
                                }
                                StaticType::Vec(_) => {
                                    clear_kind.set(ClearKind::Vec);
                                    let item_type = src_type.get_item();
                                    let Some(item_type) = item_type else {
                                        return Err(SemanticError::IncorrectArguments);
                                    };
                                    item_size.set(item_type.size_of());
                                }
                                StaticType::Map(_) => {
                                    clear_kind.set(ClearKind::Map);
                                    let item_type = src_type.get_item();
                                    let Some(item_type) = item_type else {
                                        return Err(SemanticError::IncorrectArguments);
                                    };
                                    item_size.set(item_type.size_of());
                                }
                                _ => return Err(SemanticError::IncorrectArguments),
                            },
                            _ => return Err(SemanticError::IncorrectArguments),
                        },

                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                Ok(())
            }
        }
    }
}
impl TypeOf for AllocFn {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            AllocFn::Append { .. } => Ok(e_static!(StaticType::Unit)),
            AllocFn::Insert => todo!(),
            AllocFn::Delete { .. } => Ok(e_static!(StaticType::Unit)),
            AllocFn::Free => Ok(e_static!(StaticType::Unit)),
            AllocFn::Vec { metadata, .. } => {
                metadata.signature().ok_or(SemanticError::NotResolvedYet)
            }
            AllocFn::Map => todo!(),
            AllocFn::Chan => todo!(),
            AllocFn::String { .. } => Ok(e_static!(StaticType::String(StringType()))),
            AllocFn::Alloc => Ok(e_static!(StaticType::Any)),
            AllocFn::Len => Ok(p_num!(U64)),
            AllocFn::Cap => Ok(p_num!(U64)),
            AllocFn::SizeOf { .. } => Ok(p_num!(U64)),
            AllocFn::MemCopy => Ok(e_static!(StaticType::Unit)),
            AllocFn::Clear { .. } => Ok(e_static!(StaticType::Unit)),
        }
    }
}

impl GenerateCode for AllocFn {
    fn gencode(
        &self,
        _scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            AllocFn::Append {
                item_size,
                append_kind,
            } => match append_kind.get() {
                AppendKind::Vec => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::AppendItem(item_size.get())),
                ))),
                AppendKind::StrSlice => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::AppendStrSlice(item_size.get())),
                ))),
                AppendKind::Char => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::AppendChar),
                ))),
                AppendKind::String => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::AppendString),
                ))),
            },
            AllocFn::Insert => todo!(),
            AllocFn::Delete {
                delete_kind,
                item_size,
            } => match delete_kind.get() {
                DeleteKind::Vec => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::DeleteVec(item_size.get())),
                ))),
            },
            AllocFn::Free => {
                instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
                instructions.push(Casm::Free(Free()));
            }
            AllocFn::Vec {
                with_capacity,
                item_size,
                ..
            } => instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                AllocCasm::Vec {
                    item_size: item_size.get(),
                    with_capacity: with_capacity.get(),
                },
            )))),
            AllocFn::Map => todo!(),
            AllocFn::Chan => todo!(),
            AllocFn::String { from_char, len } => {
                if from_char.get() {
                    instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                        AllocCasm::StringFromChar,
                    ))))
                } else {
                    instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                        AllocCasm::StringFromSlice { len: len.get() },
                    ))))
                }
            }
            AllocFn::Alloc => {
                instructions.push(Casm::Alloc(Alloc::Heap { size: None }));
            }
            AllocFn::Len => instructions.push(Casm::Platform(LibCasm::Core(
                super::CoreCasm::Alloc(AllocCasm::Len),
            ))),
            AllocFn::Cap => instructions.push(Casm::Platform(LibCasm::Core(
                super::CoreCasm::Alloc(AllocCasm::Cap),
            ))),
            AllocFn::SizeOf { size } => {
                instructions.push(Casm::Pop(size.get()));
                instructions.push(Casm::Serialize(Serialized {
                    data: size.get().to_le_bytes().to_vec(),
                }));
            }
            AllocFn::MemCopy => instructions.push(Casm::MemCopy(Mem::MemCopy)),
            AllocFn::Clear {
                clear_kind,
                item_size,
            } => match clear_kind.get() {
                ClearKind::Vec => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::ClearVec(item_size.get())),
                ))),
                ClearKind::String => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::ClearString(1)),
                ))),
                ClearKind::Map => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::ClearMap(item_size.get())),
                ))),
            },
        }
        Ok(())
    }
}

impl Executable for AllocCasm {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match self {
            AllocCasm::AppendChar => {
                let chara = OpPrimitive::get_char(&thread.memory())?;
                let chara = chara.to_string();
                let item_data = chara.as_bytes().to_vec();
                let item_len = chara.len();

                let vec_stack_address = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                let vec_heap_address_bytes = thread
                    .env
                    .stack
                    .read(
                        Offset::SB(vec_stack_address as usize),
                        AccessLevel::Direct,
                        8,
                    )
                    .map_err(|err| err.into())?;
                let vec_heap_address_bytes =
                    TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);

                let previous_len_bytes = thread
                    .runtime
                    .heap
                    .read(vec_heap_address as usize, 8)
                    .map_err(|e| e.into())?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let previous_cap_bytes = thread
                    .runtime
                    .heap
                    .read(vec_heap_address as usize + 8, 8)
                    .map_err(|e| e.into())?;
                let previous_cap_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_cap_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_cap = u64::from_le_bytes(*previous_cap_bytes);

                let (new_vec_heap_address, new_len, new_cap) = if previous_len + (item_len as u64)
                    >= previous_cap
                {
                    /* Reallocation */
                    let size = align(((previous_len + (item_len as u64)) * 2) as usize) + 16;
                    let address = thread
                        .runtime
                        .heap
                        .realloc(vec_heap_address as usize - 8, size)
                        .map_err(|e| e.into())?;
                    let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;
                    (
                        address as u64,
                        previous_len + (item_len as u64),
                        size as u64,
                    )
                } else {
                    (
                        vec_heap_address,
                        previous_len + (item_len as u64),
                        previous_cap,
                    )
                };
                let len_bytes = new_len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = new_cap.to_le_bytes().as_slice().to_vec();
                /* Write len */
                let _ = thread
                    .runtime
                    .heap
                    .write(new_vec_heap_address as usize, &len_bytes)
                    .map_err(|e| e.into())?;
                /* Write capacity */
                let _ = thread
                    .runtime
                    .heap
                    .write(new_vec_heap_address as usize + 8, &cap_bytes)
                    .map_err(|e| e.into())?;

                /* Write new item */
                let _ = thread
                    .runtime
                    .heap
                    .write(
                        new_vec_heap_address as usize + 16 + new_len as usize - item_len,
                        &item_data,
                    )
                    .map_err(|e| e.into())?;
                /* Update vector pointer */
                let _ = thread
                    .env
                    .stack
                    .write(
                        Offset::SB(vec_stack_address as usize),
                        AccessLevel::Direct,
                        &new_vec_heap_address.to_le_bytes(),
                    )
                    .map_err(|err| err.into())?;
            }
            AllocCasm::AppendItem(item_size) | AllocCasm::AppendStrSlice(item_size) => {
                let item_data = thread.env.stack.pop(*item_size).map_err(|e| e.into())?;

                let vec_stack_address = OpPrimitive::get_num8::<u64>(&thread.memory())?;

                let vec_heap_address_bytes = thread
                    .env
                    .stack
                    .read(
                        Offset::SB(vec_stack_address as usize),
                        AccessLevel::Direct,
                        8,
                    )
                    .map_err(|err| err.into())?;
                let vec_heap_address_bytes =
                    TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);
                let previous_len_bytes = thread
                    .runtime
                    .heap
                    .read(vec_heap_address as usize, 8)
                    .map_err(|e| e.into())?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let previous_cap_bytes = thread
                    .runtime
                    .heap
                    .read(vec_heap_address as usize + 8, 8)
                    .map_err(|e| e.into())?;
                let previous_cap_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_cap_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_cap = u64::from_le_bytes(*previous_cap_bytes);

                let len_offset = match self {
                    AllocCasm::AppendItem(_) => 1,
                    AllocCasm::AppendStrSlice(_) => *item_size as u64,
                    _ => unreachable!(),
                };

                let size_factor = match self {
                    AllocCasm::AppendItem(_) => *item_size,
                    AllocCasm::AppendStrSlice(_) => 1,
                    _ => unreachable!(),
                };

                let (new_vec_heap_address, new_len, new_cap) = if previous_len + len_offset
                    >= previous_cap
                {
                    /* Reallocation */
                    let size = align(((previous_len + len_offset) * 2) as usize * size_factor + 16);
                    let address = thread
                        .runtime
                        .heap
                        .realloc(vec_heap_address as usize - 8, size)
                        .map_err(|e| e.into())?;
                    let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;
                    (
                        address as u64,
                        previous_len + len_offset,
                        align(((previous_len + len_offset) * 2) as usize) as u64,
                    )
                } else {
                    (vec_heap_address, previous_len + len_offset, previous_cap)
                };

                let len_bytes = new_len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = new_cap.to_le_bytes().as_slice().to_vec();
                /* Write len */
                let _ = thread
                    .runtime
                    .heap
                    .write(new_vec_heap_address as usize, &len_bytes)
                    .map_err(|e| e.into())?;
                /* Write capacity */
                let _ = thread
                    .runtime
                    .heap
                    .write(new_vec_heap_address as usize + 8, &cap_bytes)
                    .map_err(|e| e.into())?;

                /* Write new item */
                let _ = thread
                    .runtime
                    .heap
                    .write(
                        new_vec_heap_address as usize
                            + 16
                            + (new_len as usize * size_factor as usize)
                            - *item_size,
                        &item_data,
                    )
                    .map_err(|e| e.into())?;
                /* Update vector pointer */
                let _ = thread
                    .env
                    .stack
                    .write(
                        Offset::SB(vec_stack_address as usize),
                        AccessLevel::Direct,
                        &new_vec_heap_address.to_le_bytes(),
                    )
                    .map_err(|err| err.into())?;
            }
            AllocCasm::AppendString => {
                let item_heap_address = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                let item_len_bytes = thread
                    .runtime
                    .heap
                    .read(item_heap_address as usize, 8)
                    .map_err(|e| e.into())?;
                let item_len_bytes = TryInto::<&[u8; 8]>::try_into(item_len_bytes.as_slice())
                    .map_err(|_| RuntimeError::Deserialization)?;
                let item_len = u64::from_le_bytes(*item_len_bytes);

                let item_data = thread
                    .runtime
                    .heap
                    .read(item_heap_address as usize + 16, item_len as usize)
                    .map_err(|e| e.into())?;

                let _ = thread
                    .runtime
                    .heap
                    .free(item_heap_address as usize - 8)
                    .map_err(|e| e.into())?;

                let vec_stack_address = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                let vec_heap_address_bytes = thread
                    .env
                    .stack
                    .read(
                        Offset::SB(vec_stack_address as usize),
                        AccessLevel::Direct,
                        8,
                    )
                    .map_err(|err| err.into())?;
                let vec_heap_address_bytes =
                    TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);

                let previous_len_bytes = thread
                    .runtime
                    .heap
                    .read(vec_heap_address as usize, 8)
                    .map_err(|e| e.into())?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let previous_cap_bytes = thread
                    .runtime
                    .heap
                    .read(vec_heap_address as usize + 8, 8)
                    .map_err(|e| e.into())?;
                let previous_cap_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_cap_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_cap = u64::from_le_bytes(*previous_cap_bytes);

                let (new_vec_heap_address, new_len, new_cap) = if previous_len + item_len
                    >= previous_cap
                {
                    /* Reallocation */
                    let size = align(((previous_len + item_len) * 2) as usize) + 16;
                    let address = thread
                        .runtime
                        .heap
                        .realloc(vec_heap_address as usize - 8, size)
                        .map_err(|e| e.into())?;
                    let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;
                    (address as u64, previous_len + item_len, size as u64)
                } else {
                    (vec_heap_address, previous_len + item_len, previous_cap)
                };
                let len_bytes = new_len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = new_cap.to_le_bytes().as_slice().to_vec();
                /* Write len */
                let _ = thread
                    .runtime
                    .heap
                    .write(new_vec_heap_address as usize, &len_bytes)
                    .map_err(|e| e.into())?;
                /* Write capacity */
                let _ = thread
                    .runtime
                    .heap
                    .write(new_vec_heap_address as usize + 8, &cap_bytes)
                    .map_err(|e| e.into())?;

                /* Write new item */
                let _ = thread
                    .runtime
                    .heap
                    .write(
                        new_vec_heap_address as usize + 16 + new_len as usize - item_len as usize,
                        &item_data,
                    )
                    .map_err(|e| e.into())?;
                /* Update vector pointer */
                let _ = thread
                    .env
                    .stack
                    .write(
                        Offset::SB(vec_stack_address as usize),
                        AccessLevel::Direct,
                        &new_vec_heap_address.to_le_bytes(),
                    )
                    .map_err(|err| err.into())?;
            }
            AllocCasm::Insert => todo!(),
            AllocCasm::DeleteVec(item_size) => {
                let index = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                let vec_stack_address = OpPrimitive::get_num8::<u64>(&thread.memory())?;

                let vec_heap_address_bytes = thread
                    .env
                    .stack
                    .read(
                        Offset::SB(vec_stack_address as usize),
                        AccessLevel::Direct,
                        8,
                    )
                    .map_err(|err| err.into())?;

                let vec_heap_address_bytes =
                    TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);
                let previous_len_bytes = thread
                    .runtime
                    .heap
                    .read(vec_heap_address as usize, 8)
                    .map_err(|e| e.into())?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let item_size = *item_size as u64;
                // let data_to_move_size = previous_len * item_size - (index + 1) * item_size;

                if index < previous_len - 1
                /* index not last item */
                {
                    /* move below */
                    let data = thread
                        .runtime
                        .heap
                        .read(
                            vec_heap_address as usize + 16 + ((index + 1) * item_size) as usize,
                            (previous_len * item_size - (index + 1) * item_size) as usize,
                        )
                        .map_err(|e| e.into())?;
                    let _ = thread
                        .runtime
                        .heap
                        .write(
                            vec_heap_address as usize + 16 + (index * item_size) as usize,
                            &data,
                        )
                        .map_err(|e| e.into())?;
                }
                /* clear last item */
                let _ = thread
                    .runtime
                    .heap
                    .write(
                        vec_heap_address as usize + 16 + ((previous_len - 1) * item_size) as usize,
                        &vec![0; item_size as usize],
                    )
                    .map_err(|e| e.into())?;

                let len_bytes = (previous_len - 1).to_le_bytes().as_slice().to_vec();
                /* Write len */
                let _ = thread
                    .runtime
                    .heap
                    .write(vec_heap_address as usize, &len_bytes)
                    .map_err(|e| e.into())?;
            }
            AllocCasm::Vec {
                item_size,
                with_capacity,
            } => {
                /* */
                let (len, cap) = if *with_capacity {
                    let cap = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                    let len = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                    (len, cap)
                } else {
                    let len = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                    (len, align(len as usize) as u64)
                };
                let alloc_size = cap * (*item_size as u64) + 16;

                let len_bytes = len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                let address = thread
                    .runtime
                    .heap
                    .alloc(alloc_size as usize)
                    .map_err(|e| e.into())?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

                /* Write len */
                let _ = thread
                    .runtime
                    .heap
                    .write(address, &len_bytes)
                    .map_err(|e| e.into())?;
                /* Write capacity */
                let _ = thread
                    .runtime
                    .heap
                    .write(address + 8, &cap_bytes)
                    .map_err(|e| e.into())?;

                let _ = thread
                    .env
                    .stack
                    .push_with(&address.to_le_bytes())
                    .map_err(|e| e.into())?;
            }
            AllocCasm::Map => todo!(),
            AllocCasm::Chan => todo!(),
            AllocCasm::StringFromSlice { len } => {
                let len = *len as u64;
                let cap = align(len as usize) as u64;
                let alloc_size = cap + 16;

                let len_bytes = len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                let address = thread
                    .runtime
                    .heap
                    .alloc(alloc_size as usize)
                    .map_err(|e| e.into())?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

                let data = thread.env.stack.pop(len as usize).map_err(|e| e.into())?;
                /* Write len */
                let _ = thread
                    .runtime
                    .heap
                    .write(address, &len_bytes)
                    .map_err(|e| e.into())?;
                /* Write capacity */
                let _ = thread
                    .runtime
                    .heap
                    .write(address + 8, &cap_bytes)
                    .map_err(|e| e.into())?;
                /* Write slice */
                let _ = thread
                    .runtime
                    .heap
                    .write(address + 16, &data)
                    .map_err(|e| e.into())?;

                let _ = thread
                    .env
                    .stack
                    .push_with(&address.to_le_bytes())
                    .map_err(|e| e.into())?;
            }
            AllocCasm::StringFromChar => {
                let chara = OpPrimitive::get_char(&thread.memory())?;
                let chara = chara.to_string();
                let chara = chara.as_bytes();

                let len = chara.len() as u64;
                let cap = align(len as usize) as u64;
                let alloc_size = cap + 16;

                let len_bytes = len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                let address = thread
                    .runtime
                    .heap
                    .alloc(alloc_size as usize)
                    .map_err(|e| e.into())?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

                /* Write len */
                let _ = thread
                    .runtime
                    .heap
                    .write(address, &len_bytes)
                    .map_err(|e| e.into())?;
                /* Write capacity */
                let _ = thread
                    .runtime
                    .heap
                    .write(address + 8, &cap_bytes)
                    .map_err(|e| e.into())?;
                /* Write slice */
                let _ = thread
                    .runtime
                    .heap
                    .write(address + 16, &chara.to_vec())
                    .map_err(|e| e.into())?;

                let _ = thread
                    .env
                    .stack
                    .push_with(&address.to_le_bytes())
                    .map_err(|e| e.into())?;
            }
            AllocCasm::Len => {
                let vec_heap_address = OpPrimitive::get_num8::<u64>(&thread.memory())?;

                let len_bytes = thread
                    .runtime
                    .heap
                    .read(vec_heap_address as usize, 8)
                    .map_err(|e| e.into())?;
                let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                    .map_err(|_| RuntimeError::Deserialization)?;
                let len = u64::from_le_bytes(*len_bytes);

                let _ = thread
                    .env
                    .stack
                    .push_with(&len.to_le_bytes())
                    .map_err(|e| e.into())?;
            }
            AllocCasm::Cap => {
                let vec_heap_address = OpPrimitive::get_num8::<u64>(&thread.memory())?;

                let cap_bytes = thread
                    .runtime
                    .heap
                    .read(vec_heap_address as usize + 8, 8)
                    .map_err(|e| e.into())?;
                let cap_bytes = TryInto::<&[u8; 8]>::try_into(cap_bytes.as_slice())
                    .map_err(|_| RuntimeError::Deserialization)?;
                let cap = u64::from_le_bytes(*cap_bytes);

                let _ = thread
                    .env
                    .stack
                    .push_with(&cap.to_le_bytes())
                    .map_err(|e| e.into())?;
            }
            AllocCasm::ClearVec(item_size) | AllocCasm::ClearString(item_size) => {
                let vec_stack_address = OpPrimitive::get_num8::<u64>(&thread.memory())?;

                let vec_heap_address_bytes = thread
                    .env
                    .stack
                    .read(
                        Offset::SB(vec_stack_address as usize),
                        AccessLevel::Direct,
                        8,
                    )
                    .map_err(|err| err.into())?;

                let vec_heap_address_bytes =
                    TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);
                let previous_len_bytes = thread
                    .runtime
                    .heap
                    .read(vec_heap_address as usize, 8)
                    .map_err(|e| e.into())?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let item_size = *item_size as u64;

                /* clear */
                let _ = thread
                    .runtime
                    .heap
                    .write(
                        vec_heap_address as usize + 16,
                        &vec![0; (previous_len * item_size) as usize],
                    )
                    .map_err(|e| e.into())?;

                let len_bytes = 0u64.to_le_bytes().as_slice().to_vec();
                /* Write len */
                let _ = thread
                    .runtime
                    .heap
                    .write(vec_heap_address as usize, &len_bytes)
                    .map_err(|e| e.into())?;
            }
            AllocCasm::ClearMap(_) => todo!(),
        }

        thread.env.program.incr();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            expressions::data::{Number, Primitive},
            statements::Statement,
            TryParse,
        },
        clear_stack,
        semantic::scope::scope_impl::Scope,
        v_num,
        vm::vm::{DeserializeFrom, Runtime},
    };

    use super::*;

    #[test]
    fn valid_string() {
        let statement = Statement::parse(
            r##"
        let x = string("Hello World");
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data = memory
            .heap
            .read(heap_address, length + 16)
            .expect("length should be readable");

        let result = <StringType as DeserializeFrom>::deserialize_from(&StringType(), &data)
            .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "Hello World");
    }

    #[test]
    fn valid_vec() {
        let statement = Statement::parse(
            r##"
        let x:Vec<u64> = vec(8);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 8);
    }

    #[test]
    fn valid_vec_modify() {
        let statement = Statement::parse(
            r##"
            let res = {
                let arr : Vec<u64> = vec(8);

                arr[2] = 420;
                
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
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 420));
    }
    #[test]
    fn valid_append() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
                append(&x,9);
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 9);

        let data = memory
            .heap
            .read(heap_address as usize, 8 * 9 + 16)
            .expect("Heap Read should have succeeded");
        let data: Vec<u64> = data
            .chunks(8)
            .map(|chunk| {
                u64::from_le_bytes(
                    TryInto::<[u8; 8]>::try_into(&chunk[0..8])
                        .expect("heap address should be deserializable"),
                )
            })
            .collect();
        assert_eq!(data, vec![9, 24, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn valid_append_no_realloc() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x:Vec<u64> = vec(8,16);
                append(&x,9);
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 9);

        let data = memory
            .heap
            .read(heap_address as usize, 8 * 9 + 16)
            .expect("Heap Read should have succeeded");
        let data: Vec<u64> = data
            .chunks(8)
            .map(|chunk| {
                u64::from_le_bytes(
                    TryInto::<[u8; 8]>::try_into(&chunk[0..8])
                        .expect("heap address should be deserializable"),
                )
            })
            .collect();
        assert_eq!(data, vec![9, 16, 0, 0, 0, 0, 0, 0, 0, 0, 9]);
    }

    #[test]
    fn valid_append_str_slice() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x = string("Hello ");
                append(&x,"World");
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data = memory
            .heap
            .read(heap_address, length + 16)
            .expect("length should be readable");

        let result = <StringType as DeserializeFrom>::deserialize_from(&StringType(), &data)
            .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "Hello World");
    }

    #[test]
    fn valid_append_str_char() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x = string("Hello Worl");
                append(&x,'d');
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data = memory
            .heap
            .read(heap_address, length + 16)
            .expect("length should be readable");

        let result = <StringType as DeserializeFrom>::deserialize_from(&StringType(), &data)
            .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "Hello World");
    }

    #[test]
    fn valid_append_str_char_complex() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x = string("Hello Worl");
                append(&x,'仗');
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data = memory
            .heap
            .read(heap_address, length + 16)
            .expect("length should be readable");

        let result = <StringType as DeserializeFrom>::deserialize_from(&StringType(), &data)
            .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "Hello Worl仗");
    }

    #[test]
    fn valid_append_str_str() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x = string("Hello ");
                let y = string("World");
                append(&x,y);
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data = memory
            .heap
            .read(heap_address, length + 16)
            .expect("length should be readable");

        let result = <StringType as DeserializeFrom>::deserialize_from(&StringType(), &data)
            .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "Hello World");
    }

    #[test]
    fn valid_len_string() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x = string("Hello World");
                return len(x);
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 11));
    }

    #[test]
    fn valid_cap_string() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x = string("Hello World");
                return cap(x);
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 16));
    }

    #[test]
    fn valid_len_vec() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x : Vec<u64> = vec(11,16);
                return len(x);
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 11));
    }

    #[test]
    fn valid_cap_vec() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x : Vec<u64> = vec(11,16);
                return cap(x);
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 16));
    }
    #[test]
    fn valid_free() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x = string("Hello ");
                free(&x);
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);

        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        assert_eq!(memory.heap.allocated_size(), 0);
    }

    #[test]
    fn valid_alloc() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x = alloc(8) as &u64;
                *x = 420;
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);

        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        assert_eq!(memory.heap.allocated_size(), 16);

        let data = clear_stack!(memory);
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let data = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(data, 420);
    }

    #[test]
    fn valid_delete_vec() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
                delete(&x,7);
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 7);

        let data = memory
            .heap
            .read(heap_address as usize, 8 * length + 16)
            .expect("Heap Read should have succeeded");
        let data: Vec<u64> = data
            .chunks(8)
            .map(|chunk| {
                u64::from_le_bytes(
                    TryInto::<[u8; 8]>::try_into(&chunk[0..8])
                        .expect("heap address should be deserializable"),
                )
            })
            .collect();
        assert_eq!(data, vec![7, 8, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn valid_delete_vec_inner() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
                delete(&x,2);
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 7);

        let data = memory
            .heap
            .read(heap_address as usize, 8 * length + 16)
            .expect("Heap Read should have succeeded");
        let data: Vec<u64> = data
            .chunks(8)
            .map(|chunk| {
                u64::from_le_bytes(
                    TryInto::<[u8; 8]>::try_into(&chunk[0..8])
                        .expect("heap address should be deserializable"),
                )
            })
            .collect();
        assert_eq!(data, vec![7, 8, 1, 2, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn valid_sizeof_type() {
        let statement = Statement::parse(
            r##"
            let res = {
                struct Point {
                    x : u64,
                    y : u64,
                }
                return sizeof(Point);
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 16));
    }

    #[test]
    fn valid_sizeof_expr() {
        let statement = Statement::parse(
            r##"
            let res = sizeof(420u64);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 8));
    }

    #[test]
    fn valid_memcpy_heap() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
                let y:Vec<u64> = vec(8);
                memcpy(y,x,8*8 + 16);
                return y;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 8);

        let data = memory
            .heap
            .read(heap_address as usize, 8 * length + 16)
            .expect("Heap Read should have succeeded");
        let data: Vec<u64> = data
            .chunks(8)
            .map(|chunk| {
                u64::from_le_bytes(
                    TryInto::<[u8; 8]>::try_into(&chunk[0..8])
                        .expect("heap address should be deserializable"),
                )
            })
            .collect();
        assert_eq!(data, vec![8, 8, 1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn valid_memcpy_stack() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x:[8]u64 = [1,2,3,4,5,6,7,8];
                let y:[8]u64 = [0,0,0,0,0,0,0,0];
                memcpy(&y,&x,8*8);
                return y;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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

        let data: Vec<u64> = data
            .chunks(8)
            .map(|chunk| {
                u64::from_le_bytes(
                    TryInto::<[u8; 8]>::try_into(&chunk[0..8])
                        .expect("heap address should be deserializable"),
                )
            })
            .collect();
        assert_eq!(data, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn valid_clear_vec() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
                clear(&x);
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 0);

        let data = memory
            .heap
            .read(heap_address as usize, 8 * 8 + 16)
            .expect("Heap Read should have succeeded");
        let data: Vec<u64> = data
            .chunks(8)
            .map(|chunk| {
                u64::from_le_bytes(
                    TryInto::<[u8; 8]>::try_into(&chunk[0..8])
                        .expect("heap address should be deserializable"),
                )
            })
            .collect();
        assert_eq!(data, vec![0, 8, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn valid_clear_str() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x = string("Hello ");
                clear(&x);
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data = memory
            .heap
            .read(heap_address, 5 + 16)
            .expect("length should be readable");

        let result = <StringType as DeserializeFrom>::deserialize_from(&StringType(), &data)
            .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "");
    }

    #[test]
    fn valid_alloc_cast() {
        let statement = Statement::parse(
            r##"
            let res = {
                struct Point {
                    x : i64,
                    y: i64
                }

                let point_any : &Any = alloc(sizeof(Point)) as &Any;

                let copy = Point {
                    x : 420,
                    y : 69,
                };
                memcpy(point_any,&copy,sizeof(Point));
                
                let point = *point_any as Point;
                
                return point.x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 420));
    }

    #[test]
    fn valid_alloc_modify() {
        let statement = Statement::parse(
            r##"
            let res = {
                struct Point {
                    x : i64,
                    y: i64
                }

                let point : &Point = alloc(sizeof(Point)) as &Point;

                point.x = 420;
                
                return point.x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 420));
    }
}
