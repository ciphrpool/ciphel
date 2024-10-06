use num_traits::ToBytes;

use crate::{
    ast::{expressions::Expression, utils::strings::ID},
    e_static,
    semantic::{
        scope::{
            scope::Scope,
            static_types::{AddrType, MapType, StaticType, TupleType, VecType},
        },
        AccessLevel, EType, Either, Info, Metadata, Resolve, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        allocator::{
            align,
            heap::Heap,
            stack::{Offset, Stack},
        },
        casm::{operation::OpPrimitive, Casm, CasmProgram},
        platform::{core::alloc::map_impl, utils::lexem, LibCasm},
        stdio::StdIO,
        vm::{CasmMetadata, CodeGenerationError, Executable, GenerateCode, RuntimeError},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum IterFn {
    MapItems {
        metadata: Metadata,
        key_size: usize,
        value_size: usize,
    },
    MapValues {
        metadata: Metadata,
        key_size: usize,
        value_size: usize,
    },
    MapKeys {
        metadata: Metadata,
        key_size: usize,
        value_size: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum IterCasm {
    MapItems { key_size: usize, value_size: usize },
    MapValues { key_size: usize, value_size: usize },
    MapKeys { key_size: usize, value_size: usize },
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for IterCasm {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            IterCasm::MapItems { .. } => stdio.push_casm_lib(engine, "items"),
            IterCasm::MapValues { .. } => stdio.push_casm_lib(engine, "values"),
            IterCasm::MapKeys { .. } => stdio.push_casm_lib(engine, "keys"),
        }
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        crate::vm::vm::CasmWeight::HIGH
    }
}

impl IterFn {
    pub fn from(suffixe: &Option<ID>, id: &ID) -> Option<Self> {
        match suffixe {
            Some(suffixe) => {
                if **suffixe != lexem::STD {
                    return None;
                }
            }
            None => {}
        }
        match id.as_str() {
            lexem::ITEMS => Some(IterFn::MapItems {
                key_size: 0,
                value_size: 0,
                metadata: Metadata::default(),
            }),
            lexem::KEYS => Some(IterFn::MapKeys {
                key_size: 0,
                value_size: 0,
                metadata: Metadata::default(),
            }),
            lexem::VALUES => Some(IterFn::MapValues {
                key_size: 0,
                value_size: 0,
                metadata: Metadata::default(),
            }),
            _ => None,
        }
    }
}

impl Resolve for IterFn {
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
            IterFn::MapItems {
                metadata,
                key_size,
                value_size,
            } => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let map = &mut extra[0];
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
                            *key_size = keys_type.as_ref().size_of();
                            *value_size = values_type.as_ref().size_of();

                            metadata.info = Info::Resolved {
                                context: context.clone(),
                                signature: Some(e_static!(StaticType::Vec(VecType(Box::new(
                                    e_static!(StaticType::Tuple(TupleType(vec![
                                        e_static!(StaticType::Address(AddrType(Box::new(
                                            keys_type.as_ref().clone()
                                        )))),
                                        e_static!(StaticType::Address(AddrType(Box::new(
                                            values_type.as_ref().clone()
                                        ))))
                                    ])))
                                ))))),
                            };
                            Ok(())
                        }
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
            }
            IterFn::MapValues {
                metadata,
                key_size,
                value_size,
            } => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let map = &mut extra[0];
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
                            *key_size = keys_type.as_ref().size_of();
                            *value_size = values_type.as_ref().size_of();

                            metadata.info = Info::Resolved {
                                context: context.clone(),
                                signature: Some(e_static!(StaticType::Vec(VecType(Box::new(
                                    e_static!(StaticType::Address(AddrType(Box::new(
                                        values_type.as_ref().clone()
                                    ))))
                                ))))),
                            };
                            Ok(())
                        }
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
            }
            IterFn::MapKeys {
                metadata,
                key_size,
                value_size,
            } => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let map = &mut extra[0];
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
                            *key_size = keys_type.as_ref().size_of();
                            *value_size = values_type.as_ref().size_of();

                            metadata.info = Info::Resolved {
                                context: context.clone(),
                                signature: Some(e_static!(StaticType::Vec(VecType(Box::new(
                                    e_static!(StaticType::Address(AddrType(Box::new(
                                        keys_type.as_ref().clone()
                                    ))))
                                ))))),
                            };
                            Ok(())
                        }
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
            }
        }
    }
}

impl TypeOf for IterFn {
    fn type_of(&self, _scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            IterFn::MapItems { metadata, .. } => {
                metadata.signature().ok_or(SemanticError::NotResolvedYet)
            }
            IterFn::MapValues { metadata, .. } => {
                metadata.signature().ok_or(SemanticError::NotResolvedYet)
            }
            IterFn::MapKeys { metadata, .. } => {
                metadata.signature().ok_or(SemanticError::NotResolvedYet)
            }
        }
    }
}

impl GenerateCode for IterFn {
    fn gencode(
        &self,
        _scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            IterFn::MapItems {
                metadata,
                key_size,
                value_size,
            } => instructions.push(Casm::Platform(LibCasm::Std(super::StdCasm::Iter(
                IterCasm::MapItems {
                    key_size: *key_size,
                    value_size: *value_size,
                },
            )))),
            IterFn::MapValues {
                metadata,
                key_size,
                value_size,
            } => instructions.push(Casm::Platform(LibCasm::Std(super::StdCasm::Iter(
                IterCasm::MapValues {
                    key_size: *key_size,
                    value_size: *value_size,
                },
            )))),
            IterFn::MapKeys {
                metadata,
                key_size,
                value_size,
            } => instructions.push(Casm::Platform(LibCasm::Std(super::StdCasm::Iter(
                IterCasm::MapKeys {
                    key_size: *key_size,
                    value_size: *value_size,
                },
            )))),
        }
        Ok(())
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for IterCasm {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self {
            IterCasm::MapItems {
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
                let map_layout =
                    map_impl::map_layout(map_heap_address, *key_size, *value_size, heap)?;
                let items_ptr = map_layout.retrieve_vec_items(heap)?;

                let len = items_ptr.len() as u64;
                let cap = align(len as usize) as u64;
                let alloc_size = cap * (len * 16) + 16;

                let items_data: Vec<u8> = items_ptr
                    .into_iter()
                    .flat_map(|(key_ptr, value_ptr)| {
                        [key_ptr.to_le_bytes(), value_ptr.to_le_bytes()].concat()
                    })
                    .collect();

                let len_bytes = len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                let address = heap.alloc(alloc_size as usize)?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

                /* Write len */
                let _ = heap.write(address, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(address + 8, &cap_bytes)?;
                /* Write data */
                let _ = heap.write(address + 16, &items_data)?;

                let _ = stack.push_with(&address.to_le_bytes())?;
            }
            IterCasm::MapValues {
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
                let map_layout =
                    map_impl::map_layout(map_heap_address, *key_size, *value_size, heap)?;
                let values_ptr = map_layout.retrieve_vec_values(heap)?;

                let len = values_ptr.len() as u64;
                let cap = align(len as usize) as u64;
                let alloc_size = cap * (len * 16) + 16;

                let values_data: Vec<u8> = values_ptr
                    .into_iter()
                    .flat_map(|value_ptr| value_ptr.to_le_bytes())
                    .collect();

                let len_bytes = len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                let address = heap.alloc(alloc_size as usize)?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

                /* Write len */
                let _ = heap.write(address, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(address + 8, &cap_bytes)?;
                /* Write data */
                let _ = heap.write(address + 16, &values_data)?;

                let _ = stack.push_with(&address.to_le_bytes())?;
            }
            IterCasm::MapKeys {
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
                let map_layout =
                    map_impl::map_layout(map_heap_address, *key_size, *value_size, heap)?;
                let keys_ptr = map_layout.retrieve_vec_keys(heap)?;

                let len = keys_ptr.len() as u64;
                let cap = align(len as usize) as u64;
                let alloc_size = cap * (len * 16) + 16;

                let keys_data: Vec<u8> = keys_ptr
                    .into_iter()
                    .flat_map(|key_ptr| key_ptr.to_le_bytes())
                    .collect();

                let len_bytes = len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                let address = heap.alloc(alloc_size as usize)?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

                /* Write len */
                let _ = heap.write(address, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(address + 8, &cap_bytes)?;
                /* Write data */
                let _ = heap.write(address + 16, &keys_data)?;

                let _ = stack.push_with(&address.to_le_bytes())?;
            }
        }
        program.incr();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{statements::Statement, TryParse},
        compile_statement,
        semantic::scope::static_types::{NumberType, PrimitiveType},
        v_num,
        vm::vm::DeserializeFrom,
    };

    use super::*;

    #[test]
    fn valid_values() {
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

                let val = values(&hmap);
                let res = 0u64;
                for val_ptr in val {
                    res = res + *val_ptr;
                }

                return res;
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
            v_num!(U64, 5 + 6 + 7 + 8 + 9 + 10 + 11 + 12),
            "Result does not match the expected value"
        );
    }

    #[test]
    fn valid_keys() {
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

                let key = keys(&hmap);
                let res = 0u64;
                for key_ptr in key {
                    res = res + (*key_ptr);
                }

                return res;
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
            v_num!(U64, 101 + 102 + 103 + 104 + 105 + 106 + 107 + 108),
            "Result does not match the expected value"
        );
    }

    #[test]
    fn valid_items() {
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

                let items = std::items(&hmap);
                let res = 0u64;
                for (key_ptr,val_ptr) in items {
                    res = res + (*key_ptr) * (*val_ptr);
                }

                return res;
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
            v_num!(
                U64,
                101 * 5 + 102 * 6 + 103 * 7 + 104 * 8 + 105 * 9 + 106 * 10 + 107 * 11 + 108 * 12
            ),
            "Result does not match the expected value"
        );
    }
}
