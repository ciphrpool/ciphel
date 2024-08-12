use std::vec;

use crate::{
    ast::utils::strings::ID,
    err_tuple,
    semantic::scope::{
        scope::Scope,
        static_types::{MapType, SliceType},
        type_traits::TypeChecking,
    },
    vm::{
        allocator::{heap::Heap, stack::Stack},
        platform::{
            core::alloc::map_impl::{bucket_idx, bucket_layout, hash_of, map_layout, top_hash},
            stdlib::{ERROR_VALUE, OK_VALUE},
        },
        stdio::StdIO,
        vm::CasmMetadata,
    },
};

use num_traits::ToBytes;

use crate::{
    ast::expressions::Expression,
    e_static, p_num,
    semantic::{
        scope::{
            static_types::{AddrType, NumberType, PrimitiveType, StaticType, StringType, VecType},
            type_traits::GetSubTypes,
        },
        AccessLevel, EType, Either, Info, Metadata, Resolve, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        allocator::{align, stack::Offset},
        casm::{
            alloc::{Access, Alloc, Free},
            data::Data,
            mem::Mem,
            operation::OpPrimitive,
            Casm, CasmProgram,
        },
        platform::{utils::lexem, LibCasm},
        vm::{CodeGenerationError, Executable, GenerateCode, RuntimeError},
    },
};

use self::map_impl::{over_load_factor, MapLayout, MAP_BUCKET_SIZE};

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AppendKind {
    Vec,
    StrSlice,
    Char,
    String,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ExtendKind {
    VecFromSlice(usize),
    VecFromVec,
    StringFromSlice(usize),
    StringFromVec,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum DeleteKind {
    Vec,
    Map(DerefHashing),
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
        item_size: usize,
        append_kind: AppendKind,
    },
    Extend {
        item_size: usize,
        extend_kind: ExtendKind,
    },
    Insert {
        key_size: usize,
        value_size: usize,
        ref_access: DerefHashing,
    },
    Get {
        key_size: usize,
        value_size: usize,
        ref_access: DerefHashing,
        metadata: Metadata,
    },
    Delete {
        key_size: usize,
        value_size: usize,
        delete_kind: DeleteKind,
        metadata: Metadata,
    },
    Len,
    Cap {
        for_map: bool,
    },
    Free,
    Alloc,
    Vec {
        with_capacity: bool,
        item_size: usize,
        metadata: Metadata,
    },
    Map {
        with_capacity: bool,
        value_size: usize,
        key_size: usize,
        metadata: Metadata,
    },
    String {
        len: usize,
        from_char: bool,
    },

    SizeOf {
        size: usize,
    },

    MemCopy,
    Clear {
        item_size: usize,
        key_size: usize,
        clear_kind: ClearKind,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum DerefHashing {
    Vec(usize),
    String,
    Default,
}

impl From<&Either> for DerefHashing {
    fn from(value: &Either) -> Self {
        match value {
            Either::Static(tmp) => match tmp.as_ref() {
                StaticType::String(_) => DerefHashing::String,
                StaticType::Vec(VecType(item_subtype)) => DerefHashing::Vec(item_subtype.size_of()),
                _ => DerefHashing::Default,
            },
            Either::User(_) => DerefHashing::Default,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AllocCasm {
    AppendChar,
    AppendItem(usize),
    AppendStrSlice(usize),
    AppendString,

    ExtendItemFromSlice {
        size: usize,
        len: usize,
    },
    ExtendItemFromVec {
        size: usize,
    },
    ExtendStringFromSlice {
        len: usize,
    },
    ExtendStringFromVec,

    Insert {
        ref_access: DerefHashing,
        key_size: usize,
        value_size: usize,
    },
    InsertAndForward {
        ref_access: DerefHashing,
        key_size: usize,
        value_size: usize,
    },
    Get {
        ref_access: DerefHashing,
        key_size: usize,
        value_size: usize,
    },
    DeleteVec(usize),
    DeleteMapKey {
        ref_access: DerefHashing,
        key_size: usize,
        value_size: usize,
    },

    ClearVec(usize),
    ClearString(usize),
    ClearMap {
        key_size: usize,
        value_size: usize,
    },

    Len,
    Cap,
    CapMap,
    Vec {
        item_size: usize,
    },
    VecWithCapacity {
        item_size: usize,
    },
    Map {
        key_size: usize,
        value_size: usize,
    },
    MapWithCapacity {
        key_size: usize,
        value_size: usize,
    },
    StringFromSlice,
    StringFromChar,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for AllocCasm {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            AllocCasm::AppendChar => stdio.push_casm_lib(engine, "append"),
            AllocCasm::AppendItem(_) => stdio.push_casm_lib(engine, "append"),
            AllocCasm::AppendStrSlice(_) => stdio.push_casm_lib(engine, "append"),
            AllocCasm::AppendString => stdio.push_casm_lib(engine, "append"),
            AllocCasm::ExtendItemFromSlice { size, len } => stdio.push_casm_lib(engine, "extend"),
            AllocCasm::ExtendItemFromVec { size } => stdio.push_casm_lib(engine, "extend"),
            AllocCasm::ExtendStringFromSlice { len } => stdio.push_casm_lib(engine, "extend"),
            AllocCasm::ExtendStringFromVec => stdio.push_casm_lib(engine, "extend"),
            AllocCasm::Insert {
                ref_access,
                key_size,
                value_size,
            } => stdio.push_casm_lib(engine, "insert"),
            AllocCasm::InsertAndForward {
                ref_access,
                key_size,
                value_size,
            } => stdio.push_casm_lib(engine, "finsert"),
            AllocCasm::Get {
                ref_access,
                key_size,
                value_size,
            } => stdio.push_casm_lib(engine, "get"),
            AllocCasm::DeleteVec(_) => stdio.push_casm_lib(engine, "delete"),
            AllocCasm::DeleteMapKey {
                ref_access,
                key_size,
                value_size,
            } => stdio.push_casm_lib(engine, "delete"),
            AllocCasm::ClearVec(_) => stdio.push_casm_lib(engine, "clear"),
            AllocCasm::ClearString(_) => stdio.push_casm_lib(engine, "clear"),
            AllocCasm::ClearMap { .. } => stdio.push_casm_lib(engine, "clear"),
            AllocCasm::Len => stdio.push_casm_lib(engine, "len"),
            AllocCasm::Cap => stdio.push_casm_lib(engine, "cap"),
            AllocCasm::CapMap => stdio.push_casm_lib(engine, "cap"),
            AllocCasm::Vec { item_size } => stdio.push_casm_lib(engine, "vec"),
            AllocCasm::VecWithCapacity { item_size } => stdio.push_casm_lib(engine, "vec"),
            AllocCasm::Map {
                key_size,
                value_size,
            } => stdio.push_casm_lib(engine, "map"),
            AllocCasm::MapWithCapacity {
                key_size,
                value_size,
            } => stdio.push_casm_lib(engine, "map"),
            AllocCasm::StringFromSlice => stdio.push_casm_lib(engine, "string"),
            AllocCasm::StringFromChar => stdio.push_casm_lib(engine, "string"),
        }
    }
}

impl AllocFn {
    pub fn from(suffixe: &Option<ID>, id: &ID) -> Option<Self> {
        match suffixe {
            Some(suffixe) => {
                if **suffixe != lexem::CORE {
                    return None;
                }
            }
            None => {}
        }

        match id.as_str() {
            lexem::APPEND => Some(AllocFn::Append {
                item_size: (0),
                append_kind: (AppendKind::Vec),
            }),
            lexem::EXTEND => Some(AllocFn::Extend {
                item_size: 0,
                extend_kind: ExtendKind::VecFromVec,
            }),
            lexem::INSERT => Some(AllocFn::Insert {
                key_size: 0,
                value_size: 0,
                ref_access: DerefHashing::Default,
            }),
            lexem::GET => Some(AllocFn::Get {
                key_size: 0,
                value_size: 0,
                metadata: Metadata::default(),
                ref_access: DerefHashing::Default,
            }),
            lexem::DELETE => Some(AllocFn::Delete {
                key_size: 0,
                value_size: 0,
                delete_kind: DeleteKind::Vec,
                metadata: Metadata::default(),
            }),
            lexem::LEN => Some(AllocFn::Len),
            lexem::CAP => Some(AllocFn::Cap { for_map: false }),
            lexem::FREE => Some(AllocFn::Free),
            lexem::VEC => Some(AllocFn::Vec {
                with_capacity: false,
                item_size: 0,
                metadata: Metadata::default(),
            }),
            lexem::MAP => Some(AllocFn::Map {
                with_capacity: false,
                key_size: 0,
                value_size: 0,
                metadata: Metadata::default(),
            }),
            lexem::STRING => Some(AllocFn::String {
                len: 0,
                from_char: false,
            }),
            lexem::ALLOC => Some(AllocFn::Alloc),
            lexem::MEMCPY => Some(AllocFn::MemCopy),
            lexem::CLEAR => Some(AllocFn::Clear {
                item_size: 0,
                key_size: 0,
                clear_kind: ClearKind::Vec,
            }),
            lexem::SIZEOF => Some(AllocFn::SizeOf { size: 0 }),
            _ => None,
        }
    }
}

impl Resolve for AllocFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression>;
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            AllocFn::Append {
                item_size,
                append_kind,
            } => {
                if extra.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, second_part) = extra.split_at_mut(1);
                let vector = &mut first_part[0];
                let item = &mut second_part[0];

                let _ = vector.resolve::<G>(scope, &None, &mut None)?;
                let mut vector_type =
                    vector.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
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
                            *append_kind = AppendKind::Vec;
                            let _ = item.resolve::<G>(scope, &item_type, &mut None)?;
                            let Some(item_type) = item_type else {
                                return Err(SemanticError::IncorrectArguments);
                            };
                            *item_size = item_type.size_of();
                            Ok(())
                        }
                        StaticType::String(_) => {
                            let _ = item.resolve::<G>(scope, &None, &mut None)?;
                            let item_type = item.type_of(&crate::arw_read!(
                                scope,
                                SemanticError::ConcurrencyError
                            )?)?;
                            match &item_type {
                                Either::Static(value) => match value.as_ref() {
                                    StaticType::Primitive(PrimitiveType::Char) => {
                                        *append_kind = AppendKind::Char;
                                    }
                                    StaticType::String(_) => {
                                        *append_kind = AppendKind::String;
                                    }
                                    StaticType::StrSlice(_) => {
                                        *append_kind = AppendKind::StrSlice;
                                    }
                                    _ => return Err(SemanticError::IncorrectArguments),
                                },
                                _ => return Err(SemanticError::IncorrectArguments),
                            }
                            *item_size = item_type.size_of();
                            Ok(())
                        }
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
            }
            AllocFn::Extend {
                item_size,
                extend_kind,
            } => {
                if extra.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let (first_part, second_part) = extra.split_at_mut(1);
                let vector = &mut first_part[0];
                let items = &mut second_part[0];

                let _ = vector.resolve::<G>(scope, &None, &mut None)?;
                let mut vector_type =
                    vector.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
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
                            let _ = items.resolve::<G>(scope, &None, &mut None)?;
                            let items_type = items.type_of(&crate::arw_read!(
                                scope,
                                SemanticError::ConcurrencyError
                            )?)?;

                            match items_type {
                                Either::Static(value) => match value.as_ref() {
                                    StaticType::Slice(SliceType {
                                        size: len,
                                        item_type,
                                    }) => {
                                        *extend_kind = ExtendKind::VecFromSlice(*len);

                                        *item_size = item_type.size_of();
                                        Ok(())
                                    }
                                    StaticType::Vec(VecType(item_type)) => {
                                        *extend_kind = ExtendKind::VecFromVec;
                                        *item_size = item_type.size_of();
                                        Ok(())
                                    }
                                    _ => return Err(SemanticError::IncorrectArguments),
                                },
                                _ => return Err(SemanticError::IncorrectArguments),
                            }
                        }
                        StaticType::String(_) => {
                            let _ = items.resolve::<G>(scope, &None, &mut None)?;
                            let items_type = items.type_of(&crate::arw_read!(
                                scope,
                                SemanticError::ConcurrencyError
                            )?)?;

                            match items_type {
                                Either::Static(value) => match value.as_ref() {
                                    StaticType::Slice(SliceType {
                                        size: len,
                                        item_type,
                                    }) => {
                                        if !item_type.is_string() {
                                            return Err(SemanticError::IncorrectArguments);
                                        }
                                        *extend_kind = ExtendKind::StringFromSlice(*len);
                                        Ok(())
                                    }
                                    StaticType::Vec(VecType(item_type)) => {
                                        if !item_type.is_string() {
                                            return Err(SemanticError::IncorrectArguments);
                                        }
                                        *extend_kind = ExtendKind::StringFromVec;
                                        Ok(())
                                    }
                                    _ => return Err(SemanticError::IncorrectArguments),
                                },
                                _ => return Err(SemanticError::IncorrectArguments),
                            }
                        }
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
            }
            AllocFn::Insert {
                key_size,
                value_size,
                ref_access,
            } => {
                if extra.len() != 3 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let (first_part, rest) = extra.split_at_mut(1);
                let (second_part, third_part) = rest.split_at_mut(1);

                let map = &mut first_part[0];
                let key = &mut second_part[0];
                let item = &mut third_part[0];

                let _ = map.resolve::<G>(scope, &None, &mut None)?;
                let mut map_type =
                    map.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                match &map_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Address(AddrType(sub)) => map_type = sub.as_ref().clone(),
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                match &map_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Map(MapType {
                            keys_type,
                            values_type,
                        }) => {
                            let _ = key.resolve::<G>(
                                scope,
                                &Some(keys_type.as_ref().clone()),
                                &mut None,
                            )?;
                            let _ = item.resolve::<G>(
                                scope,
                                &Some(values_type.as_ref().clone()),
                                &mut None,
                            )?;

                            match keys_type.as_ref() {
                                Either::Static(tmp) => match tmp.as_ref() {
                                    StaticType::String(_) => *ref_access = DerefHashing::String,
                                    StaticType::Vec(VecType(item_subtype)) => {
                                        *ref_access = DerefHashing::Vec(item_subtype.size_of())
                                    }
                                    _ => {}
                                },
                                Either::User(_) => {}
                            }
                            *value_size = values_type.size_of();
                            *key_size = keys_type.size_of();
                            Ok(())
                        }
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
            }
            AllocFn::Get {
                key_size,
                value_size,
                ref_access,
                metadata,
            } => {
                if extra.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let (first_part, second_part) = extra.split_at_mut(1);
                let map = &mut first_part[0];
                let key = &mut second_part[0];

                let _ = map.resolve::<G>(scope, &None, &mut None)?;
                let mut map_type =
                    map.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                match &map_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Address(AddrType(sub)) => map_type = sub.as_ref().clone(),
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                match &map_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Map(MapType {
                            keys_type,
                            values_type,
                        }) => {
                            let _ = key.resolve::<G>(
                                scope,
                                &Some(keys_type.as_ref().clone()),
                                &mut None,
                            )?;
                            *value_size = values_type.size_of();
                            *key_size = keys_type.size_of();

                            match keys_type.as_ref() {
                                Either::Static(tmp) => match tmp.as_ref() {
                                    StaticType::String(_) => *ref_access = DerefHashing::String,
                                    StaticType::Vec(VecType(item_subtype)) => {
                                        *ref_access = DerefHashing::Vec(item_subtype.size_of())
                                    }
                                    _ => {}
                                },
                                Either::User(_) => {}
                            }
                            metadata.info = Info::Resolved {
                                context: context.clone(),
                                signature: Some(values_type.as_ref().clone()),
                            };
                            Ok(())
                        }
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
            }
            AllocFn::Delete {
                delete_kind,
                key_size,
                metadata,
                value_size,
            } => {
                if extra.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, second_part) = extra.split_at_mut(1);
                let vector = &mut first_part[0];
                let index = &mut second_part[0];

                let _ = vector.resolve::<G>(scope, &None, &mut None)?;
                let mut vector_type =
                    vector.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
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
                            let _ = index.resolve::<G>(scope, &Some(p_num!(U64)), &mut None)?;
                            let index_type = index.type_of(&crate::arw_read!(
                                scope,
                                SemanticError::ConcurrencyError
                            )?)?;
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
                            *delete_kind = DeleteKind::Vec;
                            let Some(item_type) = item_type else {
                                return Err(SemanticError::IncorrectArguments);
                            };
                            *value_size = item_type.size_of();
                            metadata.info = Info::Resolved {
                                context: context.clone(),
                                signature: Some(item_type),
                            };
                            Ok(())
                        }
                        StaticType::Map(MapType {
                            keys_type,
                            values_type,
                        }) => {
                            match keys_type.as_ref() {
                                Either::Static(tmp) => match tmp.as_ref() {
                                    StaticType::String(_) => {
                                        *delete_kind = DeleteKind::Map(DerefHashing::String)
                                    }
                                    StaticType::Vec(VecType(item_subtype)) => {
                                        *delete_kind = DeleteKind::Map(DerefHashing::Vec(
                                            item_subtype.size_of(),
                                        ))
                                    }
                                    _ => {
                                        *delete_kind = DeleteKind::Map(DerefHashing::Default);
                                    }
                                },
                                Either::User(_) => {
                                    *delete_kind = DeleteKind::Map(DerefHashing::Default);
                                }
                            }

                            let _ = index.resolve::<G>(
                                scope,
                                &Some(keys_type.as_ref().clone()),
                                &mut None,
                            )?;
                            *value_size = values_type.size_of();
                            *key_size = keys_type.size_of();
                            metadata.info = Info::Resolved {
                                context: context.clone(),
                                signature: Some(values_type.as_ref().clone()),
                            };
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

                let address = &mut extra[0];

                let _ = address.resolve::<G>(scope, &None, &mut None)?;
                let address_type =
                    address.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
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
                if extra.len() > 2 || extra.len() == 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                if extra.len() == 2 {
                    *with_capacity = true;
                } else {
                    *with_capacity = false;
                }
                for param in extra {
                    let _ = param.resolve::<G>(scope, &Some(p_num!(U64)), &mut None)?;
                }
                if context.is_none() {
                    return Err(SemanticError::CantInferType(format!(
                        "of this vector allocation"
                    )));
                }
                match &context {
                    Some(value) => match value {
                        Either::Static(value) => match value.as_ref() {
                            StaticType::Vec(VecType(item)) => *item_size = item.size_of(),
                            _ => return Err(SemanticError::IncompatibleTypes),
                        },
                        Either::User(_) => return Err(SemanticError::IncompatibleTypes),
                    },
                    None => unreachable!(),
                }

                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: context.clone(),
                };
                Ok(())
            }
            AllocFn::Map {
                with_capacity,
                value_size,
                key_size,
                metadata,
            } => {
                if extra.len() > 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                if extra.len() == 1 {
                    *with_capacity = true;
                } else {
                    *with_capacity = false;
                }
                for param in extra {
                    let _ = param.resolve::<G>(scope, &Some(p_num!(U64)), &mut None)?;
                }
                if context.is_none() {
                    return Err(SemanticError::CantInferType(format!(
                        "of this map allocation"
                    )));
                }
                match &context {
                    Some(value) => match value {
                        Either::Static(value) => match value.as_ref() {
                            StaticType::Map(MapType {
                                keys_type,
                                values_type,
                            }) => {
                                *value_size = values_type.size_of();
                                *key_size = keys_type.size_of();
                            }
                            _ => return Err(SemanticError::IncompatibleTypes),
                        },
                        Either::User(_) => return Err(SemanticError::IncompatibleTypes),
                    },
                    None => unreachable!(),
                }

                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: context.clone(),
                };
                Ok(())
            }
            AllocFn::String { len, from_char } => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let param = extra.first_mut().unwrap();
                let _ = param.resolve::<G>(scope, &None, &mut None)?;
                let param_type =
                    param.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                match param_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::StrSlice(slice) => {
                            *from_char = false;
                            *len = slice.size_of();
                        }
                        StaticType::Primitive(PrimitiveType::Char) => {
                            *from_char = true;
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

                let size = &mut extra[0];

                let _ = size.resolve::<G>(scope, &Some(p_num!(U64)), &mut None)?;
                let size_type =
                    size.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
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
                let address = &mut extra[0];

                let _ = address.resolve::<G>(scope, &None, &mut None)?;
                let address_type =
                    address.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
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
            AllocFn::Cap { for_map } => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let address = &mut extra[0];

                let _ = address.resolve::<G>(scope, &None, &mut None)?;
                let address_type =
                    address.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                *for_map = false;
                match &address_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::String(_) => {}
                        StaticType::Vec(_) => {}
                        StaticType::Map(_) => {
                            *for_map = true;
                        }
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
                let param = &mut extra[0];

                let _ = param.resolve::<G>(scope, &None, &mut None)?;
                let param_type =
                    param.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;

                *size = param_type.size_of();

                Ok(())
            }
            AllocFn::MemCopy => {
                if extra.len() != 3 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, rest) = extra.split_at_mut(1);
                let (second_part, third_part) = rest.split_at_mut(1);

                // Get mutable references to the elements
                let dest = &mut first_part[0];
                let src = &mut second_part[0];
                let size = &mut third_part[0];

                let _ = dest.resolve::<G>(scope, &None, &mut None)?;
                let _ = src.resolve::<G>(scope, &None, &mut None)?;
                let dest_type =
                    dest.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let src_type =
                    src.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
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
                let _ = size.resolve::<G>(scope, &Some(p_num!(U64)), &mut None)?;
                let size_type =
                    size.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
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
                key_size,
            } => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let src = &mut extra[0];

                let _ = src.resolve::<G>(scope, &None, &mut None)?;
                let src_type =
                    src.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;

                match &src_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Address(AddrType(inner)) => match inner.as_ref() {
                            Either::Static(value) => match value.as_ref() {
                                StaticType::String(_) => {
                                    *clear_kind = ClearKind::String;
                                    *item_size = 1;
                                }
                                StaticType::Vec(_) => {
                                    *clear_kind = ClearKind::Vec;
                                    let item_type = src_type.get_item();
                                    let Some(item_type) = item_type else {
                                        return Err(SemanticError::IncorrectArguments);
                                    };
                                    *item_size = item_type.size_of();
                                }
                                StaticType::Map(MapType {
                                    keys_type,
                                    values_type,
                                }) => {
                                    *clear_kind = ClearKind::Map;

                                    *item_size = values_type.as_ref().size_of();
                                    *key_size = keys_type.as_ref().size_of();
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
    fn type_of(&self, _scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            AllocFn::Append { .. } => Ok(e_static!(StaticType::Unit)),
            AllocFn::Insert { .. } => Ok(e_static!(StaticType::Unit)),
            AllocFn::Get { metadata, .. } => metadata
                .signature()
                .ok_or(SemanticError::NotResolvedYet)
                .map(|value| err_tuple!(value)),
            AllocFn::Delete { metadata, .. } => metadata
                .signature()
                .ok_or(SemanticError::NotResolvedYet)
                .map(|value| err_tuple!(value)),
            AllocFn::Free => Ok(e_static!(StaticType::Unit)),
            AllocFn::Vec { metadata, .. } => {
                metadata.signature().ok_or(SemanticError::NotResolvedYet)
            }
            AllocFn::Map { metadata, .. } => {
                metadata.signature().ok_or(SemanticError::NotResolvedYet)
            }
            AllocFn::String { .. } => Ok(e_static!(StaticType::String(StringType()))),
            AllocFn::Alloc => Ok(e_static!(StaticType::Any)),
            AllocFn::Len => Ok(p_num!(U64)),
            AllocFn::Cap { .. } => Ok(p_num!(U64)),
            AllocFn::SizeOf { .. } => Ok(p_num!(U64)),
            AllocFn::MemCopy => Ok(e_static!(StaticType::Unit)),
            AllocFn::Clear { .. } => Ok(e_static!(StaticType::Unit)),
            AllocFn::Extend {
                item_size,
                extend_kind,
            } => Ok(e_static!(StaticType::Unit)),
        }
    }
}

impl GenerateCode for AllocFn {
    fn gencode(
        &self,
        _scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            AllocFn::Append {
                item_size,
                append_kind,
            } => match append_kind {
                AppendKind::Vec => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::AppendItem(*item_size)),
                ))),
                AppendKind::StrSlice => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::AppendStrSlice(*item_size)),
                ))),
                AppendKind::Char => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::AppendChar),
                ))),
                AppendKind::String => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::AppendString),
                ))),
            },
            AllocFn::Extend {
                item_size,
                extend_kind,
            } => match extend_kind {
                ExtendKind::VecFromSlice(len) => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::ExtendItemFromSlice {
                        len: *len,
                        size: *item_size,
                    }),
                ))),
                ExtendKind::VecFromVec => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::ExtendItemFromVec { size: *item_size }),
                ))),
                ExtendKind::StringFromSlice(len) => {
                    instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                        AllocCasm::ExtendStringFromSlice { len: *len },
                    ))))
                }
                ExtendKind::StringFromVec => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::ExtendStringFromVec),
                ))),
            },
            AllocFn::Insert {
                key_size,
                value_size,
                ref_access,
            } => instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                AllocCasm::Insert {
                    key_size: *key_size,
                    value_size: *value_size,
                    ref_access: *ref_access,
                },
            )))),
            AllocFn::Get {
                key_size,
                value_size,
                ref_access,
                ..
            } => instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                AllocCasm::Get {
                    key_size: *key_size,
                    value_size: *value_size,
                    ref_access: *ref_access,
                },
            )))),
            AllocFn::Delete {
                delete_kind,
                value_size,
                key_size,
                ..
            } => match delete_kind {
                DeleteKind::Vec => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::DeleteVec(*value_size)),
                ))),
                DeleteKind::Map(ref_access) => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::DeleteMapKey {
                        key_size: *key_size,
                        value_size: *value_size,
                        ref_access: *ref_access,
                    }),
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
            } => {
                if *with_capacity {
                    instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                        AllocCasm::VecWithCapacity {
                            item_size: *item_size,
                        },
                    ))))
                } else {
                    instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                        AllocCasm::Vec {
                            item_size: *item_size,
                        },
                    ))))
                }
            }
            AllocFn::Map {
                with_capacity,
                value_size,
                key_size,
                ..
            } => {
                if *with_capacity {
                    instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                        AllocCasm::MapWithCapacity {
                            key_size: *key_size,
                            value_size: *value_size,
                        },
                    ))))
                } else {
                    instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                        AllocCasm::Map {
                            key_size: *key_size,
                            value_size: *value_size,
                        },
                    ))))
                }
            }
            AllocFn::String { from_char, .. } => {
                if *from_char {
                    instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                        AllocCasm::StringFromChar,
                    ))))
                } else {
                    instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                        AllocCasm::StringFromSlice,
                    ))))
                }
            }
            AllocFn::Alloc => {
                instructions.push(Casm::Alloc(Alloc::Heap { size: None }));
            }
            AllocFn::Len => instructions.push(Casm::Platform(LibCasm::Core(
                super::CoreCasm::Alloc(AllocCasm::Len),
            ))),
            AllocFn::Cap { for_map } => {
                if *for_map {
                    instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                        AllocCasm::CapMap,
                    ))))
                } else {
                    instructions.push(Casm::Platform(LibCasm::Core(super::CoreCasm::Alloc(
                        AllocCasm::Cap,
                    ))))
                }
            }
            AllocFn::SizeOf { size } => {
                instructions.push(Casm::Pop(*size));
                instructions.push(Casm::Data(Data::Serialized {
                    data: Box::new(size.to_le_bytes()),
                }));
            }
            AllocFn::MemCopy => instructions.push(Casm::Mem(Mem::MemCopy)),
            AllocFn::Clear {
                clear_kind,
                item_size,
                key_size,
            } => match clear_kind {
                ClearKind::Vec => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::ClearVec(*item_size)),
                ))),
                ClearKind::String => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::ClearString(1)),
                ))),
                ClearKind::Map => instructions.push(Casm::Platform(LibCasm::Core(
                    super::CoreCasm::Alloc(AllocCasm::ClearMap {
                        key_size: *key_size,
                        value_size: *item_size,
                    }),
                ))),
            },
        }
        Ok(())
    }
}

pub mod map_impl {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use num_traits::ToBytes;
    use rand::Rng;

    use crate::vm::{
        allocator::heap::{Heap, HeapError, HEAP_SIZE},
        vm::RuntimeError,
    };

    use super::DerefHashing;

    pub const MAP_BUCKET_SIZE: usize = 8;
    pub const MAP_LAYOUT_SIZE: usize = 21;
    enum TopHashValue {
        RestIsEmpty = 0,
        EmptyCell = 1,
        MIN = 2,
    }

    /*
        MAP ALLOCATION LAYOUT:
            len : u64,
            log_cap : u8, // (called B in go) log_2 of number of buckets
            hash_seed : u32,
            ptr_buckets : u64,

        total size : 8 + 1 + 4 + 8 = 21, align to :
    */
    #[derive(Debug)]
    pub struct MapLayout {
        pub ptr_map_layout: u64,
        pub bucket_size: usize,
        pub key_size: usize,
        pub value_size: usize,
        pub len: u64,
        pub log_cap: u8, // (called B in go) log_2 of number of buckets
        pub hash_seed: u32,
        pub ptr_buckets: u64,
    }
    #[derive(Debug)]
    pub struct BucketLayout {
        pub ptr_top_hash: u64,
        pub keys_top_hash: [u8; MAP_BUCKET_SIZE],
        pub ptr_keys: u64,
        pub key_size: usize,
        pub ptr_values: u64,
        pub value_size: usize,
    }

    impl BucketLayout {
        pub fn assign(
            &self,
            top_hash: u8,
            key: &[u8],
            ref_access: DerefHashing,

            heap: &mut Heap,
        ) -> Result<Option<(u64, u64, u64, bool)>, RuntimeError> {
            let mut indexes = None;
            let mut first_empty_cell = None;
            for (idx, self_top_hash) in self.keys_top_hash.iter().enumerate() {
                if *self_top_hash == TopHashValue::EmptyCell as u8 {
                    if first_empty_cell.is_none() {
                        first_empty_cell = Some(idx);
                    }
                    continue;
                }
                if *self_top_hash == top_hash {
                    // Read key
                    let found_key =
                        heap.read(self.ptr_keys as usize + idx * self.key_size, self.key_size)?;
                    match ref_access {
                        DerefHashing::Vec(item_size) => {
                            // found_key is a pointer to a vec
                            let vec_heap_address_bytes =
                                TryInto::<&[u8; 8]>::try_into(found_key.as_slice())
                                    .map_err(|_| RuntimeError::Deserialization)?;
                            let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);
                            let len_bytes = heap.read(vec_heap_address as usize, 8)?;
                            let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                                .map_err(|_| RuntimeError::Deserialization)?;
                            let len = u64::from_le_bytes(*len_bytes);

                            let items_bytes = heap
                                .read(vec_heap_address as usize + 16, len as usize * item_size)?;
                            if items_bytes.as_slice() == key {
                                indexes = Some((
                                    self.ptr_top_hash + idx as u64,
                                    self.ptr_keys + (idx * self.key_size) as u64,
                                    self.ptr_values + (idx * self.value_size) as u64,
                                    false,
                                ))
                            }
                        }
                        DerefHashing::String => {
                            // found_key is a pointer to a string
                            let vec_heap_address_bytes =
                                TryInto::<&[u8; 8]>::try_into(found_key.as_slice())
                                    .map_err(|_| RuntimeError::Deserialization)?;
                            let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);
                            let len_bytes = heap.read(vec_heap_address as usize, 8)?;
                            let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                                .map_err(|_| RuntimeError::Deserialization)?;
                            let len = u64::from_le_bytes(*len_bytes);

                            let items_bytes =
                                heap.read(vec_heap_address as usize + 16, len as usize)?;
                            if items_bytes.as_slice() == key {
                                indexes = Some((
                                    self.ptr_top_hash + idx as u64,
                                    self.ptr_keys + (idx * self.key_size) as u64,
                                    self.ptr_values + (idx * self.value_size) as u64,
                                    false,
                                ))
                            }
                        }
                        DerefHashing::Default => {
                            if found_key.as_slice() == key {
                                indexes = Some((
                                    self.ptr_top_hash + idx as u64,
                                    self.ptr_keys + (idx * self.key_size) as u64,
                                    self.ptr_values + (idx * self.value_size) as u64,
                                    false,
                                ))
                            }
                        }
                    }
                }
                if *self_top_hash == TopHashValue::RestIsEmpty as u8 {
                    if first_empty_cell.is_none() {
                        first_empty_cell = Some(idx);
                    }
                    break;
                }
            }
            if indexes.is_none() {
                if let Some(idx) = first_empty_cell {
                    return Ok(Some((
                        self.ptr_top_hash + idx as u64,
                        self.ptr_keys + (idx * self.key_size) as u64,
                        self.ptr_values + (idx * self.value_size) as u64,
                        true,
                    )));
                } else {
                    return Ok(None);
                }
            } else {
                return Ok(indexes);
            }
        }
        pub fn get(
            &self,
            top_hash: u8,
            key: &[u8],
            ref_access: DerefHashing,

            heap: &mut Heap,
        ) -> Result<Option<u64>, RuntimeError> {
            for (idx, self_top_hash) in self.keys_top_hash.iter().enumerate() {
                if *self_top_hash == TopHashValue::EmptyCell as u8 {
                    continue;
                }
                if *self_top_hash == top_hash {
                    // Read key
                    let found_key =
                        heap.read(self.ptr_keys as usize + idx * self.key_size, self.key_size)?;
                    match ref_access {
                        DerefHashing::Vec(item_size) => {
                            // found_key is a pointer to a vec
                            let vec_heap_address_bytes =
                                TryInto::<&[u8; 8]>::try_into(found_key.as_slice())
                                    .map_err(|_| RuntimeError::Deserialization)?;
                            let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);
                            let len_bytes = heap.read(vec_heap_address as usize, 8)?;
                            let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                                .map_err(|_| RuntimeError::Deserialization)?;
                            let len = u64::from_le_bytes(*len_bytes);

                            let items_bytes = heap
                                .read(vec_heap_address as usize + 16, len as usize * item_size)?;
                            if items_bytes.as_slice() == key {
                                return Ok(Some(self.ptr_values + (idx * self.value_size) as u64));
                            }
                        }
                        DerefHashing::String => {
                            // found_key is a pointer to a string
                            let vec_heap_address_bytes =
                                TryInto::<&[u8; 8]>::try_into(found_key.as_slice())
                                    .map_err(|_| RuntimeError::Deserialization)?;
                            let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);
                            let len_bytes = heap.read(vec_heap_address as usize, 8)?;
                            let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                                .map_err(|_| RuntimeError::Deserialization)?;
                            let len = u64::from_le_bytes(*len_bytes);

                            let items_bytes =
                                heap.read(vec_heap_address as usize + 16, len as usize)?;
                            if items_bytes.as_slice() == key {
                                return Ok(Some(self.ptr_values + (idx * self.value_size) as u64));
                            }
                        }
                        DerefHashing::Default => {
                            if found_key.as_slice() == key {
                                return Ok(Some(self.ptr_values + (idx * self.value_size) as u64));
                            }
                        }
                    }
                }
                if *self_top_hash == TopHashValue::RestIsEmpty as u8 {
                    break;
                }
            }
            return Ok(None);
        }
        pub fn delete(
            &self,
            top_hash: u8,
            key: &[u8],
            ref_access: DerefHashing,

            heap: &mut Heap,
        ) -> Result<Option<u64>, RuntimeError> {
            let mut found_idx = None;
            for (idx, self_top_hash) in self.keys_top_hash.iter().enumerate() {
                if *self_top_hash == TopHashValue::EmptyCell as u8 {
                    continue;
                }
                if *self_top_hash == top_hash {
                    // Read key
                    let found_key =
                        heap.read(self.ptr_keys as usize + idx * self.key_size, self.key_size)?;
                    match ref_access {
                        DerefHashing::Vec(item_size) => {
                            // found_key is a pointer to a vec
                            let vec_heap_address_bytes =
                                TryInto::<&[u8; 8]>::try_into(found_key.as_slice())
                                    .map_err(|_| RuntimeError::Deserialization)?;
                            let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);
                            let len_bytes = heap.read(vec_heap_address as usize, 8)?;
                            let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                                .map_err(|_| RuntimeError::Deserialization)?;
                            let len = u64::from_le_bytes(*len_bytes);

                            let items_bytes = heap
                                .read(vec_heap_address as usize + 16, len as usize * item_size)?;
                            if items_bytes.as_slice() == key {
                                found_idx = Some(idx);
                            }
                        }
                        DerefHashing::String => {
                            // found_key is a pointer to a string
                            let vec_heap_address_bytes =
                                TryInto::<&[u8; 8]>::try_into(found_key.as_slice())
                                    .map_err(|_| RuntimeError::Deserialization)?;
                            let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);
                            let len_bytes = heap.read(vec_heap_address as usize, 8)?;
                            let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                                .map_err(|_| RuntimeError::Deserialization)?;
                            let len = u64::from_le_bytes(*len_bytes);

                            let items_bytes =
                                heap.read(vec_heap_address as usize + 16, len as usize)?;
                            if items_bytes.as_slice() == key {
                                return Ok(Some(self.ptr_values + (idx * self.value_size) as u64));
                            }
                        }
                        DerefHashing::Default => {
                            if found_key.as_slice() == key {
                                found_idx = Some(idx);
                            }
                        }
                    }
                }
                if *self_top_hash == TopHashValue::RestIsEmpty as u8 {
                    break;
                }
            }
            // Update Cell
            if let Some(idx) = found_idx {
                if idx < MAP_BUCKET_SIZE - 1 {
                    // Get next cell
                    let next_top_hash = self.keys_top_hash[idx + 1];
                    if next_top_hash == TopHashValue::RestIsEmpty as u8 {
                        // Write RestIsEmpty
                        heap.write(
                            self.ptr_top_hash as usize + idx,
                            &vec![TopHashValue::RestIsEmpty as u8],
                        )?;
                    } else {
                        // Write EmptyCell
                        heap.write(
                            self.ptr_top_hash as usize + idx,
                            &vec![TopHashValue::EmptyCell as u8],
                        )?;
                    }
                } else {
                    // Write RestIsEmpty
                    heap.write(
                        self.ptr_top_hash as usize + idx,
                        &vec![TopHashValue::RestIsEmpty as u8],
                    )?;
                }
                Ok(Some(self.ptr_values + (idx * self.value_size) as u64))
            } else {
                Ok(None)
            }
        }
    }

    impl MapLayout {
        fn len_offset() -> usize {
            0
        }

        fn log_cap_offset() -> usize {
            8
        }

        fn hash_seed_offset() -> usize {
            8 + 1
        }

        pub fn ptr_buckets_offset() -> usize {
            8 + 1 + 4
        }

        pub fn new(key_size: usize, value_size: usize, log_cap: u8) -> Self {
            Self {
                ptr_map_layout: 0,
                bucket_size: MAP_BUCKET_SIZE
                    + MAP_BUCKET_SIZE * key_size
                    + MAP_BUCKET_SIZE * value_size,
                key_size,
                value_size,
                len: 0,
                log_cap,
                hash_seed: gen_seed(),
                ptr_buckets: 0,
            }
        }

        pub fn init_in_mem(&self, heap: &mut Heap) -> Result<usize, RuntimeError> {
            // alloc map layout
            let map_ptr = heap.alloc(MAP_LAYOUT_SIZE)? + 8;

            let mut data = [0; MAP_LAYOUT_SIZE];
            // write len
            data[0..8].copy_from_slice(&self.len.to_le_bytes());
            // write log_cap
            data[8] = self.log_cap;
            // write seed
            data[9..13].copy_from_slice(&self.hash_seed.to_le_bytes());

            // alloc buckets
            let buckets_ptr = heap.alloc((1 << self.log_cap) * self.bucket_size)? + 8;
            // clean buckets
            let _ = heap.write(
                buckets_ptr,
                &vec![0u8; (1 << self.log_cap) * self.bucket_size],
            )?;

            let buckets_ptr = buckets_ptr as u64;

            // write buckets_ptr
            data[13..].copy_from_slice(&buckets_ptr.to_le_bytes());

            // write map layout in mem
            let _ = heap.write(map_ptr, &data.to_vec())?;

            Ok(map_ptr)
        }

        fn update_log_cap(&self, new_log_cap: u8, heap: &mut Heap) -> Result<(), RuntimeError> {
            let _ = heap.write(
                self.ptr_map_layout as usize + MapLayout::log_cap_offset(),
                &vec![new_log_cap],
            )?;
            Ok(())
        }

        fn update_buckets_ptr(&self, bucket_ptr: u64, heap: &mut Heap) -> Result<(), RuntimeError> {
            let _ = heap.write(
                self.ptr_map_layout as usize + MapLayout::ptr_buckets_offset(),
                &bucket_ptr.to_le_bytes().to_vec(),
            )?;
            Ok(())
        }

        pub fn resize(&self, heap: &mut Heap) -> Result<(), RuntimeError> {
            let mut new_log_cap = self.log_cap + 1;

            let previous_ptr_bucket = self.ptr_buckets;
            // get all buckets
            let bytes_buckets = heap.read(
                self.ptr_buckets as usize,
                (1 << self.log_cap) * self.bucket_size,
            )?;
            let mut resizing_is_over = false;
            let mut new_bytes_buckets = Vec::new();
            'again: while !resizing_is_over {
                let alloc_size = (1 << new_log_cap) * self.bucket_size;
                if alloc_size > HEAP_SIZE {
                    return Err(RuntimeError::HeapError(HeapError::AllocationError));
                }
                new_bytes_buckets = vec![0u8; (1 << new_log_cap) * self.bucket_size];

                for bucket in bytes_buckets.chunks_exact(self.bucket_size) {
                    let key_value_pair = bucket
                        [MAP_BUCKET_SIZE..MAP_BUCKET_SIZE + MAP_BUCKET_SIZE * self.key_size]
                        .chunks_exact(self.key_size)
                        .zip(
                            bucket[MAP_BUCKET_SIZE + MAP_BUCKET_SIZE * self.key_size
                                ..MAP_BUCKET_SIZE
                                    + MAP_BUCKET_SIZE * self.key_size
                                    + MAP_BUCKET_SIZE * self.value_size]
                                .chunks_exact(self.value_size),
                        )
                        .enumerate();
                    for (idx_key_value, (key, value)) in key_value_pair {
                        if bucket[idx_key_value] <= TopHashValue::MIN as u8 {
                            // Skip empty cell
                            continue;
                        }
                        // Compute hash
                        let hash = hash_of(key, self.hash_seed);
                        let new_bucket_idx = bucket_idx(hash, new_log_cap) as usize;
                        // index of key_top_hash in new bucket
                        let ptr_new_bucket = new_bucket_idx * self.bucket_size;
                        let mut idx_in_new_bucket = None;
                        for idx_top_hash in 0..MAP_BUCKET_SIZE {
                            if new_bytes_buckets[ptr_new_bucket + idx_top_hash]
                                <= TopHashValue::MIN as u8
                            {
                                idx_in_new_bucket = Some(idx_top_hash);
                                break;
                            }
                        }
                        if idx_in_new_bucket.is_none() {
                            new_log_cap += 1;
                            continue 'again;
                        }
                        let idx_in_new_bucket = idx_in_new_bucket.unwrap();
                        // Update top_hash, key and value in the found idx
                        // update top_hash
                        new_bytes_buckets[ptr_new_bucket + idx_in_new_bucket] = top_hash(hash);
                        // update key
                        new_bytes_buckets[ptr_new_bucket
                            + MAP_BUCKET_SIZE
                            + idx_in_new_bucket * self.key_size
                            ..ptr_new_bucket
                                + MAP_BUCKET_SIZE
                                + idx_in_new_bucket * self.key_size
                                + self.key_size]
                            .copy_from_slice(key);

                        // update value
                        new_bytes_buckets[ptr_new_bucket
                            + MAP_BUCKET_SIZE
                            + MAP_BUCKET_SIZE * self.key_size
                            + idx_in_new_bucket * self.value_size
                            ..ptr_new_bucket
                                + MAP_BUCKET_SIZE
                                + MAP_BUCKET_SIZE * self.key_size
                                + idx_in_new_bucket * self.value_size
                                + self.value_size]
                            .copy_from_slice(value);
                    }
                }
                resizing_is_over = true;
            }

            // Update log_cap in memory: log_cap += 1
            let _ = self.update_log_cap(new_log_cap, heap)?;

            // free previous_buckets
            let _ = heap.free((previous_ptr_bucket - 8) as usize)?;

            // alloc new buckets
            let new_buckets_ptr = heap
            .alloc(new_bytes_buckets.len())? + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;
            // copy new buckets in memory
            let _ = heap.write(new_buckets_ptr, &new_bytes_buckets)?;

            // update buckets ptr
            let _ = self.update_buckets_ptr(new_buckets_ptr as u64, heap)?;

            Ok(())
        }

        pub fn clear_buckets(&self, heap: &mut Heap) -> Result<(), RuntimeError> {
            let _ = heap.write(
                self.ptr_buckets as usize,
                &vec![0; (1 << self.log_cap) * self.bucket_size],
            )?;
            Ok(())
        }
        pub fn retrieve_vec_values(&self, heap: &mut Heap) -> Result<Vec<u64>, RuntimeError> {
            // get all buckets
            let bytes_buckets = heap.read(
                self.ptr_buckets as usize,
                (1 << self.log_cap) * self.bucket_size,
            )?;

            let mut items_ptr = Vec::with_capacity(self.len as usize);

            for (idx, bucket) in bytes_buckets.chunks_exact(self.bucket_size).enumerate() {
                for idx_top_hash in 0..MAP_BUCKET_SIZE {
                    if bucket[idx_top_hash] > TopHashValue::MIN as u8 {
                        items_ptr.push(
                            self.ptr_buckets
                                + idx as u64 * self.bucket_size as u64
                                + MAP_BUCKET_SIZE as u64
                                + MAP_BUCKET_SIZE as u64 * self.key_size as u64
                                + idx_top_hash as u64 * self.value_size as u64,
                        )
                    }
                }
            }
            Ok(items_ptr)
        }
        pub fn retrieve_vec_keys(&self, heap: &mut Heap) -> Result<Vec<u64>, RuntimeError> {
            // get all buckets
            let bytes_buckets = heap.read(
                self.ptr_buckets as usize,
                (1 << self.log_cap) * self.bucket_size,
            )?;

            let mut items_ptr = Vec::with_capacity(self.len as usize);

            for (idx, bucket) in bytes_buckets.chunks_exact(self.bucket_size).enumerate() {
                for idx_top_hash in 0..MAP_BUCKET_SIZE {
                    if bucket[idx_top_hash] > TopHashValue::MIN as u8 {
                        items_ptr.push(
                            self.ptr_buckets
                                + idx as u64 * self.bucket_size as u64
                                + MAP_BUCKET_SIZE as u64
                                + idx_top_hash as u64 * self.key_size as u64,
                        )
                    }
                }
            }
            Ok(items_ptr)
        }
        pub fn retrieve_vec_items(&self, heap: &mut Heap) -> Result<Vec<(u64, u64)>, RuntimeError> {
            // get all buckets
            let bytes_buckets = heap.read(
                self.ptr_buckets as usize,
                (1 << self.log_cap) * self.bucket_size,
            )?;

            let mut items_ptr = Vec::with_capacity(self.len as usize);

            for (idx, bucket) in bytes_buckets.chunks_exact(self.bucket_size).enumerate() {
                for idx_top_hash in 0..MAP_BUCKET_SIZE {
                    if bucket[idx_top_hash] > TopHashValue::MIN as u8 {
                        items_ptr.push((
                            self.ptr_buckets
                                + idx as u64 * self.bucket_size as u64
                                + MAP_BUCKET_SIZE as u64
                                + idx_top_hash as u64 * self.key_size as u64,
                            self.ptr_buckets
                                + idx as u64 * self.bucket_size as u64
                                + MAP_BUCKET_SIZE as u64
                                + MAP_BUCKET_SIZE as u64 * self.key_size as u64
                                + idx_top_hash as u64 * self.value_size as u64,
                        ))
                    }
                }
            }
            Ok(items_ptr)
        }
    }

    pub fn over_load_factor(size: u64, log_cap: u8) -> bool {
        size > MAP_BUCKET_SIZE as u64 && size > ((3 * (1 << log_cap) * MAP_BUCKET_SIZE as u64) / 4)
    }

    pub fn bucket_layout(
        address: u64,
        key_size: usize,
        value_size: usize,

        heap: &mut Heap,
    ) -> Result<BucketLayout, RuntimeError> {
        let data = heap.read(address as usize, MAP_BUCKET_SIZE)?;
        if data.len() != MAP_BUCKET_SIZE {
            return Err(RuntimeError::CodeSegmentation);
        }
        let keys_top_hash: [u8; MAP_BUCKET_SIZE] =
            data.try_into().map_err(|_| RuntimeError::Deserialization)?;
        Ok(BucketLayout {
            ptr_top_hash: address,
            keys_top_hash,
            ptr_keys: address + MAP_BUCKET_SIZE as u64,
            key_size,
            ptr_values: address + MAP_BUCKET_SIZE as u64 + MAP_BUCKET_SIZE as u64 * key_size as u64,
            value_size,
        })
    }

    pub fn map_layout(
        address: u64,
        key_size: usize,
        value_size: usize,
        heap: &mut Heap,
    ) -> Result<MapLayout, RuntimeError> {
        let data = heap.read(address as usize, 21)?;
        if data.len() != 21 {
            return Err(RuntimeError::CodeSegmentation);
        }
        let len = u64::from_le_bytes(
            data[0..8]
                .try_into()
                .map_err(|_| RuntimeError::Deserialization)?,
        );
        let log_cap = u8::from_le_bytes(
            data[8..9]
                .try_into()
                .map_err(|_| RuntimeError::Deserialization)?,
        );
        let hash_seed = u32::from_le_bytes(
            data[9..13]
                .try_into()
                .map_err(|_| RuntimeError::Deserialization)?,
        );
        let ptr_buckets = u64::from_le_bytes(
            data[13..21]
                .try_into()
                .map_err(|_| RuntimeError::Deserialization)?,
        );
        Ok(MapLayout {
            ptr_map_layout: address,
            bucket_size: MAP_BUCKET_SIZE + 8 * key_size + 8 * value_size,
            key_size,
            value_size,
            len,
            log_cap,
            hash_seed,
            ptr_buckets,
        })
    }

    pub fn top_hash(hash: u64) -> u8 {
        let mut top = (hash >> 48) as u8;
        if top <= TopHashValue::MIN as u8 {
            top += TopHashValue::MIN as u8;
        }
        return top;
    }

    pub fn bucket_idx(hash: u64, log_cap: u8) -> u64 {
        hash & ((1 << log_cap) - 1)
    }

    pub fn hash_of(bytes: &[u8], seed: u32) -> u64 {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish()
    }

    fn gen_seed() -> u32 {
        let mut rng = rand::thread_rng();
        let seed: u32 = rng.gen();
        seed
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for AllocCasm {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match self {
            AllocCasm::AppendChar => {
                let chara = OpPrimitive::get_char(stack)?;
                let chara = chara.to_string();
                let item_data = chara.as_bytes().to_vec();
                let item_len = chara.len();

                let vec_stack_address = OpPrimitive::get_num8::<u64>(stack)?;
                let vec_heap_address_bytes = stack.read(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    8,
                )?;
                let vec_heap_address_bytes = TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);

                let previous_len_bytes = heap.read(vec_heap_address as usize, 8)?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let previous_cap_bytes = heap.read(vec_heap_address as usize + 8, 8)?;
                let previous_cap_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_cap_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_cap = u64::from_le_bytes(*previous_cap_bytes);

                let (new_vec_heap_address, new_len, new_cap) = if previous_len + (item_len as u64)
                    >= previous_cap
                {
                    /* Reallocation */
                    let size = align(((previous_len + (item_len as u64)) * 2) as usize) + 16;
                    let address = heap.realloc(vec_heap_address as usize - 8, size)?;
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
                let _ = heap.write(new_vec_heap_address as usize, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(new_vec_heap_address as usize + 8, &cap_bytes)?;

                /* Write new item */
                let _ = heap.write(
                    new_vec_heap_address as usize + 16 + new_len as usize - item_len,
                    &item_data,
                )?;
                /* Update vector pointer */
                let _ = stack.write(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    &new_vec_heap_address.to_le_bytes(),
                )?;
            }
            AllocCasm::AppendItem(item_size) | AllocCasm::AppendStrSlice(item_size) => {
                let item_size = match self {
                    AllocCasm::AppendItem(_) => *item_size,
                    AllocCasm::AppendStrSlice(_) => {
                        let len = OpPrimitive::get_num8::<u64>(stack)?;
                        len as usize
                    }
                    _ => unreachable!(),
                };
                let item_data = stack.pop(item_size)?.to_owned();

                let vec_stack_address = OpPrimitive::get_num8::<u64>(stack)?;

                let vec_heap_address_bytes = stack.read(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    8,
                )?;
                let vec_heap_address_bytes = TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);
                let previous_len_bytes = heap.read(vec_heap_address as usize, 8)?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let previous_cap_bytes = heap.read(vec_heap_address as usize + 8, 8)?;
                let previous_cap_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_cap_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_cap = u64::from_le_bytes(*previous_cap_bytes);

                let len_offset = match self {
                    AllocCasm::AppendItem(_) => 1,
                    AllocCasm::AppendStrSlice(_) => item_size as u64,
                    _ => unreachable!(),
                };

                let size_factor = match self {
                    AllocCasm::AppendItem(_) => item_size,
                    AllocCasm::AppendStrSlice(_) => 1,
                    _ => unreachable!(),
                };

                let (new_vec_heap_address, new_len, new_cap) = if previous_len + len_offset
                    >= previous_cap
                {
                    /* Reallocation */
                    let size = align(((previous_len + len_offset) * 2) as usize * size_factor + 16);
                    let address = heap.realloc(vec_heap_address as usize - 8, size)?;
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
                let _ = heap.write(new_vec_heap_address as usize, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(new_vec_heap_address as usize + 8, &cap_bytes)?;

                /* Write new item */
                let _ = heap.write(
                    new_vec_heap_address as usize + 16 + (new_len as usize * size_factor as usize)
                        - item_size,
                    &item_data.to_vec(),
                )?;
                /* Update vector pointer */
                let _ = stack.write(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    &new_vec_heap_address.to_le_bytes(),
                )?;
            }
            AllocCasm::AppendString => {
                let item_heap_address = OpPrimitive::get_num8::<u64>(stack)?;
                let item_len_bytes = heap.read(item_heap_address as usize, 8)?;
                let item_len_bytes = TryInto::<&[u8; 8]>::try_into(item_len_bytes.as_slice())
                    .map_err(|_| RuntimeError::Deserialization)?;
                let item_len = u64::from_le_bytes(*item_len_bytes);

                let item_data = heap.read(item_heap_address as usize + 16, item_len as usize)?;

                let _ = heap.free(item_heap_address as usize - 8)?;

                let vec_stack_address = OpPrimitive::get_num8::<u64>(stack)?;
                let vec_heap_address_bytes = stack.read(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    8,
                )?;
                let vec_heap_address_bytes = TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);

                let previous_len_bytes = heap.read(vec_heap_address as usize, 8)?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let previous_cap_bytes = heap.read(vec_heap_address as usize + 8, 8)?;
                let previous_cap_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_cap_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_cap = u64::from_le_bytes(*previous_cap_bytes);

                let (new_vec_heap_address, new_len, new_cap) = if previous_len + item_len
                    >= previous_cap
                {
                    /* Reallocation */
                    let size = align(((previous_len + item_len) * 2) as usize) + 16;
                    let address = heap.realloc(vec_heap_address as usize - 8, size)?;
                    let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;
                    (address as u64, previous_len + item_len, size as u64)
                } else {
                    (vec_heap_address, previous_len + item_len, previous_cap)
                };
                let len_bytes = new_len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = new_cap.to_le_bytes().as_slice().to_vec();
                /* Write len */
                let _ = heap.write(new_vec_heap_address as usize, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(new_vec_heap_address as usize + 8, &cap_bytes)?;

                /* Write new item */
                let _ = heap.write(
                    new_vec_heap_address as usize + 16 + new_len as usize - item_len as usize,
                    &item_data,
                )?;
                /* Update vector pointer */
                let _ = stack.write(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    &new_vec_heap_address.to_le_bytes(),
                )?;
            }
            AllocCasm::Insert {
                key_size,
                value_size,
                ref_access,
            } => {
                let value_data = stack.pop(*value_size)?.to_owned();
                let (key_data_ref_if_exist, key_data) = match ref_access {
                    DerefHashing::Vec(item_size) => {
                        let vec_heap_address = OpPrimitive::get_num8::<u64>(stack)?;

                        let len_bytes = heap.read(vec_heap_address as usize, 8)?;
                        let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                            .map_err(|_| RuntimeError::Deserialization)?;
                        let len = u64::from_le_bytes(*len_bytes);
                        let items_bytes =
                            heap.read(vec_heap_address as usize + 16, len as usize * *item_size)?;
                        (Some(vec_heap_address.to_le_bytes()), items_bytes)
                    }
                    DerefHashing::String => {
                        let str_heap_address = OpPrimitive::get_num8::<u64>(stack)?;
                        let len_bytes = heap.read(str_heap_address as usize, 8)?;
                        let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                            .map_err(|_| RuntimeError::Deserialization)?;
                        let len = u64::from_le_bytes(*len_bytes);
                        let items_bytes =
                            heap.read(str_heap_address as usize + 16, len as usize)?;
                        (Some(str_heap_address.to_le_bytes()), items_bytes)
                    }
                    DerefHashing::Default => (None, stack.pop(*key_size)?.to_vec()),
                };

                let map_stack_address = OpPrimitive::get_num8::<u64>(stack)?;

                let map_heap_address_bytes = stack.read(
                    Offset::SB(map_stack_address as usize),
                    AccessLevel::Direct,
                    8,
                )?;
                let map_heap_address_bytes = TryInto::<&[u8; 8]>::try_into(map_heap_address_bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                let map_heap_address = u64::from_le_bytes(*map_heap_address_bytes);
                let mut insertion_successful = false;

                while !insertion_successful {
                    let map_layout = map_layout(map_heap_address, *key_size, *value_size, heap)?;

                    let hash = hash_of(&key_data, map_layout.hash_seed);
                    let top_hash = top_hash(hash);
                    let bucket_idx = bucket_idx(hash, map_layout.log_cap) as u64;

                    // get address of the bucket
                    let bucket_address =
                        map_layout.ptr_buckets + bucket_idx * map_layout.bucket_size as u64;

                    let bucket_layout =
                        bucket_layout(bucket_address, *key_size, *value_size, heap)?;

                    let opt_ptr_key_value =
                        bucket_layout.assign(top_hash, &key_data, *ref_access, heap)?;
                    match opt_ptr_key_value {
                        Some((ptr_tophash, ptr_key, ptr_value, is_new_value)) => {
                            // trigger resizing if overload
                            if is_new_value {
                                if over_load_factor(map_layout.len + 1, map_layout.log_cap) {
                                    let _ = map_layout.resize(heap)?;
                                    // resizing invalidates everything so perform the whole operation again
                                    continue;
                                }
                            }
                            // insert in found place
                            let _ = heap.write(ptr_tophash as usize, &vec![top_hash])?;
                            match key_data_ref_if_exist {
                                Some(real_key_data) => {
                                    let _ =
                                        heap.write(ptr_key as usize, &real_key_data.to_vec())?;
                                }
                                None => {
                                    let _ = heap.write(ptr_key as usize, &key_data)?;
                                }
                            }
                            let _ = heap.write(ptr_value as usize, &value_data.to_vec())?;
                            if is_new_value {
                                // update len
                                let _ = heap.write(
                                    map_heap_address as usize,
                                    &(map_layout.len + 1).to_le_bytes().to_vec(),
                                )?;
                            }
                            insertion_successful = true;
                        }
                        None => {
                            // resize and retry
                            let _ = map_layout.resize(heap)?;
                        }
                    }
                }
            }
            AllocCasm::InsertAndForward {
                ref_access,
                key_size,
                value_size,
            } => {
                let value_data = stack.pop(*value_size)?.to_owned();
                let (key_data_ref_if_exist, key_data) = match ref_access {
                    DerefHashing::Vec(item_size) => {
                        let vec_heap_address = OpPrimitive::get_num8::<u64>(stack)?;

                        let len_bytes = heap.read(vec_heap_address as usize, 8)?;
                        let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                            .map_err(|_| RuntimeError::Deserialization)?;
                        let len = u64::from_le_bytes(*len_bytes);
                        let items_bytes =
                            heap.read(vec_heap_address as usize + 16, len as usize * *item_size)?;
                        (Some(vec_heap_address.to_le_bytes()), items_bytes)
                    }
                    DerefHashing::String => {
                        let str_heap_address = OpPrimitive::get_num8::<u64>(stack)?;
                        let len_bytes = heap.read(str_heap_address as usize, 8)?;
                        let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                            .map_err(|_| RuntimeError::Deserialization)?;
                        let len = u64::from_le_bytes(*len_bytes);
                        let items_bytes =
                            heap.read(str_heap_address as usize + 16, len as usize)?;
                        (Some(str_heap_address.to_le_bytes()), items_bytes)
                    }
                    DerefHashing::Default => (None, stack.pop(*key_size)?.to_vec()),
                };

                let map_heap_address = OpPrimitive::get_num8::<u64>(stack)?;

                let mut insertion_successful = false;

                while !insertion_successful {
                    let map_layout = map_layout(map_heap_address, *key_size, *value_size, heap)?;

                    let hash = hash_of(&key_data, map_layout.hash_seed);
                    let top_hash = top_hash(hash);
                    let bucket_idx = bucket_idx(hash, map_layout.log_cap) as u64;

                    // get address of the bucket
                    let bucket_address =
                        map_layout.ptr_buckets + bucket_idx * map_layout.bucket_size as u64;

                    let bucket_layout =
                        bucket_layout(bucket_address, *key_size, *value_size, heap)?;

                    let opt_ptr_key_value =
                        bucket_layout.assign(top_hash, &key_data, *ref_access, heap)?;
                    match opt_ptr_key_value {
                        Some((ptr_tophash, ptr_key, ptr_value, is_new_value)) => {
                            // trigger resizing if overload
                            if is_new_value {
                                if over_load_factor(map_layout.len + 1, map_layout.log_cap) {
                                    let _ = map_layout.resize(heap)?;
                                    // resizing invalidates everything so perform the whole operation again
                                    continue;
                                }
                            }
                            // insert in found place
                            let _ = heap.write(ptr_tophash as usize, &vec![top_hash])?;
                            match key_data_ref_if_exist {
                                Some(real_key_data) => {
                                    let _ =
                                        heap.write(ptr_key as usize, &real_key_data.to_vec())?;
                                }
                                None => {
                                    let _ = heap.write(ptr_key as usize, &key_data)?;
                                }
                            }
                            let _ = heap.write(ptr_value as usize, &value_data.to_vec())?;
                            if is_new_value {
                                // update len
                                let _ = heap.write(
                                    map_heap_address as usize,
                                    &(map_layout.len + 1).to_le_bytes().to_vec(),
                                )?;
                            }
                            insertion_successful = true;
                        }
                        None => {
                            // resize and retry
                            let _ = map_layout.resize(heap)?;
                        }
                    }
                }

                let _ = stack.push_with(&map_heap_address.to_le_bytes())?;
            }

            AllocCasm::Get {
                key_size,
                value_size,
                ref_access,
            } => {
                let key_data = match ref_access {
                    DerefHashing::Vec(item_size) => {
                        let vec_heap_address = OpPrimitive::get_num8::<u64>(stack)?;

                        let len_bytes = heap.read(vec_heap_address as usize, 8)?;
                        let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                            .map_err(|_| RuntimeError::Deserialization)?;
                        let len = u64::from_le_bytes(*len_bytes);
                        let items_bytes =
                            heap.read(vec_heap_address as usize + 16, len as usize * *item_size)?;
                        items_bytes
                    }
                    DerefHashing::String => {
                        let str_heap_address = OpPrimitive::get_num8::<u64>(stack)?;
                        let len_bytes = heap.read(str_heap_address as usize, 8)?;
                        let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                            .map_err(|_| RuntimeError::Deserialization)?;
                        let len = u64::from_le_bytes(*len_bytes);
                        let items_bytes =
                            heap.read(str_heap_address as usize + 16, len as usize)?;
                        items_bytes
                    }
                    DerefHashing::Default => stack.pop(*key_size)?.to_vec(),
                };

                let map_stack_address = OpPrimitive::get_num8::<u64>(stack)?;

                let map_heap_address_bytes = stack.read(
                    Offset::SB(map_stack_address as usize),
                    AccessLevel::Direct,
                    8,
                )?;
                let map_heap_address_bytes = TryInto::<&[u8; 8]>::try_into(map_heap_address_bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                let map_heap_address = u64::from_le_bytes(*map_heap_address_bytes);
                let map_layout = map_layout(map_heap_address, *key_size, *value_size, heap)?;

                let hash = hash_of(&key_data, map_layout.hash_seed);
                let top_hash = top_hash(hash);
                let bucket_idx = bucket_idx(hash, map_layout.log_cap) as u64;

                // get address of the bucket
                let bucket_address =
                    map_layout.ptr_buckets + bucket_idx * map_layout.bucket_size as u64;

                let bucket_layout = bucket_layout(bucket_address, *key_size, *value_size, heap)?;

                let opt_ptr_value = bucket_layout.get(top_hash, &key_data, *ref_access, heap)?;
                match opt_ptr_value {
                    Some(ptr_value) => {
                        let value_data = heap.read(ptr_value as usize, *value_size)?;

                        let _ = stack.push_with(&value_data)?;
                        // push NO_ERROR
                        let _ = stack.push_with(&OK_VALUE)?;
                    }
                    None => {
                        let _ = stack.push_with(&vec![0u8; *value_size])?;
                        // push ERROR
                        let _ = stack.push_with(&ERROR_VALUE)?;
                    }
                }
            }
            AllocCasm::DeleteMapKey {
                key_size,
                value_size,
                ref_access,
            } => {
                let key_data = match ref_access {
                    DerefHashing::Vec(item_size) => {
                        let vec_heap_address = OpPrimitive::get_num8::<u64>(stack)?;

                        let len_bytes = heap.read(vec_heap_address as usize, 8)?;
                        let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                            .map_err(|_| RuntimeError::Deserialization)?;
                        let len = u64::from_le_bytes(*len_bytes);
                        let items_bytes =
                            heap.read(vec_heap_address as usize + 16, len as usize * *item_size)?;
                        items_bytes
                    }
                    DerefHashing::String => {
                        let str_heap_address = OpPrimitive::get_num8::<u64>(stack)?;
                        let len_bytes = heap.read(str_heap_address as usize, 8)?;
                        let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                            .map_err(|_| RuntimeError::Deserialization)?;
                        let len = u64::from_le_bytes(*len_bytes);
                        let items_bytes =
                            heap.read(str_heap_address as usize + 16, len as usize)?;
                        items_bytes
                    }
                    DerefHashing::Default => stack.pop(*key_size)?.to_vec(),
                };

                let map_stack_address = OpPrimitive::get_num8::<u64>(stack)?;

                let map_heap_address_bytes = stack.read(
                    Offset::SB(map_stack_address as usize),
                    AccessLevel::Direct,
                    8,
                )?;
                let map_heap_address_bytes = TryInto::<&[u8; 8]>::try_into(map_heap_address_bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                let map_heap_address = u64::from_le_bytes(*map_heap_address_bytes);
                let map_layout = map_layout(map_heap_address, *key_size, *value_size, heap)?;

                let hash = hash_of(&key_data, map_layout.hash_seed);
                let top_hash = top_hash(hash);
                let bucket_idx = bucket_idx(hash, map_layout.log_cap) as u64;

                // get address of the bucket
                let bucket_address =
                    map_layout.ptr_buckets + bucket_idx * map_layout.bucket_size as u64;

                let bucket_layout = bucket_layout(bucket_address, *key_size, *value_size, heap)?;

                let opt_ptr_value = bucket_layout.delete(top_hash, &key_data, *ref_access, heap)?;
                match opt_ptr_value {
                    Some(ptr_value) => {
                        // update len
                        let _ = heap.write(
                            map_heap_address as usize,
                            &(map_layout.len - 1).to_le_bytes().to_vec(),
                        )?;
                        // read in found place
                        let value_data = heap.read(ptr_value as usize, *value_size)?;

                        let _ = stack.push_with(&value_data)?;
                        // push NO_ERROR
                        let _ = stack.push_with(&OK_VALUE)?;
                    }
                    None => {
                        let _ = stack.push_with(&vec![0u8; *value_size])?;
                        // push ERROR
                        let _ = stack.push_with(&ERROR_VALUE)?;
                    }
                }
            }
            AllocCasm::DeleteVec(item_size) => {
                let index = OpPrimitive::get_num8::<u64>(stack)?;
                let vec_stack_address = OpPrimitive::get_num8::<u64>(stack)?;

                let vec_heap_address_bytes = stack.read(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    8,
                )?;

                let vec_heap_address_bytes = TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);
                let previous_len_bytes = heap.read(vec_heap_address as usize, 8)?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let item_size = *item_size as u64;

                if index < previous_len {
                    // Read deleted item
                    let deleted_item_data = heap.read(
                        vec_heap_address as usize + 16 + ((index) * item_size) as usize,
                        item_size as usize,
                    )?;
                    if index < previous_len - 1
                    /* index not last item */
                    {
                        /* move below */
                        let data = heap.read(
                            vec_heap_address as usize + 16 + ((index + 1) * item_size) as usize,
                            (previous_len * item_size - (index + 1) * item_size) as usize,
                        )?;
                        let _ = heap.write(
                            vec_heap_address as usize + 16 + (index * item_size) as usize,
                            &data,
                        )?;
                    }
                    /* clear last item */
                    let _ = heap.write(
                        vec_heap_address as usize + 16 + ((previous_len - 1) * item_size) as usize,
                        &vec![0; item_size as usize],
                    )?;

                    let len_bytes = (previous_len - 1).to_le_bytes().as_slice().to_vec();
                    /* Write len */
                    let _ = heap.write(vec_heap_address as usize, &len_bytes)?;

                    // Push deleted item and error
                    let _ = stack.push_with(&deleted_item_data)?;
                    // Push no error
                    let _ = stack.push_with(&OK_VALUE)?;
                } else {
                    // Push zeroes and error
                    let _ = stack.push_with(&vec![0; item_size as usize])?;
                    // Push no error
                    let _ = stack.push_with(&ERROR_VALUE)?;
                }
            }
            AllocCasm::Vec { item_size } | AllocCasm::VecWithCapacity { item_size } => {
                /* */
                let with_capacity = match &self {
                    AllocCasm::Vec { .. } => false,
                    AllocCasm::VecWithCapacity { .. } => true,
                    _ => unreachable!(),
                };
                let (len, cap) = if with_capacity {
                    let cap = OpPrimitive::get_num8::<u64>(stack)?;
                    let len = OpPrimitive::get_num8::<u64>(stack)?;
                    (len, cap)
                } else {
                    let len = OpPrimitive::get_num8::<u64>(stack)?;
                    (len, align(len as usize) as u64)
                };
                let alloc_size = cap * (*item_size as u64) + 16;

                let len_bytes = len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                let address = heap.alloc(alloc_size as usize)?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

                /* Write len */
                let _ = heap.write(address, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(address + 8, &cap_bytes)?;

                let _ = stack.push_with(&address.to_le_bytes())?;
            }
            AllocCasm::Map {
                key_size,
                value_size,
            }
            | AllocCasm::MapWithCapacity {
                key_size,
                value_size,
            } => {
                /* */
                let with_capacity = match &self {
                    AllocCasm::Map { .. } => false,
                    AllocCasm::MapWithCapacity { .. } => true,
                    _ => unreachable!(),
                };
                let mut log_cap: u8 = 0;
                if with_capacity {
                    let cap = OpPrimitive::get_num8::<u64>(stack)?;
                    // to reduce cap size and therefore number of created bucket -> the map will try to fill up buckets in priority rather than reallocating
                    if cap <= MAP_BUCKET_SIZE as u64 {
                        log_cap = 0;
                    } else {
                        log_cap = ((cap as f64 / MAP_BUCKET_SIZE as f64).log2().ceil() as u64)
                            .try_into()
                            .map_err(|_| RuntimeError::Deserialization)?;
                    }
                    // while over_load_factor(cap, log_cap) {
                    //     log_cap += 1;
                    // }
                } else {
                    log_cap = 0;
                }

                let map = MapLayout::new(*key_size, *value_size, log_cap);
                let map_ptr = map.init_in_mem(heap)?;
                let _ = stack.push_with(&(map_ptr as u64).to_le_bytes())?;
            }
            AllocCasm::StringFromSlice => {
                let len = OpPrimitive::get_num8::<u64>(stack)?;
                let cap = align(len as usize) as u64;
                let alloc_size = cap + 16;

                let len_bytes = len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                let address = heap.alloc(alloc_size as usize)?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

                let data = stack.pop(len as usize)?;
                /* Write len */
                let _ = heap.write(address, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(address + 8, &cap_bytes)?;
                /* Write slice */
                let _ = heap.write(address + 16, &data.to_vec())?;

                let _ = stack.push_with(&address.to_le_bytes())?;
            }
            AllocCasm::StringFromChar => {
                let chara = OpPrimitive::get_char(stack)?;
                let chara = chara.to_string();
                let chara = chara.as_bytes();

                let len = chara.len() as u64;
                let cap = align(len as usize) as u64;
                let alloc_size = cap + 16;

                let len_bytes = len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                let address = heap.alloc(alloc_size as usize)?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

                /* Write len */
                let _ = heap.write(address, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(address + 8, &cap_bytes)?;
                /* Write slice */
                let _ = heap.write(address + 16, &chara.to_vec())?;

                let _ = stack.push_with(&address.to_le_bytes())?;
            }
            AllocCasm::Len => {
                let vec_heap_address = OpPrimitive::get_num8::<u64>(stack)?;

                let len_bytes = heap.read(vec_heap_address as usize, 8)?;
                let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                    .map_err(|_| RuntimeError::Deserialization)?;
                let len = u64::from_le_bytes(*len_bytes);

                let _ = stack.push_with(&len.to_le_bytes())?;
            }
            AllocCasm::Cap => {
                let vec_heap_address = OpPrimitive::get_num8::<u64>(stack)?;

                let cap_bytes = heap.read(vec_heap_address as usize + 8, 8)?;
                let cap_bytes = TryInto::<&[u8; 8]>::try_into(cap_bytes.as_slice())
                    .map_err(|_| RuntimeError::Deserialization)?;
                let cap = u64::from_le_bytes(*cap_bytes);

                let _ = stack.push_with(&cap.to_le_bytes())?;
            }
            AllocCasm::CapMap => {
                let vec_heap_address = OpPrimitive::get_num8::<u64>(stack)?;

                let log_cap_bytes = heap.read(vec_heap_address as usize + 8, 1)?;
                let log_cap = log_cap_bytes[0];

                let cap = if log_cap == 1 {
                    MAP_BUCKET_SIZE
                } else {
                    (1u64 << log_cap) as usize * MAP_BUCKET_SIZE
                };
                let _ = stack.push_with(&(cap as u64).to_le_bytes())?;
            }
            AllocCasm::ClearVec(item_size) | AllocCasm::ClearString(item_size) => {
                let vec_stack_address = OpPrimitive::get_num8::<u64>(stack)?;

                let vec_heap_address_bytes = stack.read(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    8,
                )?;

                let vec_heap_address_bytes = TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);
                let previous_len_bytes = heap.read(vec_heap_address as usize, 8)?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let item_size = *item_size as u64;

                /* clear */
                let _ = heap.write(
                    vec_heap_address as usize + 16,
                    &vec![0; (previous_len * item_size) as usize],
                )?;

                let len_bytes = 0u64.to_le_bytes().as_slice().to_vec();
                /* Write len */
                let _ = heap.write(vec_heap_address as usize, &len_bytes)?;
            }
            AllocCasm::ClearMap {
                key_size,
                value_size,
            } => {
                let map_stack_address = OpPrimitive::get_num8::<u64>(stack)?;

                let map_heap_address_bytes = stack.read(
                    Offset::SB(map_stack_address as usize),
                    AccessLevel::Direct,
                    8,
                )?;

                let map_heap_address_bytes = TryInto::<&[u8; 8]>::try_into(map_heap_address_bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                let map_heap_address = u64::from_le_bytes(*map_heap_address_bytes);
                let map_layout = map_layout(map_heap_address, *key_size, *value_size, heap)?;
                let _ = map_layout.clear_buckets(heap)?;
                // update len
                let _ = heap.write(map_heap_address as usize, &(0u64).to_le_bytes().to_vec())?;
            }
            AllocCasm::ExtendItemFromSlice { size, len } => {
                let item_size = *size;
                let slice_len = *len;
                let slice_data = stack.pop(item_size * slice_len)?.to_owned();

                let vec_stack_address = OpPrimitive::get_num8::<u64>(stack)?;

                let vec_heap_address_bytes = stack.read(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    8,
                )?;
                let vec_heap_address_bytes = TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);

                let previous_len_bytes = heap.read(vec_heap_address as usize, 8)?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let previous_cap_bytes = heap.read(vec_heap_address as usize + 8, 8)?;
                let previous_cap_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_cap_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_cap = u64::from_le_bytes(*previous_cap_bytes);

                let len_offset = slice_len as u64;
                let size_factor = item_size;
                let (new_vec_heap_address, new_len, new_cap) = if previous_len + len_offset
                    >= previous_cap
                {
                    /* Reallocation */
                    let size = align(((previous_len + len_offset) * 2) as usize * size_factor + 16);
                    let address = heap.realloc(vec_heap_address as usize - 8, size)?;
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
                let _ = heap.write(new_vec_heap_address as usize, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(new_vec_heap_address as usize + 8, &cap_bytes)?;

                /* Write new items */
                let _ = heap.write(
                    new_vec_heap_address as usize
                        + 16
                        + (previous_len as usize * size_factor as usize),
                    &slice_data.to_vec(),
                )?;

                /* Update vector pointer */
                let _ = stack.write(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    &new_vec_heap_address.to_le_bytes(),
                )?;
            }
            AllocCasm::ExtendItemFromVec { size } => {
                let item_size = *size;

                let other_heap_address = OpPrimitive::get_num8::<u64>(stack)?;
                let previous_len_bytes = heap.read(other_heap_address as usize, 8)?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let slice_len = u64::from_le_bytes(*previous_len_bytes);
                let slice_data = heap.read(
                    other_heap_address as usize + 16,
                    slice_len as usize * item_size,
                )?;

                let vec_stack_address = OpPrimitive::get_num8::<u64>(stack)?;

                let vec_heap_address_bytes = stack.read(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    8,
                )?;
                let vec_heap_address_bytes = TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);

                let previous_len_bytes = heap.read(vec_heap_address as usize, 8)?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let previous_cap_bytes = heap.read(vec_heap_address as usize + 8, 8)?;
                let previous_cap_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_cap_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_cap = u64::from_le_bytes(*previous_cap_bytes);

                let len_offset = slice_len as u64;
                let size_factor = item_size;
                let (new_vec_heap_address, new_len, new_cap) = if previous_len + len_offset
                    >= previous_cap
                {
                    /* Reallocation */
                    let size = align(((previous_len + len_offset) * 2) as usize * size_factor + 16);
                    let address = heap.realloc(vec_heap_address as usize - 8, size)?;
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
                let _ = heap.write(new_vec_heap_address as usize, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(new_vec_heap_address as usize + 8, &cap_bytes)?;

                /* Write new items */
                let _ = heap.write(
                    new_vec_heap_address as usize
                        + 16
                        + (previous_len as usize * size_factor as usize),
                    &slice_data,
                )?;

                /* Update vector pointer */
                let _ = stack.write(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    &new_vec_heap_address.to_le_bytes(),
                )?;
            }
            AllocCasm::ExtendStringFromSlice { len } => {
                let mut slice_data = Vec::new();
                for _ in 0..*len {
                    let string_heap_address = OpPrimitive::get_num8::<u64>(stack)?;

                    let string_len_bytes = heap.read(string_heap_address as usize, 8)?;
                    let string_len_bytes =
                        TryInto::<&[u8; 8]>::try_into(string_len_bytes.as_slice())
                            .map_err(|_| RuntimeError::Deserialization)?;
                    let string_len = u64::from_le_bytes(*string_len_bytes);
                    let string_data =
                        heap.read(string_heap_address as usize + 16, string_len as usize)?;
                    slice_data.push(string_data);
                }
                let slice_data = slice_data.into_iter().rev().flatten().collect::<Vec<u8>>();
                let slice_len = slice_data.len();

                let vec_stack_address = OpPrimitive::get_num8::<u64>(stack)?;

                let vec_heap_address_bytes = stack.read(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    8,
                )?;
                let vec_heap_address_bytes = TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);

                let previous_len_bytes = heap.read(vec_heap_address as usize, 8)?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let previous_cap_bytes = heap.read(vec_heap_address as usize + 8, 8)?;
                let previous_cap_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_cap_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_cap = u64::from_le_bytes(*previous_cap_bytes);

                let len_offset = slice_len as u64;
                let size_factor = 1;
                let (new_vec_heap_address, new_len, new_cap) = if previous_len + len_offset
                    >= previous_cap
                {
                    /* Reallocation */
                    let size = align(((previous_len + len_offset) * 2) as usize * size_factor + 16);
                    let address = heap.realloc(vec_heap_address as usize - 8, size)?;
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
                let _ = heap.write(new_vec_heap_address as usize, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(new_vec_heap_address as usize + 8, &cap_bytes)?;

                /* Write new items */
                let _ = heap.write(
                    new_vec_heap_address as usize
                        + 16
                        + (previous_len as usize * size_factor as usize),
                    &slice_data,
                )?;

                /* Update vector pointer */
                let _ = stack.write(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    &new_vec_heap_address.to_le_bytes(),
                )?;
            }
            AllocCasm::ExtendStringFromVec => {
                let other_heap_address = OpPrimitive::get_num8::<u64>(stack)?;
                let previous_len_bytes = heap.read(other_heap_address as usize, 8)?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let other_len = u64::from_le_bytes(*previous_len_bytes);

                let mut slice_data = Vec::new();
                for i in 0..other_len as usize {
                    let string_heap_address =
                        heap.read(other_heap_address as usize + 16 + 8 * i, 8)?;
                    let string_heap_address =
                        TryInto::<&[u8; 8]>::try_into(string_heap_address.as_slice())
                            .map_err(|_| RuntimeError::Deserialization)?;
                    let string_heap_address = u64::from_le_bytes(*string_heap_address);

                    let string_len_bytes = heap.read(string_heap_address as usize, 8)?;
                    let string_len_bytes =
                        TryInto::<&[u8; 8]>::try_into(string_len_bytes.as_slice())
                            .map_err(|_| RuntimeError::Deserialization)?;
                    let string_len = u64::from_le_bytes(*string_len_bytes);
                    let string_data =
                        heap.read(string_heap_address as usize + 16, string_len as usize)?;
                    slice_data.extend(string_data);
                }
                let slice_len = slice_data.len();

                let vec_stack_address = OpPrimitive::get_num8::<u64>(stack)?;

                let vec_heap_address_bytes = stack.read(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    8,
                )?;
                let vec_heap_address_bytes = TryInto::<&[u8; 8]>::try_into(vec_heap_address_bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                let vec_heap_address = u64::from_le_bytes(*vec_heap_address_bytes);

                let previous_len_bytes = heap.read(vec_heap_address as usize, 8)?;
                let previous_len_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_len_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_len = u64::from_le_bytes(*previous_len_bytes);

                let previous_cap_bytes = heap.read(vec_heap_address as usize + 8, 8)?;
                let previous_cap_bytes =
                    TryInto::<&[u8; 8]>::try_into(previous_cap_bytes.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                let previous_cap = u64::from_le_bytes(*previous_cap_bytes);

                let len_offset = slice_len as u64;
                let size_factor = 1;
                let (new_vec_heap_address, new_len, new_cap) = if previous_len + len_offset
                    >= previous_cap
                {
                    /* Reallocation */
                    let size = align(((previous_len + len_offset) * 2) as usize * size_factor + 16);
                    let address = heap.realloc(vec_heap_address as usize - 8, size)?;
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
                let _ = heap.write(new_vec_heap_address as usize, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(new_vec_heap_address as usize + 8, &cap_bytes)?;

                /* Write new items */
                let _ = heap.write(
                    new_vec_heap_address as usize
                        + 16
                        + (previous_len as usize * size_factor as usize),
                    &slice_data,
                )?;

                /* Update vector pointer */
                let _ = stack.write(
                    Offset::SB(vec_stack_address as usize),
                    AccessLevel::Direct,
                    &new_vec_heap_address.to_le_bytes(),
                )?;
            }
        }

        program.incr();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive},
                Atomic,
            },
            statements::Statement,
            TryParse,
        },
        clear_stack, compile_statement, compile_statement_for_string,
        semantic::scope::scope::Scope,
        v_num,
        vm::vm::{DeserializeFrom, Runtime},
    };

    use super::*;

    #[test]
    fn valid_string() {
        let mut statement = Statement::parse(
            r##"
        let x = string("Hello World");
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let result = compile_statement_for_string!(statement);

        assert_eq!(result, "Hello World");
    }

    #[test]
    fn valid_vec() {
        let mut statement = Statement::parse(
            r##"
        let x:Vec<u64> = vec(8);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = heap
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
        let mut statement = Statement::parse(
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
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 420));
    }
    #[test]
    fn valid_append() {
        let mut statement = Statement::parse(
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
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 9);

        let data = heap
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
        let mut statement = Statement::parse(
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
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 9);

        let data = heap
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
        let mut statement = Statement::parse(
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

        let result = compile_statement_for_string!(statement);

        assert_eq!(result, "Hello World");
    }

    #[test]
    fn valid_append_str_char() {
        let mut statement = Statement::parse(
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

        let result = compile_statement_for_string!(statement);

        assert_eq!(result, "Hello World");
    }

    #[test]
    fn valid_append_str_char_complex() {
        let mut statement = Statement::parse(
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

        let result = compile_statement_for_string!(statement);

        assert_eq!(result, "Hello Worl仗");
    }

    #[test]
    fn valid_append_str_str() {
        let mut statement = Statement::parse(
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

        let result = compile_statement_for_string!(statement);

        assert_eq!(result, "Hello World");
    }

    #[test]
    fn valid_len_string() {
        let mut statement = Statement::parse(
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
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 11));
    }

    #[test]
    fn valid_cap_string() {
        let mut statement = Statement::parse(
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
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 16));
    }

    #[test]
    fn valid_len_vec() {
        let mut statement = Statement::parse(
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

        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 11));
    }

    #[test]
    fn valid_cap_vec() {
        let mut statement = Statement::parse(
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

        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 16));
    }
    #[test]
    fn valid_free() {
        let mut statement = Statement::parse(
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
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        assert_eq!(heap.allocated_size(), 0);
    }

    #[test]
    fn valid_alloc() {
        let mut statement = Statement::parse(
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
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        assert_eq!(heap.allocated_size(), 16);

        let data = clear_stack!(memory);
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data = heap
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
        let mut statement = Statement::parse(
            r##"
            let res = {
                let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
                let (old,err) = delete(&x,7);
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 7);

        let data = heap
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
        let mut statement = Statement::parse(
            r##"
            let res = {
                let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
                let (old,err) = delete(&x,2);
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 7);

        let data = heap
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
    fn robustness_delete_vec() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
                let (old,err) = delete(&x,15);
                return err;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let _ = compile_statement!(statement);
        let data = compile_statement!(statement);
        let result = data.first().unwrap();

        assert_eq!(*result, 1u8, "Result does not match the expected value");
    }

    #[test]
    fn valid_size_of_type() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                struct Point {
                    x : u64,
                    y : u64,
                }
                return size_of(Point);
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 16));
    }

    #[test]
    fn valid_size_of_expr() {
        let mut statement = Statement::parse(
            r##"
            let res = size_of(420u64);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 8));
    }

    #[test]
    fn valid_memcpy_heap() {
        let mut statement = Statement::parse(
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
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 8);

        let data = heap
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
        let mut statement = Statement::parse(
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
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let mut statement = Statement::parse(
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
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 0);

        let data = heap
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
        let mut statement = Statement::parse(
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
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data_length = heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data = heap
            .read(heap_address, 5 + 16)
            .expect("length should be readable");

        let result = <StringType as DeserializeFrom>::deserialize_from(&StringType(), &data)
            .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "");
    }

    #[test]
    fn valid_alloc_cast() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                struct Point {
                    x : i64,
                    y: i64
                }

                let point_any : &Any = alloc(size_of(Point)) as &Any;

                let copy = Point {
                    x : 420,
                    y : 69,
                };
                memcpy(point_any,&copy,size_of(Point));
                
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
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 420));
    }

    #[test]
    fn valid_alloc_modify() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                struct Point {
                    x : i64,
                    y: i64
                }

                let point : &Point = alloc(size_of(Point)) as &Point;

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
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 420));
    }

    #[test]
    fn valid_extend_vec_from_slice() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let slice = [5,6,7,8];
                let vector:Vec<i64> = vec(8);
                extend(&vector,slice);
                return vector[8];
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 5));
    }

    #[test]
    fn valid_extend_vec_from_vec() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let vec1 = vec[5,6,7,8];
                let vector:Vec<i64> = vec(8);
                extend(&vector,vec1);
                return vector[8];
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 5));
    }

    #[test]
    fn valid_extend_string_from_slice() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let vec1 = [string("lo"),string(" Wor"),string("ld")];
                let hello = string("Hel");
                extend(&hello,vec1);
                return hello;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data_length = heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data = heap
            .read(heap_address, length + 16)
            .expect("length should be readable");

        let result = <StringType as DeserializeFrom>::deserialize_from(&StringType(), &data)
            .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "Hello World");
    }

    #[test]
    fn valid_extend_string_from_vec() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let vec1 = vec[string("lo"),string(" Wor"),string("ld")];
                let hello = string("Hel");
                extend(&hello,vec1);
                return hello;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
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
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data_length = heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data = heap
            .read(heap_address, length + 16)
            .expect("length should be readable");

        let result = <StringType as DeserializeFrom>::deserialize_from(&StringType(), &data)
            .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "Hello World");
    }

    #[test]
    fn valid_map_init_no_cap() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map();
                return true;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let _ = compile_statement!(statement);
    }

    #[test]
    fn valid_map_init_with_cap() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map(8);
                return true;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let _ = compile_statement!(statement);
    }

    #[test]
    fn valid_map_len_empty() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map(8);
                return len(hmap);
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(
            result,
            v_num!(U64, 0),
            "Result does not match the expected value"
        );
    }

    #[test]
    fn valid_map_len() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map();
                insert(&hmap,101,5);
                insert(&hmap,102,6);
                insert(&hmap,103,7);
                insert(&hmap,104,8);
                insert(&hmap,105,9);
                insert(&hmap,106,10);
                insert(&hmap,107,11);
                insert(&hmap,108,12);
                return len(hmap);
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(
            result,
            v_num!(U64, 8),
            "Result does not match the expected value"
        );
    }

    #[test]
    fn valid_map_len_with_upsert() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map();
                insert(&hmap,101,5);
                insert(&hmap,102,6);
                insert(&hmap,103,7);
                insert(&hmap,103,8);
                insert(&hmap,105,9);
                insert(&hmap,103,10);
                insert(&hmap,107,11);
                insert(&hmap,103,12);
                return len(hmap);
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(
            result,
            v_num!(U64, 5),
            "Result does not match the expected value"
        );
    }
    #[test]
    fn valid_map_cap() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map(8);
                return cap(hmap);
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(
            result,
            v_num!(U64, 8),
            "Result does not match the expected value"
        );
    }

    #[test]
    fn valid_map_cap_over_min() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map(60);
                return cap(hmap);
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(
            result,
            v_num!(U64, 64),
            "Result does not match the expected value"
        );
    }

    #[test]
    fn valid_map_access() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map(64);
                insert(&hmap,420,69);
                let (value,err) = get(&hmap,420);
                assert(err);
                return value;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let _ = compile_statement!(statement);
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(
            result,
            v_num!(U64, 69),
            "Result does not match the expected value"
        );
    }

    #[test]
    fn valid_map_access_complex() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map(64);
                insert(&hmap,420,69);
                return get(&hmap,420).0;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let _ = compile_statement!(statement);
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(
            result,
            v_num!(U64, 69),
            "Result does not match the expected value"
        );
    }
    #[test]
    fn robustness_map_access() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map(64);
                let (value,err) = get(&hmap,420);
                return err;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let _ = compile_statement!(statement);
        let data = compile_statement!(statement);
        let result = data.first().unwrap();

        assert_eq!(*result, 1u8, "Result does not match the expected value");
    }

    #[test]
    fn valid_map_insert_resize() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map();
                insert(&hmap,101,5);
                insert(&hmap,102,6);
                insert(&hmap,103,7);
                insert(&hmap,104,8);
                insert(&hmap,105,9);
                insert(&hmap,106,10);
                insert(&hmap,107,11);
                insert(&hmap,108,12);
                insert(&hmap,109,13);
                insert(&hmap,110,14);
                let (value,err) = get(&hmap,103);
                assert(err);
                return value;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let _ = compile_statement!(statement);
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(
            result,
            v_num!(U64, 7),
            "Result does not match the expected value"
        );
    }

    #[test]
    fn valid_map_upsert_resize() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map();
                insert(&hmap,101,5);
                insert(&hmap,102,6);
                insert(&hmap,103,7);
                insert(&hmap,104,8);
                insert(&hmap,105,9);
                insert(&hmap,106,10);
                insert(&hmap,107,11);
                insert(&hmap,108,12);
                insert(&hmap,109,13);
                insert(&hmap,110,14);
                insert(&hmap,103,420);
                let (value,err) = get(&hmap,103);
                assert(err);
                return value;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let _ = compile_statement!(statement);
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(
            result,
            v_num!(U64, 420),
            "Result does not match the expected value"
        );
    }

    #[test]
    fn valid_map_insert() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map(64);
                insert(&hmap,420,69);
                return len(hmap);
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(
            result,
            v_num!(U64, 1),
            "Result does not match the expected value"
        );
    }

    #[test]
    fn valid_map_delete() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map(64);
                insert(&hmap,420,69);
                insert(&hmap,120,75);
                let (value,err) = delete(&hmap,420);
                assert(err);
                return (value,len(hmap));
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result =
            <crate::semantic::scope::static_types::TupleType as DeserializeFrom>::deserialize_from(
                &crate::semantic::scope::static_types::TupleType(vec![p_num!(U64), p_num!(U64)]),
                &data,
            )
            .expect("Deserialization should have succeeded");
        let value = &result.value[0];
        let value = match value {
            Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(v)))) => match v {
                Number::U64(n) => n,
                _ => unreachable!("Should be a u64"),
            },
            _ => unreachable!("Should be a u64"),
        };
        let len = &result.value[1];
        let len = match len {
            Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(v)))) => match v {
                Number::U64(n) => n,
                _ => unreachable!("Should be a u64"),
            },
            _ => unreachable!("Should be a u64"),
        };
        assert_eq!(*value, 69);
        assert_eq!(*len, 1);
    }

    #[test]
    fn valid_map_clear() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map();
                insert(&hmap,101,5);
                insert(&hmap,102,6);
                insert(&hmap,103,7);
                insert(&hmap,104,8);
                insert(&hmap,105,9);
                insert(&hmap,106,10);
                insert(&hmap,107,11);
                insert(&hmap,108,12);
                clear(&hmap);
                return len(hmap);
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(
            result,
            v_num!(U64, 0),
            "Result does not match the expected value"
        );
    }
    #[test]
    fn valid_map_delete_cant_read_after() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<u64,u64> = map(64);
                insert(&hmap,420,69);
                insert(&hmap,120,75);
                let (value,err) = delete(&hmap,420);
                assert(err);
                let (value,err) = get(&hmap,420);
                return err;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let _ = compile_statement!(statement);
        let data = compile_statement!(statement);
        let result = data.first().unwrap();

        assert_eq!(*result, 1u8, "Result does not match the expected value");
    }

    #[test]
    fn valid_map_insert_key_str() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<str<10>,u64> = map(64);
                insert(&hmap,"test",69);
                insert(&hmap,"test1",80);
                insert(&hmap,"test11",46);
                insert(&hmap,"test111",16);
                let (value,err) = get(&hmap,"test1");
                assert(err);
                return value;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(
            result,
            v_num!(U64, 80),
            "Result does not match the expected value"
        );
    }

    #[test]
    fn valid_map_insert_key_string() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let hmap : Map<String,u64> = map(64);
                insert(&hmap,string("test"),69);
                insert(&hmap,string("test1"),80);
                insert(&hmap,string("test11"),46);
                insert(&hmap,string("test111"),16);
                insert(&hmap,f"test11{52}",28);
                let (value,err) = get(&hmap,string("test1"));
                assert(err);
                return value;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(
            result,
            v_num!(U64, 80),
            "Result does not match the expected value"
        );
    }
    #[test]
    fn valid_map_upsert_key_string() {
        let mut statement = Statement::parse(
            r##"
            let res = {
                let x = 1;
                let hmap : Map<String,u64> = map(64);

                insert(&hmap,string("test"),69);
                insert(&hmap,string("test1"),80);
                insert(&hmap,string("test11"),46);
                insert(&hmap,string("test111"),16);
                
                insert(&hmap,f"test{x}",28);

                let (value,err) = get(&hmap,string("test1"));
                assert(err);
                return value;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(
            result,
            v_num!(U64, 28),
            "Result does not match the expected value"
        );
    }
}
