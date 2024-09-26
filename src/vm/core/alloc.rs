use std::{ops::Add, vec};

use crate::{
    ast::utils::strings::ID,
    err_tuple,
    semantic::{
        scope::{
            scope::ScopeManager,
            static_types::{MapType, SliceType},
        },
        ResolveCore,
    },
    vm::{
        allocator::{heap::Heap, stack::Stack, MemoryAddress},
        asm::operation::{GetNumFrom, PopNum},
        stdio::StdIO,
        vm::CasmMetadata,
    },
};

use num_traits::ToBytes;

use crate::{
    ast::expressions::Expression,
    e_static, p_num,
    semantic::{
        scope::static_types::{
            AddrType, NumberType, PrimitiveType, StaticType, StringType, VecType,
        },
        EType, Info, Metadata, Resolve, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        allocator::align,
        asm::{
            alloc::{Access, Alloc, Free},
            data::Data,
            mem::Mem,
            operation::OpPrimitive,
            Asm, Program,
        },
        vm::{CodeGenerationError, Executable, GenerateCode, RuntimeError},
    },
};

use super::{lexem, CoreCasm, PathFinder, ERROR_SLICE, OK_SLICE};

#[derive(Debug, Clone, PartialEq)]
pub enum AllocFn {
    Len,
    Cap,
    Free,
    Alloc,
    SizeOf { size: usize },
    MemCopy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AllocCasm {
    Len,
    Cap,
    Free,
    Alloc,
    MemCopy,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for AllocCasm {
    fn name(&self, stdio: &mut StdIO, program: &mut Program, engine: &mut G) {
        match self {
            AllocCasm::Len => stdio.push_asm_lib(engine, "len"),
            AllocCasm::Cap => stdio.push_asm_lib(engine, "cap"),
            AllocCasm::Free => stdio.push_asm_lib(engine, "free"),
            AllocCasm::Alloc => stdio.push_asm_lib(engine, "malloc"),
            AllocCasm::MemCopy => stdio.push_asm_lib(engine, "memcpy"),
        }
    }

    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            AllocCasm::Len => crate::vm::vm::CasmWeight::LOW,
            AllocCasm::Cap => crate::vm::vm::CasmWeight::LOW,
            AllocCasm::Free => crate::vm::vm::CasmWeight::MEDIUM,
            AllocCasm::Alloc => crate::vm::vm::CasmWeight::HIGH,
            AllocCasm::MemCopy => crate::vm::vm::CasmWeight::HIGH,
        }
    }
}

impl PathFinder for AllocFn {
    fn find(path: &[String], name: &str) -> Option<Self>
    where
        Self: Sized,
    {
        if (path.len() == 1 && path[0] == lexem::VEC) || path.len() == 0 {
            return match name {
                lexem::LEN => Some(AllocFn::Len),
                lexem::CAP => Some(AllocFn::Cap),
                lexem::FREE => Some(AllocFn::Free),
                lexem::ALLOC => Some(AllocFn::Alloc),
                lexem::MEMCPY => Some(AllocFn::MemCopy),
                lexem::SIZEOF => Some(AllocFn::SizeOf { size: 0 }),
                _ => None,
            };
        }
        None
    }
}

impl ResolveCore for AllocFn {
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        match self {
            AllocFn::Free => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let address = &mut parameters[0];

                let _ = address.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
                let address_type = address.type_of(&scope_manager, scope_id)?;
                match &address_type {
                    EType::Static(StaticType::Address(_)) => {}
                    EType::Static(StaticType::Vec(_)) => {}
                    EType::Static(StaticType::String(_)) => {}
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                Ok(EType::Static(StaticType::Error))
            }
            AllocFn::Alloc => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let size = &mut parameters[0];

                let _ =
                    size.resolve::<G>(scope_manager, scope_id, &Some(p_num!(U64)), &mut None)?;

                let p_num!(U64) = size.type_of(&scope_manager, scope_id)? else {
                    return Err(SemanticError::IncorrectArguments);
                };

                let inner = match context {
                    Some(context) => context.clone(),
                    None => EType::Static(StaticType::Any),
                };

                Ok(err_tuple!(EType::Static(StaticType::Address(AddrType(
                    inner.into()
                )))))
            }
            AllocFn::Len => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let address = &mut parameters[0];

                let _ = address.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
                let address_type = address.type_of(&scope_manager, scope_id)?;
                match &address_type {
                    EType::Static(value) => match value {
                        StaticType::String(_) => {}
                        StaticType::Vec(_) => {}
                        StaticType::Map(_) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(p_num!(U64))
            }
            AllocFn::Cap => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let address = &mut parameters[0];

                let _ = address.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
                let address_type = address.type_of(&scope_manager, scope_id)?;

                match &address_type {
                    EType::Static(value) => match value {
                        StaticType::String(_) => {}
                        StaticType::Vec(_) => {}
                        StaticType::Map(_) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(p_num!(U64))
            }
            AllocFn::SizeOf { size } => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let param = &mut parameters[0];

                let _ = param.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
                let param_type = param.type_of(&scope_manager, scope_id)?;

                *size = param_type.size_of();

                Ok(p_num!(U64))
            }
            AllocFn::MemCopy => {
                if parameters.len() != 3 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, rest) = parameters.split_at_mut(1);
                let (second_part, third_part) = rest.split_at_mut(1);

                // Get mutable references to the elements
                let dest = &mut first_part[0];
                let src = &mut second_part[0];
                let size = &mut third_part[0];

                let _ = dest.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
                let _ = src.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
                let dest_type = dest.type_of(&scope_manager, scope_id)?;
                let src_type = src.type_of(&scope_manager, scope_id)?;

                match &dest_type {
                    EType::Static(value) => match value {
                        StaticType::Address(_) => {}
                        StaticType::String(_) => {}
                        StaticType::StrSlice(_) => {}
                        StaticType::Slice(_) => {}
                        StaticType::Vec(_) => {}
                        StaticType::Map(_) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                match &src_type {
                    EType::Static(value) => match value {
                        StaticType::Address(_) => {}
                        StaticType::String(_) => {}
                        StaticType::StrSlice(_) => {}
                        StaticType::Slice(_) => {}
                        StaticType::Vec(_) => {}
                        StaticType::Map(_) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                let _ =
                    size.resolve::<G>(scope_manager, scope_id, &Some(p_num!(U64)), &mut None)?;
                let p_num!(U64) = size.type_of(&scope_manager, scope_id)? else {
                    return Err(SemanticError::IncorrectArguments);
                };

                Ok(EType::Static(StaticType::Error))
            }
        }
    }
}

impl GenerateCode for AllocFn {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            AllocFn::Free => instructions.push(Asm::Core(super::CoreCasm::Alloc(AllocCasm::Free))),
            AllocFn::Alloc => {
                instructions.push(Asm::Core(super::CoreCasm::Alloc(AllocCasm::Alloc)))
            }
            AllocFn::Len => instructions.push(Asm::Core(super::CoreCasm::Alloc(AllocCasm::Len))),
            AllocFn::Cap => instructions.push(Asm::Core(super::CoreCasm::Alloc(AllocCasm::Cap))),
            AllocFn::SizeOf { size } => {
                instructions.push(Asm::Pop(*size));
                instructions.push(Asm::Data(Data::Serialized {
                    data: Box::new(size.to_le_bytes()),
                }));
            }
            AllocFn::MemCopy => {
                instructions.push(Asm::Core(super::CoreCasm::Alloc(AllocCasm::MemCopy)))
            }
        }
        Ok(())
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for AllocCasm {
    fn execute(
        &self,
        program: &mut Program,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self {
            AllocCasm::Len => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let len = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)?;

                let _ = stack.push_with(&len.to_le_bytes())?;
            }
            AllocCasm::Cap => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let cap = OpPrimitive::get_num_from::<u64>(address, stack, heap)?;

                let _ = stack.push_with(&cap.to_le_bytes())?;
            }
            AllocCasm::Free => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                if heap.free(address).is_ok() {
                    stack.push_with(&OK_SLICE)?;
                } else {
                    stack.push_with(&ERROR_SLICE)?;
                }
            }
            AllocCasm::Alloc => {
                let size = OpPrimitive::pop_num::<u64>(stack)? as usize;
                if let Ok(address) = heap.alloc(size) {
                    let address: u64 = address.into(stack);
                    stack.push_with(&address.to_le_bytes())?;
                    stack.push_with(&OK_SLICE)?;
                } else {
                    stack.push_with(&(0u64).to_le_bytes())?;
                    stack.push_with(&ERROR_SLICE)?;
                }
            }
            AllocCasm::MemCopy => {
                let size = OpPrimitive::pop_num::<u64>(stack)? as usize;
                let source: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let destination: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let source_data = match source {
                    MemoryAddress::Heap { .. } => heap.read(source, size).ok(),
                    MemoryAddress::Stack { .. } => {
                        stack.read(source, size).map(|v| v.to_vec()).ok()
                    }
                    MemoryAddress::Global { .. } => stack
                        .read_global(destination, size)
                        .map(|v| v.to_vec())
                        .ok(),
                    MemoryAddress::Frame { .. } => {
                        stack.read_in_frame(source, size).map(|v| v.to_vec()).ok()
                    }
                };

                if let Some(data) = source_data {
                    let res = match destination {
                        MemoryAddress::Heap { .. } => heap.write(destination, &data).ok(),
                        MemoryAddress::Stack { .. } => stack.write(destination, &data).ok(),
                        MemoryAddress::Frame { .. } => {
                            stack.write_in_frame(destination, &data).ok()
                        }
                        MemoryAddress::Global { .. } => stack.write_global(destination, &data).ok(),
                    };
                    if res.is_some() {
                        stack.push_with(&OK_SLICE)?;
                    } else {
                        stack.push_with(&ERROR_SLICE)?;
                    }
                } else {
                    stack.push_with(&ERROR_SLICE)?;
                }
            }
        }

        program.incr();
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         ast::{
//             expressions::{
//                 data::{Data, Number, Primitive},
//                 Atomic,
//             },
//             statements::Statement,
//             TryParse,
//         },
//         clear_stack, compile_statement, compile_statement_for_string,
//         semantic::scope::scope::ScopeManager,
//         v_num,
//         vm::vm::{DeserializeFrom, Runtime},
//     };

//     use super::*;

//     #[test]
//     fn valid_string() {
//         let mut statement = Statement::parse(
//             r##"
//         let x = string("Hello World");
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;

//         let result = compile_statement_for_string!(statement);

//         assert_eq!(result, "Hello World");
//     }

//     #[test]
//     fn valid_vec() {
//         let mut statement = Statement::parse(
//             r##"
//         let x:Vec<u64> = vec(8);
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let heap_address = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         let data_length = heap
//             .read(heap_address, 8)
//             .expect("length should be readable");
//         let length = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data_length[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         assert_eq!(length, 8);
//     }

//     #[test]
//     fn valid_vec_modify() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let arr : Vec<u64> = vec(8);

//                 arr[2] = 420;

//                 return arr[2];
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");
//         assert_eq!(result, v_num!(U64, 420));
//     }
//     #[test]
//     fn valid_append() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
//                 append(&x,9);
//                 return x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let heap_address = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         let data_length = heap
//             .read(heap_address, 8)
//             .expect("length should be readable");
//         let length = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data_length[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         assert_eq!(length, 9);

//         let data = heap
//             .read(heap_address as usize, 8 * 9 + 16)
//             .expect("Heap Read should have succeeded");
//         let data: Vec<u64> = data
//             .chunks(8)
//             .map(|chunk| {
//                 u64::from_le_bytes(
//                     TryInto::<[u8; 8]>::try_into(&chunk[0..8])
//                         .expect("heap address should be deserializable"),
//                 )
//             })
//             .collect();
//         assert_eq!(data, vec![9, 24, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
//     }

//     #[test]
//     fn valid_append_no_realloc() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x:Vec<u64> = vec(8,16);
//                 append(&x,9);
//                 return x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let heap_address = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         let data_length = heap
//             .read(heap_address, 8)
//             .expect("length should be readable");
//         let length = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data_length[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         assert_eq!(length, 9);

//         let data = heap
//             .read(heap_address as usize, 8 * 9 + 16)
//             .expect("Heap Read should have succeeded");
//         let data: Vec<u64> = data
//             .chunks(8)
//             .map(|chunk| {
//                 u64::from_le_bytes(
//                     TryInto::<[u8; 8]>::try_into(&chunk[0..8])
//                         .expect("heap address should be deserializable"),
//                 )
//             })
//             .collect();
//         assert_eq!(data, vec![9, 16, 0, 0, 0, 0, 0, 0, 0, 0, 9]);
//     }

//     #[test]
//     fn valid_append_str_slice() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x = string("Hello ");
//                 append(&x,"World");
//                 return x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;

//         let result = compile_statement_for_string!(statement);

//         assert_eq!(result, "Hello World");
//     }

//     #[test]
//     fn valid_append_str_char() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x = string("Hello Worl");
//                 append(&x,'d');
//                 return x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;

//         let result = compile_statement_for_string!(statement);

//         assert_eq!(result, "Hello World");
//     }

//     #[test]
//     fn valid_append_str_char_complex() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x = string("Hello Worl");
//                 append(&x,'仗');
//                 return x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;

//         let result = compile_statement_for_string!(statement);

//         assert_eq!(result, "Hello Worl仗");
//     }

//     #[test]
//     fn valid_append_str_str() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x = string("Hello ");
//                 let y = string("World");
//                 append(&x,y);
//                 return x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;

//         let result = compile_statement_for_string!(statement);

//         assert_eq!(result, "Hello World");
//     }

//     #[test]
//     fn valid_len_string() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x = string("Hello World");
//                 return len(x);
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");
//         assert_eq!(result, v_num!(U64, 11));
//     }

//     #[test]
//     fn valid_cap_string() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x = string("Hello World");
//                 return cap(x);
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");
//         assert_eq!(result, v_num!(U64, 16));
//     }

//     #[test]
//     fn valid_len_vec() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x : Vec<u64> = vec(11,16);
//                 return len(x);
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;

//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");
//         assert_eq!(result, v_num!(U64, 11));
//     }

//     #[test]
//     fn valid_cap_vec() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x : Vec<u64> = vec(11,16);
//                 return cap(x);
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;

//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");
//         assert_eq!(result, v_num!(U64, 16));
//     }
//     #[test]
//     fn valid_free() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x = string("Hello ");
//                 free(&x);
//                 return x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         assert_eq!(heap.allocated_size(), 0);
//     }

//     #[test]
//     fn valid_alloc() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x = alloc(8) as &u64;
//                 *x = 420;
//                 return x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         assert_eq!(heap.allocated_size(), 16);

//         let data = clear_stack!(memory);
//         let heap_address = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;

//         let data = heap
//             .read(heap_address, 8)
//             .expect("length should be readable");
//         let data = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         assert_eq!(data, 420);
//     }

//     #[test]
//     fn valid_delete_vec() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
//                 let (old,err) = delete(&x,7);
//                 return x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let heap_address = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         let data_length = heap
//             .read(heap_address, 8)
//             .expect("length should be readable");
//         let length = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data_length[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         assert_eq!(length, 7);

//         let data = heap
//             .read(heap_address as usize, 8 * length + 16)
//             .expect("Heap Read should have succeeded");
//         let data: Vec<u64> = data
//             .chunks(8)
//             .map(|chunk| {
//                 u64::from_le_bytes(
//                     TryInto::<[u8; 8]>::try_into(&chunk[0..8])
//                         .expect("heap address should be deserializable"),
//                 )
//             })
//             .collect();
//         assert_eq!(data, vec![7, 8, 1, 2, 3, 4, 5, 6, 7]);
//     }

//     #[test]
//     fn valid_delete_vec_inner() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
//                 let (old,err) = delete(&x,2);
//                 return x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let heap_address = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         let data_length = heap
//             .read(heap_address, 8)
//             .expect("length should be readable");
//         let length = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data_length[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         assert_eq!(length, 7);

//         let data = heap
//             .read(heap_address as usize, 8 * length + 16)
//             .expect("Heap Read should have succeeded");
//         let data: Vec<u64> = data
//             .chunks(8)
//             .map(|chunk| {
//                 u64::from_le_bytes(
//                     TryInto::<[u8; 8]>::try_into(&chunk[0..8])
//                         .expect("heap address should be deserializable"),
//                 )
//             })
//             .collect();
//         assert_eq!(data, vec![7, 8, 1, 2, 4, 5, 6, 7, 8]);
//     }

//     #[test]
//     fn robustness_delete_vec() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
//                 let (old,err) = delete(&x,15);
//                 return err;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let _ = compile_statement!(statement);
//         let data = compile_statement!(statement);
//         let result = data.first().unwrap();

//         assert_eq!(*result, 1u8, "Result does not match the expected value");
//     }

//     #[test]
//     fn valid_size_of_type() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 struct Point {
//                     x : u64,
//                     y : u64,
//                 }
//                 return size_of(Point);
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");
//         assert_eq!(result, v_num!(U64, 16));
//     }

//     #[test]
//     fn valid_size_of_expr() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = size_of(420u64);
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");
//         assert_eq!(result, v_num!(U64, 8));
//     }

//     #[test]
//     fn valid_memcpy_heap() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
//                 let y:Vec<u64> = vec(8);
//                 memcpy(y,x,8*8 + 16);
//                 return y;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let heap_address = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         let data_length = heap
//             .read(heap_address, 8)
//             .expect("length should be readable");
//         let length = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data_length[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         assert_eq!(length, 8);

//         let data = heap
//             .read(heap_address as usize, 8 * length + 16)
//             .expect("Heap Read should have succeeded");
//         let data: Vec<u64> = data
//             .chunks(8)
//             .map(|chunk| {
//                 u64::from_le_bytes(
//                     TryInto::<[u8; 8]>::try_into(&chunk[0..8])
//                         .expect("heap address should be deserializable"),
//                 )
//             })
//             .collect();
//         assert_eq!(data, vec![8, 8, 1, 2, 3, 4, 5, 6, 7, 8]);
//     }

//     #[test]
//     fn valid_memcpy_stack() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x:[8]u64 = [1,2,3,4,5,6,7,8];
//                 let y:[8]u64 = [0,0,0,0,0,0,0,0];
//                 memcpy(&y,&x,8*8);
//                 return y;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);

//         let data: Vec<u64> = data
//             .chunks(8)
//             .map(|chunk| {
//                 u64::from_le_bytes(
//                     TryInto::<[u8; 8]>::try_into(&chunk[0..8])
//                         .expect("heap address should be deserializable"),
//                 )
//             })
//             .collect();
//         assert_eq!(data, vec![1, 2, 3, 4, 5, 6, 7, 8]);
//     }

//     #[test]
//     fn valid_clear_vec() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
//                 clear(&x);
//                 return x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let heap_address = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         let data_length = heap
//             .read(heap_address, 8)
//             .expect("length should be readable");
//         let length = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data_length[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         assert_eq!(length, 0);

//         let data = heap
//             .read(heap_address as usize, 8 * 8 + 16)
//             .expect("Heap Read should have succeeded");
//         let data: Vec<u64> = data
//             .chunks(8)
//             .map(|chunk| {
//                 u64::from_le_bytes(
//                     TryInto::<[u8; 8]>::try_into(&chunk[0..8])
//                         .expect("heap address should be deserializable"),
//                 )
//             })
//             .collect();
//         assert_eq!(data, vec![0, 8, 0, 0, 0, 0, 0, 0, 0, 0]);
//     }

//     #[test]
//     fn valid_clear_str() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x = string("Hello ");
//                 clear(&x);
//                 return x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let heap_address = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;

//         let data_length = heap
//             .read(heap_address, 8)
//             .expect("length should be readable");
//         let length = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data_length[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;
//         let data = heap
//             .read(heap_address, 5 + 16)
//             .expect("length should be readable");

//         let result = <StringType as DeserializeFrom>::deserialize_from(&StringType(), &data)
//             .expect("Deserialization should have succeeded");

//         assert_eq!(result.value, "");
//     }

//     #[test]
//     fn valid_alloc_cast() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 struct Point {
//                     x : i64,
//                     y: i64
//                 }

//                 let point_any : &Any = alloc(size_of(Point)) as &Any;

//                 let copy = Point {
//                     x : 420,
//                     y : 69,
//                 };
//                 memcpy(point_any,&copy,size_of(Point));

//                 let point = *point_any as Point;

//                 return point.x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");
//         assert_eq!(result, v_num!(U64, 420));
//     }

//     #[test]
//     fn valid_alloc_modify() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 struct Point {
//                     x : i64,
//                     y: i64
//                 }

//                 let point : &Point = alloc(size_of(Point)) as &Point;

//                 point.x = 420;

//                 return point.x;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");
//         assert_eq!(result, v_num!(U64, 420));
//     }

//     #[test]
//     fn valid_extend_vec_from_slice() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let slice = [5,6,7,8];
//                 let vector:Vec<i64> = vec(8);
//                 extend(&vector,slice);
//                 return vector[8];
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::I64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");
//         assert_eq!(result, v_num!(I64, 5));
//     }

//     #[test]
//     fn valid_extend_vec_from_vec() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let vec1 = vec[5,6,7,8];
//                 let vector:Vec<i64> = vec(8);
//                 extend(&vector,vec1);
//                 return vector[8];
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;

//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::I64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");
//         assert_eq!(result, v_num!(I64, 5));
//     }

//     #[test]
//     fn valid_extend_string_from_slice() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let vec1 = [string("lo"),string(" Wor"),string("ld")];
//                 let hello = string("Hel");
//                 extend(&hello,vec1);
//                 return hello;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let heap_address = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;

//         let data_length = heap
//             .read(heap_address, 8)
//             .expect("length should be readable");
//         let length = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data_length[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;

//         let data = heap
//             .read(heap_address, length + 16)
//             .expect("length should be readable");

//         let result = <StringType as DeserializeFrom>::deserialize_from(&StringType(), &data)
//             .expect("Deserialization should have succeeded");

//         assert_eq!(result.value, "Hello World");
//     }

//     #[test]
//     fn valid_extend_string_from_vec() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let vec1 = vec[string("lo"),string(" Wor"),string("ld")];
//                 let hello = string("Hel");
//                 extend(&hello,vec1);
//                 return hello;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = statement
//             .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         statement
//             .gencode(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0, "No instructions generated");
//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         let mut engine = crate::vm::vm::NoopGameEngine {};

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         let heap_address = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;

//         let data_length = heap
//             .read(heap_address, 8)
//             .expect("length should be readable");
//         let length = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data_length[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;

//         let data = heap
//             .read(heap_address, length + 16)
//             .expect("length should be readable");

//         let result = <StringType as DeserializeFrom>::deserialize_from(&StringType(), &data)
//             .expect("Deserialization should have succeeded");

//         assert_eq!(result.value, "Hello World");
//     }

//     #[test]
//     fn valid_map_init_no_cap() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map();
//                 return true;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let _ = compile_statement!(statement);
//     }

//     #[test]
//     fn valid_map_init_with_cap() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map(8);
//                 return true;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let _ = compile_statement!(statement);
//     }

//     #[test]
//     fn valid_map_len_empty() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map(8);
//                 return len(hmap);
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result,
//             v_num!(U64, 0),
//             "Result does not match the expected value"
//         );
//     }

//     #[test]
//     fn valid_map_len() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map();
//                 insert(&hmap,101,5);
//                 insert(&hmap,102,6);
//                 insert(&hmap,103,7);
//                 insert(&hmap,104,8);
//                 insert(&hmap,105,9);
//                 insert(&hmap,106,10);
//                 insert(&hmap,107,11);
//                 insert(&hmap,108,12);
//                 return len(hmap);
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result,
//             v_num!(U64, 8),
//             "Result does not match the expected value"
//         );
//     }

//     #[test]
//     fn valid_map_len_with_upsert() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map();
//                 insert(&hmap,101,5);
//                 insert(&hmap,102,6);
//                 insert(&hmap,103,7);
//                 insert(&hmap,103,8);
//                 insert(&hmap,105,9);
//                 insert(&hmap,103,10);
//                 insert(&hmap,107,11);
//                 insert(&hmap,103,12);
//                 return len(hmap);
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result,
//             v_num!(U64, 5),
//             "Result does not match the expected value"
//         );
//     }
//     #[test]
//     fn valid_map_cap() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map(8);
//                 return cap(hmap);
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result,
//             v_num!(U64, 8),
//             "Result does not match the expected value"
//         );
//     }

//     #[test]
//     fn valid_map_cap_over_min() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map(60);
//                 return cap(hmap);
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result,
//             v_num!(U64, 64),
//             "Result does not match the expected value"
//         );
//     }

//     #[test]
//     fn valid_map_access() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map(64);
//                 insert(&hmap,420,69);
//                 let (value,err) = get(&hmap,420);
//                 assert(err);
//                 return value;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let _ = compile_statement!(statement);
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result,
//             v_num!(U64, 69),
//             "Result does not match the expected value"
//         );
//     }

//     #[test]
//     fn valid_map_access_complex() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map(64);
//                 insert(&hmap,420,69);
//                 return get(&hmap,420).0;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let _ = compile_statement!(statement);
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result,
//             v_num!(U64, 69),
//             "Result does not match the expected value"
//         );
//     }
//     #[test]
//     fn robustness_map_access() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map(64);
//                 let (value,err) = get(&hmap,420);
//                 return err;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let _ = compile_statement!(statement);
//         let data = compile_statement!(statement);
//         let result = data.first().unwrap();

//         assert_eq!(*result, 1u8, "Result does not match the expected value");
//     }

//     #[test]
//     fn valid_map_insert_resize() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map();
//                 insert(&hmap,101,5);
//                 insert(&hmap,102,6);
//                 insert(&hmap,103,7);
//                 insert(&hmap,104,8);
//                 insert(&hmap,105,9);
//                 insert(&hmap,106,10);
//                 insert(&hmap,107,11);
//                 insert(&hmap,108,12);
//                 insert(&hmap,109,13);
//                 insert(&hmap,110,14);
//                 let (value,err) = get(&hmap,103);
//                 assert(err);
//                 return value;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let _ = compile_statement!(statement);
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");
//         assert_eq!(
//             result,
//             v_num!(U64, 7),
//             "Result does not match the expected value"
//         );
//     }

//     #[test]
//     fn valid_map_upsert_resize() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map();
//                 insert(&hmap,101,5);
//                 insert(&hmap,102,6);
//                 insert(&hmap,103,7);
//                 insert(&hmap,104,8);
//                 insert(&hmap,105,9);
//                 insert(&hmap,106,10);
//                 insert(&hmap,107,11);
//                 insert(&hmap,108,12);
//                 insert(&hmap,109,13);
//                 insert(&hmap,110,14);
//                 insert(&hmap,103,420);
//                 let (value,err) = get(&hmap,103);
//                 assert(err);
//                 return value;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let _ = compile_statement!(statement);
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result,
//             v_num!(U64, 420),
//             "Result does not match the expected value"
//         );
//     }

//     #[test]
//     fn valid_map_insert() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map(64);
//                 insert(&hmap,420,69);
//                 return len(hmap);
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result,
//             v_num!(U64, 1),
//             "Result does not match the expected value"
//         );
//     }

//     #[test]
//     fn valid_map_delete() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map(64);
//                 insert(&hmap,420,69);
//                 insert(&hmap,120,75);
//                 let (value,err) = delete(&hmap,420);
//                 assert(err);
//                 return (value,len(hmap));
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let data = compile_statement!(statement);
//         let result =
//             <crate::semantic::scope::static_types::TupleType as DeserializeFrom>::deserialize_from(
//                 &crate::semantic::scope::static_types::TupleType(vec![p_num!(U64), p_num!(U64)]),
//                 &data,
//             )
//             .expect("Deserialization should have succeeded");
//         let value = &result.value[0];
//         let value = match value {
//             Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(v)))) => match v {
//                 Number::U64(n) => n,
//                 _ => unreachable!("Should be a u64"),
//             },
//             _ => unreachable!("Should be a u64"),
//         };
//         let len = &result.value[1];
//         let len = match len {
//             Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(v)))) => match v {
//                 Number::U64(n) => n,
//                 _ => unreachable!("Should be a u64"),
//             },
//             _ => unreachable!("Should be a u64"),
//         };
//         assert_eq!(*value, 69);
//         assert_eq!(*len, 1);
//     }

//     #[test]
//     fn valid_map_clear() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map();
//                 insert(&hmap,101,5);
//                 insert(&hmap,102,6);
//                 insert(&hmap,103,7);
//                 insert(&hmap,104,8);
//                 insert(&hmap,105,9);
//                 insert(&hmap,106,10);
//                 insert(&hmap,107,11);
//                 insert(&hmap,108,12);
//                 clear(&hmap);
//                 return len(hmap);
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result,
//             v_num!(U64, 0),
//             "Result does not match the expected value"
//         );
//     }
//     #[test]
//     fn valid_map_delete_cant_read_after() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<u64,u64> = map(64);
//                 insert(&hmap,420,69);
//                 insert(&hmap,120,75);
//                 let (value,err) = delete(&hmap,420);
//                 assert(err);
//                 let (value,err) = get(&hmap,420);
//                 return err;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let _ = compile_statement!(statement);
//         let data = compile_statement!(statement);
//         let result = data.first().unwrap();

//         assert_eq!(*result, 1u8, "Result does not match the expected value");
//     }

//     #[test]
//     fn valid_map_insert_key_str() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<str<10>,u64> = map(64);
//                 insert(&hmap,"test",69);
//                 insert(&hmap,"test1",80);
//                 insert(&hmap,"test11",46);
//                 insert(&hmap,"test111",16);
//                 let (value,err) = get(&hmap,"test1");
//                 assert(err);
//                 return value;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result,
//             v_num!(U64, 80),
//             "Result does not match the expected value"
//         );
//     }

//     #[test]
//     fn valid_map_insert_key_string() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let hmap : Map<String,u64> = map(64);
//                 insert(&hmap,string("test"),69);
//                 insert(&hmap,string("test1"),80);
//                 insert(&hmap,string("test11"),46);
//                 insert(&hmap,string("test111"),16);
//                 insert(&hmap,f"test11{52}",28);
//                 let (value,err) = get(&hmap,string("test1"));
//                 assert(err);
//                 return value;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result,
//             v_num!(U64, 80),
//             "Result does not match the expected value"
//         );
//     }
//     #[test]
//     fn valid_map_upsert_key_string() {
//         let mut statement = Statement::parse(
//             r##"
//             let res = {
//                 let x = 1;
//                 let hmap : Map<String,u64> = map(64);

//                 insert(&hmap,string("test"),69);
//                 insert(&hmap,string("test1"),80);
//                 insert(&hmap,string("test11"),46);
//                 insert(&hmap,string("test111"),16);

//                 insert(&hmap,f"test{x}",28);

//                 let (value,err) = get(&hmap,string("test1"));
//                 assert(err);
//                 return value;
//             };
//         "##
//             .into(),
//         )
//         .expect("Parsing should have succeeded")
//         .1;
//         let data = compile_statement!(statement);
//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::U64),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result,
//             v_num!(U64, 28),
//             "Result does not match the expected value"
//         );
//     }
// }
