use num_traits::ToBytes;

use crate::{
    ast::expressions::Expression,
    e_static,
    semantic::{
        scope::static_types::{AddrType, MapType, StaticType, TupleType, VecType},
        EType, Resolve, ResolveCore, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        allocator::{align, MemoryAddress},
        asm::{
            operation::{OpPrimitive, PopNum},
            Asm,
        },
        core::{lexem, map::map_layout},
        runtime::RuntimeError,
        scheduler::Executable,
        stdio::StdIO,
        GenerateCode,
    },
};

use super::PathFinder;

#[derive(Debug, Clone, PartialEq)]
pub enum IterFn {
    MapItems { key_size: usize, value_size: usize },
    MapValues { key_size: usize, value_size: usize },
    MapKeys { key_size: usize, value_size: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum IterAsm {
    MapItems { key_size: usize, value_size: usize },
    MapValues { key_size: usize, value_size: usize },
    MapKeys { key_size: usize, value_size: usize },
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for IterAsm {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            IterAsm::MapItems { .. } => stdio.push_asm_lib(engine, "items"),
            IterAsm::MapValues { .. } => stdio.push_asm_lib(engine, "values"),
            IterAsm::MapKeys { .. } => stdio.push_asm_lib(engine, "keys"),
        }
    }
}

impl crate::vm::AsmWeight for IterAsm {
    fn weight(&self) -> crate::vm::Weight {
        crate::vm::Weight::HIGH
    }
}

impl PathFinder for IterFn {
    fn find(path: &[String], name: &str) -> Option<Self>
    where
        Self: Sized,
    {
        if (path.len() == 1 && path[0] == lexem::ITER) || path.len() == 0 {
            return match name {
                lexem::ITEMS => Some(IterFn::MapItems {
                    key_size: 0,
                    value_size: 0,
                }),
                lexem::KEYS => Some(IterFn::MapKeys {
                    key_size: 0,
                    value_size: 0,
                }),
                lexem::VALUES => Some(IterFn::MapValues {
                    key_size: 0,
                    value_size: 0,
                }),
                _ => None,
            };
        }
        None
    }
}

impl ResolveCore for IterFn {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        match self {
            IterFn::MapItems {
                key_size,
                value_size,
            } => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let map = &mut parameters[0];
                let _ = map.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                let mut map_type = map.type_of(&scope_manager, scope_id)?;
                match &map_type {
                    EType::Static(value) => match value {
                        StaticType::Address(AddrType(sub)) => map_type = sub.as_ref().clone(),
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                match &map_type {
                    EType::Static(value) => match value {
                        StaticType::Map(MapType {
                            keys_type,
                            values_type,
                        }) => {
                            *key_size = keys_type.as_ref().size_of();
                            *value_size = values_type.as_ref().size_of();

                            Ok(e_static!(StaticType::Vec(VecType(Box::new(e_static!(
                                StaticType::Tuple(TupleType(vec![
                                    e_static!(StaticType::Address(AddrType(Box::new(
                                        keys_type.as_ref().clone()
                                    )))),
                                    e_static!(StaticType::Address(AddrType(Box::new(
                                        values_type.as_ref().clone()
                                    ))))
                                ]))
                            ))))))
                        }
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
            }
            IterFn::MapValues {
                key_size,
                value_size,
            } => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let map = &mut parameters[0];
                let _ = map.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                let mut map_type = map.type_of(&scope_manager, scope_id)?;
                match &map_type {
                    EType::Static(value) => match value {
                        StaticType::Address(AddrType(sub)) => map_type = sub.as_ref().clone(),
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                match &map_type {
                    EType::Static(value) => match value {
                        StaticType::Map(MapType {
                            keys_type,
                            values_type,
                        }) => {
                            *key_size = keys_type.as_ref().size_of();
                            *value_size = values_type.as_ref().size_of();

                            Ok(e_static!(StaticType::Vec(VecType(Box::new(e_static!(
                                StaticType::Address(AddrType(Box::new(
                                    values_type.as_ref().clone()
                                )))
                            ))))))
                        }
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
            }
            IterFn::MapKeys {
                key_size,
                value_size,
            } => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let map = &mut parameters[0];
                let _ = map.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                let mut map_type = map.type_of(&scope_manager, scope_id)?;
                match &map_type {
                    EType::Static(value) => match value {
                        StaticType::Address(AddrType(sub)) => map_type = sub.as_ref().clone(),
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                match &map_type {
                    EType::Static(value) => match value {
                        StaticType::Map(MapType {
                            keys_type,
                            values_type,
                        }) => {
                            *key_size = keys_type.as_ref().size_of();
                            *value_size = values_type.as_ref().size_of();

                            Ok(e_static!(StaticType::Vec(VecType(Box::new(e_static!(
                                StaticType::Address(AddrType(Box::new(keys_type.as_ref().clone())))
                            ))))))
                        }
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
            }
        }
    }
}

impl GenerateCode for IterFn {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            IterFn::MapItems {
                key_size,
                value_size,
            } => instructions.push(Asm::Core(super::CoreAsm::Iter(IterAsm::MapItems {
                key_size: *key_size,
                value_size: *value_size,
            }))),
            IterFn::MapValues {
                key_size,
                value_size,
            } => instructions.push(Asm::Core(super::CoreAsm::Iter(IterAsm::MapValues {
                key_size: *key_size,
                value_size: *value_size,
            }))),
            IterFn::MapKeys {
                key_size,
                value_size,
            } => instructions.push(Asm::Core(super::CoreAsm::Iter(IterAsm::MapKeys {
                key_size: *key_size,
                value_size: *value_size,
            }))),
        }
        Ok(())
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for IterAsm {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::runtime::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::TID>,
    ) -> Result<(), RuntimeError> {
        match self {
            IterAsm::MapItems {
                key_size,
                value_size,
            } => {
                let map_address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let map_layout = map_layout(map_address, *key_size, *value_size, heap)?;
                let items_ptr = map_layout.retrieve_vec_items(heap)?;

                let len = items_ptr.len() as u64;
                let cap = align(len as usize) as u64;
                let alloc_size = cap * (len * 16) + 16;

                let items_data: Vec<u8> = items_ptr
                    .into_iter()
                    .flat_map(|(key_ptr, value_ptr)| {
                        let key: u64 = key_ptr.into(stack);
                        let value: u64 = value_ptr.into(stack);
                        [key.to_le_bytes(), value.to_le_bytes()].concat()
                    })
                    .collect();

                let len_bytes = len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                let address = heap.alloc(alloc_size as usize)?;

                /* Write len */
                let _ = heap.write(address, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(address.add(8), &cap_bytes)?;
                /* Write data */
                let _ = heap.write(address.add(16), &items_data)?;

                let address: u64 = address.into(stack);
                let _ = stack.push_with(&address.to_le_bytes())?;
            }
            IterAsm::MapValues {
                key_size,
                value_size,
            } => {
                let map_address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let map_layout = map_layout(map_address, *key_size, *value_size, heap)?;

                let values_ptr = map_layout.retrieve_vec_values(heap)?;

                let len = values_ptr.len() as u64;
                let cap = align(len as usize) as u64;
                let alloc_size = cap * (len * 16) + 16;

                let values_data: Vec<u8> = values_ptr
                    .into_iter()
                    .flat_map(|value_ptr| {
                        let value: u64 = value_ptr.into(stack);
                        value.to_le_bytes()
                    })
                    .collect();

                let len_bytes = len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                let address = heap.alloc(alloc_size as usize)?;

                /* Write len */
                let _ = heap.write(address, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(address.add(8), &cap_bytes)?;
                /* Write data */
                let _ = heap.write(address.add(16), &values_data)?;

                let address: u64 = address.into(stack);
                let _ = stack.push_with(&address.to_le_bytes())?;
            }
            IterAsm::MapKeys {
                key_size,
                value_size,
            } => {
                let map_address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let map_layout = map_layout(map_address, *key_size, *value_size, heap)?;
                let keys_ptr = map_layout.retrieve_vec_keys(heap)?;

                let len = keys_ptr.len() as u64;
                let cap = align(len as usize) as u64;
                let alloc_size = cap * (len * 16) + 16;

                let keys_data: Vec<u8> = keys_ptr
                    .into_iter()
                    .flat_map(|key_ptr| {
                        let key: u64 = key_ptr.into(stack);
                        key.to_le_bytes()
                    })
                    .collect();

                let len_bytes = len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                let address = heap.alloc(alloc_size as usize)?;

                /* Write len */
                let _ = heap.write(address, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(address.add(8), &cap_bytes)?;
                /* Write data */
                let _ = heap.write(address.add(16), &keys_data)?;

                let address: u64 = address.into(stack);
                let _ = stack.push_with(&address.to_le_bytes())?;
            }
        }
        scheduler.next();
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn valid_values() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = {
    //             let hmap : Map<u64,u64> = map();
    //             insert(&hmap,101,5);
    //             insert(&hmap,102,6);
    //             insert(&hmap,103,7);
    //             insert(&hmap,104,8);
    //             insert(&hmap,105,9);
    //             insert(&hmap,106,10);
    //             insert(&hmap,107,11);
    //             insert(&hmap,108,12);

    //             let val = values(&hmap);
    //             let res = 0u64;
    //             for val_ptr in val {
    //                 res = res + *val_ptr;
    //             }

    //             return res;
    //         };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");

    //     assert_eq!(
    //         result,
    //         v_num!(U64, 5 + 6 + 7 + 8 + 9 + 10 + 11 + 12),
    //         "Result does not match the expected value"
    //     );
    // }

    // #[test]
    // fn valid_keys() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = {
    //             let hmap : Map<u64,u64> = map();
    //             insert(&hmap,101,5);
    //             insert(&hmap,102,6);
    //             insert(&hmap,103,7);
    //             insert(&hmap,104,8);
    //             insert(&hmap,105,9);
    //             insert(&hmap,106,10);
    //             insert(&hmap,107,11);
    //             insert(&hmap,108,12);

    //             let key = keys(&hmap);
    //             let res = 0u64;
    //             for key_ptr in key {
    //                 res = res + (*key_ptr);
    //             }

    //             return res;
    //         };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");

    //     assert_eq!(
    //         result,
    //         v_num!(U64, 101 + 102 + 103 + 104 + 105 + 106 + 107 + 108),
    //         "Result does not match the expected value"
    //     );
    // }

    // #[test]
    // fn valid_items() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = {
    //             let hmap : Map<u64,u64> = map();
    //             insert(&hmap,101,5);
    //             insert(&hmap,102,6);
    //             insert(&hmap,103,7);
    //             insert(&hmap,104,8);
    //             insert(&hmap,105,9);
    //             insert(&hmap,106,10);
    //             insert(&hmap,107,11);
    //             insert(&hmap,108,12);

    //             let items = std::items(&hmap);
    //             let res = 0u64;
    //             for (key_ptr,val_ptr) in items {
    //                 res = res + (*key_ptr) * (*val_ptr);
    //             }

    //             return res;
    //         };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");

    //     assert_eq!(
    //         result,
    //         v_num!(
    //             U64,
    //             101 * 5 + 102 * 6 + 103 * 7 + 104 * 8 + 105 * 9 + 106 * 10 + 107 * 11 + 108 * 12
    //         ),
    //         "Result does not match the expected value"
    //     );
    // }
}
