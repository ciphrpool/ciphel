use std::{
    cell::{Cell, Ref},
    vec,
};

use nom_supreme::parser_ext::Value;
use num_traits::ToBytes;

use crate::{
    ast::expressions::Expression,
    semantic::{
        scope::{
            static_types::{
                self, AddrType, NumberType, PrimitiveType, StaticType, StringType, VecType,
            },
            type_traits::{GetSubTypes, TypeChecking},
            ScopeApi,
        },
        AccessLevel, EType, Either, Info, Metadata, MutRc, Resolve, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        allocator::{align, stack::Offset, MemoryAddress},
        casm::{
            alloc::{Access, Alloc, Free, Realloc},
            branch::{BranchIf, Goto, Label},
            memcopy::MemCopy,
            operation::{
                Addition, Equal, Greater, Mult, OpPrimitive, Operation, OperationKind, Substraction,
            },
            serialize::Serialized,
            Casm, CasmProgram,
        },
        platform::{utils::lexem, LibCasm},
        scheduler::Thread,
        vm::{CodeGenerationError, Executable, GenerateCode, RuntimeError},
    },
};

use super::thread::ThreadFn;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AppendKind {
    Vec,
    StrSlice,
    Char,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AllocFn {
    Append {
        item_size: Cell<usize>,
        append_kind: Cell<AppendKind>,
    },
    Insert,
    Delete,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum AllocCasm {
    AppendChar,
    AppendItem(usize),
    AppendStrSlice(usize),
    AppendString,
    Insert,
    Delete,
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
    pub fn from(id: &String) -> Option<Self> {
        match id.as_str() {
            lexem::APPEND => Some(AllocFn::Append {
                item_size: Cell::new(0),
                append_kind: Cell::new(AppendKind::Vec),
            }),
            lexem::INSERT => Some(AllocFn::Insert),
            lexem::DELETE => Some(AllocFn::Delete),
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
            _ => None,
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for AllocFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression<Scope>>;
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
            AllocFn::Delete => todo!(),
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
                    let _ = param.resolve(
                        scope,
                        &Some(Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                        )),
                        &(),
                    )?;
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

                let _ = size.resolve(
                    scope,
                    &Some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    )),
                    &(),
                )?;
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
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for AllocFn {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            AllocFn::Append { .. } => Ok(Either::Static(StaticType::Unit.into())),
            AllocFn::Insert => todo!(),
            AllocFn::Delete => todo!(),
            AllocFn::Free => Ok(Either::Static(StaticType::Unit.into())),
            AllocFn::Vec { metadata, .. } => {
                metadata.signature().ok_or(SemanticError::NotResolvedYet)
            }
            AllocFn::Map => todo!(),
            AllocFn::Chan => todo!(),
            AllocFn::String { .. } => Ok(Either::Static(StaticType::String(StringType()).into())),
            AllocFn::Alloc => Ok(Either::Static(StaticType::Any.into())),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for AllocFn {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
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
            AllocFn::Delete => instructions.push(Casm::Platform(LibCasm::Core(
                super::CoreCasm::Alloc(AllocCasm::Delete),
            ))),
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
            AllocCasm::Delete => todo!(),
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
        }

        thread.env.program.incr();
        Ok(())
    }
}

fn gencode_append_string_char<Scope: ScopeApi>(
    scope: &MutRc<Scope>,
    instructions: &CasmProgram,
    item_size: usize,
) -> Result<(), CodeGenerationError> {
    /* STATCK : STACK_ADDR | CHAR(4) */
    /* Convert Char to UTF8 */
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::CastCharToUTF8,
    }));

    /* STATCK : STACK_ADDR | CharUTF8 | LEN */

    todo!();
}
fn gencode_append_string_string<Scope: ScopeApi>(
    scope: &MutRc<Scope>,
    instructions: &CasmProgram,
    _item_size: usize,
) -> Result<(), CodeGenerationError> {
    let else_label = Label::gen();
    let end_label = Label::gen();

    /* STATCK : STACK_ADDR | ITEM_ADDR */
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    /* STATCK : STACK_ADDR | ITEM_ADDR | ITEM_ADDR */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | ITEM_LEN */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 3),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | ITEM_LEN | STACK_ADDR */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | ITEM_LEN | HEAP_ADDR */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | ITEM_LEN | STR_LEN */
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | ITEM_LEN + STR_LEN */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 3),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | ITEM_LEN + STR_LEN | STACK_ADDR */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | ITEM_LEN + STR_LEN | HEAP_ADDR */
    instructions.push(Casm::Serialize(Serialized {
        data: (&8u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | ITEM_LEN + STR_LEN | CAP */
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Greater(Greater {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::If(BranchIf { else_label }));

    /* STATCK : STACK_ADDR | ITEM_ADDR  */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 2),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | STACK_ADDR */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 2),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR |ITEM_LEN */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 2),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_LEN | HEAP_ADDR */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_LEN | STR_LEN */
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&(2u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Align,
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&16u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | (ITEM_LEN + STR_LEN)*2 +16 */
    instructions.push(Casm::Realloc(Realloc { size: None }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 3),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | STACK_ADDR */
    instructions.push(Casm::MemCopy(MemCopy::Take { size: 8 }));
    /* STATCK : STACK_ADDR | ITEM_ADDR */
    /* START Update capacity */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 2),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | STACK_ADDR*/
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR  */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 2),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR*/
    instructions.push(Casm::MemCopy(MemCopy::Dup(16)));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR*/
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR  | HEAP_ADDR |ITEM_LEN */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 2),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_LEN | HEAP_ADDR */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_LEN | STR_LEN */
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&(2u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Align,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | HEAP_ADDR | (ITEM_LEN + STR_LEN)*2 */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 2),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | HEAP_ADDR | (ITEM_LEN + STR_LEN)*2 | HEAP_ADDR */
    instructions.push(Casm::Serialize(Serialized {
        data: (&8u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STACK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | HEAP_ADDR | (ITEM_LEN + STR_LEN)*2 | HEAP_ADDR+8 */
    instructions.push(Casm::MemCopy(MemCopy::Take { size: 8 }));
    /* END Update capacity */
    /* STACK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | HEAP_ADDR | HEAP_ADDR+8 */
    instructions.push(Casm::Pop(8 * 3));
    /* STACK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR */
    instructions.push(Casm::Goto(Goto {
        label: Some(end_label),
    }));
    /* STATCK : HEAP_ADDR | ITEM_ADDR | HEAP_ADDR  */
    instructions.push_label_id(else_label, "append_no_realloc".into());
    /* STATCK : STACK_ADDR | ITEM_ADDR */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 2),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | STACK_ADDR */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : HEAP_ADDR | ITEM_ADDR | HEAP_ADDR  */
    instructions.push_label_id(end_label, "end_append".into());

    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR  */
    /* START Update Length */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 2),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR*/
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | ITEM_ADDR*/
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR  | ITEM_LEN | ITEM_LEN */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 4),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | ITEM_LEN | ITEM_LEN | HEAP_ADDR */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR |  ITEM_LEN | ITEM_LEN | STR_LEN */
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR |  ITEM_LEN  |  ITEM_LEN + STR_LEN */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 4),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR |  ITEM_LEN  |  ITEM_LEN + STR_LEN | HEAP_ADDR*/
    instructions.push(Casm::MemCopy(MemCopy::Take { size: 8 }));
    /* END Update Length */
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | ITEM_LEN  | HEAP_ADDR*/
    instructions.push(Casm::Serialize(Serialized {
        data: (&16u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
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
    instructions.push(Casm::Serialize(Serialized {
        data: (&1u64.to_le_bytes()).to_vec(), /* Offset by 1 to reach the next index */
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | index */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-8 * 2),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | index | ITEM_ADDR */
    instructions.push(Casm::Serialize(Serialized {
        data: (&16u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | index | ITEM_ADDR +16 */
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | index | ITEM_ADDR +16 | ITEM_ADDR +16 */
    instructions.push(Casm::Serialize(Serialized {
        data: (&16u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Substraction(Substraction {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | index | ITEM_ADDR+16| ITEM_ADDR  */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR | index | ITEM_ADDR+16 | ITEM_LEN  */
    instructions.push(Casm::MemCopy(MemCopy::MemCopy));
    /* STATCK : STACK_ADDR | ITEM_ADDR | HEAP_ADDR | ITEM_ADDR  */
    instructions.push(Casm::Pop(8 * 4));
    /* STATCK :  */
    Ok(())
}
fn gencode_append_string_slice<Scope: ScopeApi>(
    scope: &MutRc<Scope>,
    instructions: &CasmProgram,
    item_size: usize,
) -> Result<(), CodeGenerationError> {
    let else_label = Label::gen();
    let end_label = Label::gen();

    // Retrieve the heap_address of the vector
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-((item_size + 8) as isize)),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));

    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR |  HEAP_ADDR | HEAP_ADDR */
    // Retrieve the length of the vector
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR |  HEAP_ADDR | LENGTH */
    instructions.push(Casm::Serialize(Serialized {
        data: (&(item_size as u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-16),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR | HEAP_ADDR | LENGTH | HEAP_ADRR*/
    instructions.push(Casm::Serialize(Serialized {
        data: (&(8u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR | HEAP_ADDR | LENGTH | HEAP_ADRR+8*/
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR | HEAP_ADDR | LENGTH | CAP*/
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Greater(Greater {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::If(BranchIf { else_label }));
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR | HEAP_ADDR |*/
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR | LENGTH */
    instructions.push(Casm::Serialize(Serialized {
        data: (&(item_size as u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&(2u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Align,
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&16u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR | LENGTH */
    instructions.push(Casm::Realloc(Realloc { size: None }));
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-((item_size + 8 * 2) as isize)),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR | STACK_ADDR */
    instructions.push(Casm::MemCopy(MemCopy::Take { size: 8 }));
    /* STATCK : STACK_ADDR | SLICE | */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-((item_size + 8) as isize)),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR  */
    /* START Update Capacity */
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR */
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));

    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | HEAP_ADDR */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | Length */
    instructions.push(Casm::Serialize(Serialized {
        data: (&(item_size as u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&(2u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Align,
    }));
    /* update capacity */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-16),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&8u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | Length | HEAP_ADDR+8 */
    instructions.push(Casm::MemCopy(MemCopy::Take { size: 8 }));
    /* END Update Capacity */
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | HEAP_ADDR+8*/
    instructions.push(Casm::Pop(16));
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR  */
    instructions.push(Casm::Goto(Goto {
        label: Some(end_label),
    }));
    instructions.push_label_id(else_label, "append_no_realloc".into());
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR | HEAP_ADDR |*/
    instructions.push(Casm::Pop(8));
    /* STATCK : STACK_ADDR | SLICE | HEAP_ADDR |*/
    instructions.push_label_id(end_label, "end_append".into());

    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR |  */

    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | HEAP_ADDR  */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));

    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | LENGTH */
    instructions.push(Casm::Serialize(Serialized {
        data: (&(item_size as u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | LENGTH */
    /* update length */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-16),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | LENGTH | HEAP_ADDR */
    instructions.push(Casm::MemCopy(MemCopy::Take { size: 8 }));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | HEAP_ADDR  */
    instructions.push(Casm::Pop(8));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR   */

    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));

    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR  | LENGTH */
    instructions.push(Casm::Serialize(Serialized {
        data: (&(item_size as u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Substraction(Substraction {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR  | Index */
    instructions.push(Casm::Serialize(Serialized {
        data: (&16u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
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
    /* STACK : STACK_ADDR | ITEM | ITEM_HEAP_ADDR*/
    instructions.push(Casm::MemCopy(MemCopy::Take { size: item_size }));
    /* STACK : STACK_ADDR |*/
    instructions.push(Casm::Pop(8));

    Ok(())
}

fn gencode_append_vec<Scope: ScopeApi>(
    scope: &MutRc<Scope>,
    instructions: &CasmProgram,
    item_size: usize,
) -> Result<(), CodeGenerationError> {
    let else_label = Label::gen();
    let end_label = Label::gen();

    // Retrieve the heap_address of the vector
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-((item_size + 8) as isize)),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    // Retrieve the length of the vector
    instructions.push(Casm::Access(Access::Runtime { size: Some(16) }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Equal(Equal { left: 8, right: 8 }),
    }));
    instructions.push(Casm::If(BranchIf { else_label }));
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&(2u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&(item_size as u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Align,
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&16u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Realloc(Realloc { size: None }));

    /* STATCK : STACK_ADDR | ITEM | HEAP_ADDR */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-((item_size + 8 * 2) as isize)),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STATCK : STACK_ADDR | ITEM | HEAP_ADDR | STACK_ADDR */
    instructions.push(Casm::MemCopy(MemCopy::Take { size: 8 }));
    /* STATCK : STACK_ADDR | ITEM | */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-((item_size + 8) as isize)),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));

    /* START Update capacity */
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR */
    instructions.push(Casm::Serialize(Serialized {
        data: (&8u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR+8 */

    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));

    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR+8 | HEAP_ADDR+8 */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR+8 | Capacity */
    instructions.push(Casm::Serialize(Serialized {
        data: (&(2u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Align,
    }));
    /* update capacity */
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR+8 | Capacity | HEAP_ADDR+8 */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-16),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    instructions.push(Casm::MemCopy(MemCopy::Take { size: 8 }));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR+8 | HEAP_ADDR+8*/
    instructions.push(Casm::Pop(16));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR |  */
    /* END Update Capacity */
    instructions.push(Casm::Goto(Goto {
        label: Some(end_label),
    }));
    instructions.push_label_id(else_label, "append_no_realloc".into());
    instructions.push(Casm::Pop(8));
    instructions.push_label_id(end_label, "end_append".into());

    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR |  */
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | HEAP_ADDR  */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));

    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | LENGTH */
    instructions.push(Casm::Serialize(Serialized {
        data: (&1u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* update length */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-16),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | LENGTH | HEAP_ADDR */
    instructions.push(Casm::MemCopy(MemCopy::Take { size: 8 }));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | HEAP_ADDR  */
    instructions.push(Casm::Pop(8));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR   */

    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));

    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR  | LENGTH */
    instructions.push(Casm::Serialize(Serialized {
        data: (&1u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Substraction(Substraction {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STACK : STACK_ADDR | ITEM | HEAP_ADDR  | Index */
    instructions.push(Casm::Serialize(Serialized {
        data: (&(item_size as u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&16u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
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
    /* STACK : STACK_ADDR | ITEM | ITEM_HEAP_ADDR*/
    instructions.push(Casm::MemCopy(MemCopy::Take { size: item_size }));
    /* STACK : STACK_ADDR |*/
    instructions.push(Casm::Pop(8));

    Ok(())
}

fn gencode_vec<Scope: ScopeApi>(
    scope: &MutRc<Scope>,
    instructions: &CasmProgram,
    item_size: usize,
    with_capacity: bool,
) -> Result<(), CodeGenerationError> {
    if !with_capacity {
        instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Align,
        }))
    }
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    instructions.push(Casm::Serialize(Serialized {
        data: (&(item_size as u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));

    instructions.push(Casm::Serialize(Serialized {
        data: (&16u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Alloc(Alloc::Heap { size: None }));
    instructions.push(Casm::MemCopy(MemCopy::TakeToHeap {
        //offset: vec_stack_address + 8,
        size: 16,
    }));

    Ok(())
}

fn gencode_string<Scope: ScopeApi>(
    scope: &MutRc<Scope>,
    instructions: &CasmProgram,
    params_size: usize,
) -> Result<(), CodeGenerationError> {
    let len_bytes = (params_size as u64).to_le_bytes().as_slice().to_vec();
    let cap_bytes = (align(params_size) as u64)
        .to_le_bytes()
        .as_slice()
        .to_vec();

    // Push Length on stack
    instructions.push(Casm::Serialize(Serialized { data: len_bytes }));
    // Push Capacity on stack
    instructions.push(Casm::Serialize(Serialized { data: cap_bytes }));

    // Copy data on stack to heap at address
    // Alloc and push heap address on stack
    instructions.push(Casm::Alloc(Alloc::Heap {
        size: Some(align(params_size + 16)),
    }));
    // Take the address on the top of the stack
    // and copy the data on the stack in the heap at given address and given offset
    // ( removing the data from the stack ) and put back the address on top of the stack
    instructions.push(Casm::MemCopy(MemCopy::TakeToHeap {
        //offset: vec_stack_address + 8,
        size: 16,
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (16u64).to_le_bytes().to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));

    instructions.push(Casm::MemCopy(MemCopy::TakeToHeap {
        //offset: vec_stack_address + 8,
        size: params_size,
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (16u64).to_le_bytes().to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Substraction(Substraction {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{statements::Statement, TryParse},
        clear_stack,
        semantic::scope::scope_impl::Scope,
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

        let result = <StringType as DeserializeFrom<Scope>>::deserialize_from(&StringType(), &data)
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

        let result = <StringType as DeserializeFrom<Scope>>::deserialize_from(&StringType(), &data)
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

        let result = <StringType as DeserializeFrom<Scope>>::deserialize_from(&StringType(), &data)
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

        let result = <StringType as DeserializeFrom<Scope>>::deserialize_from(&StringType(), &data)
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
        dbg!(&heap_address);

        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        dbg!(length);
        let data = memory
            .heap
            .read(heap_address, length + 16)
            .expect("length should be readable");

        let result = <StringType as DeserializeFrom<Scope>>::deserialize_from(&StringType(), &data)
            .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "Hello World");
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
}
