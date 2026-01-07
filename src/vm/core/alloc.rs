use std::{ops::Add, vec};

use crate::{
    err_tuple,
    semantic::ResolveCore,
    vm::{
        allocator::MemoryAddress,
        asm::operation::{GetNumFrom, PopNum},
        runtime::RuntimeError,
        scheduler::Executable,
        stdio::StdIO,
        GenerateCode,
    },
};

use num_traits::ToBytes;

use crate::{
    ast::expressions::Expression,
    e_static, p_num,
    semantic::{
        scope::static_types::{AddrType, StaticType},
        EType, Resolve, SemanticError, SizeOf, TypeOf,
    },
    vm::asm::{data::Data, operation::OpPrimitive, Asm},
};

use super::{lexem, PathFinder, ERROR_SLICE, OK_SLICE};

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
pub enum AllocAsm {
    Len,
    Cap,
    Free,
    Alloc,
    MemCopy,
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for AllocAsm {
    fn name(
        &self,
        stdio: &mut StdIO,
        program: &crate::vm::program::Program<E>,
        engine: &mut E,
        pid: E::PID,
    ) {
        match self {
            AllocAsm::Len => stdio.push_asm_lib(engine, pid, "len"),
            AllocAsm::Cap => stdio.push_asm_lib(engine, pid, "cap"),
            AllocAsm::Free => stdio.push_asm_lib(engine, pid, "free"),
            AllocAsm::Alloc => stdio.push_asm_lib(engine, pid, "malloc"),
            AllocAsm::MemCopy => stdio.push_asm_lib(engine, pid, "memcpy"),
        }
    }
}

impl crate::vm::AsmWeight for AllocAsm {
    fn weight(&self) -> crate::vm::Weight {
        match self {
            AllocAsm::Len => crate::vm::Weight::LOW,
            AllocAsm::Cap => crate::vm::Weight::LOW,
            AllocAsm::Free => crate::vm::Weight::MEDIUM,
            AllocAsm::Alloc => crate::vm::Weight::HIGH,
            AllocAsm::MemCopy => crate::vm::Weight::HIGH,
        }
    }
}

impl PathFinder for AllocFn {
    fn find(path: &[String], name: &str) -> Option<Self>
    where
        Self: Sized,
    {
        if (path.len() == 1 && path[0] == lexem::MEM) || path.len() == 0 {
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
    fn resolve<E: crate::vm::external::Engine>(
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

                let _ = address.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                let address_type = address.type_of(&scope_manager, scope_id)?;
                match &address_type {
                    EType::Static(StaticType::Address(_)) => {}
                    EType::Static(StaticType::Vec(_)) => {}
                    EType::Static(StaticType::String(_)) => {}
                    EType::Static(StaticType::Map(_)) => {}
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
                    size.resolve::<E>(scope_manager, scope_id, &Some(p_num!(U64)), &mut None)?;

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

                let _ = address.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
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

                let _ = address.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
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

                let _ = param.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
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

                let _ = dest.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                let _ = src.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
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
                    size.resolve::<E>(scope_manager, scope_id, &Some(p_num!(U64)), &mut None)?;
                let p_num!(U64) = size.type_of(&scope_manager, scope_id)? else {
                    return Err(SemanticError::IncorrectArguments);
                };

                Ok(EType::Static(StaticType::Error))
            }
        }
    }
}

impl GenerateCode for AllocFn {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            AllocFn::Free => instructions.push(Asm::Core(super::CoreAsm::Alloc(AllocAsm::Free))),
            AllocFn::Alloc => instructions.push(Asm::Core(super::CoreAsm::Alloc(AllocAsm::Alloc))),
            AllocFn::Len => instructions.push(Asm::Core(super::CoreAsm::Alloc(AllocAsm::Len))),
            AllocFn::Cap => instructions.push(Asm::Core(super::CoreAsm::Alloc(AllocAsm::Cap))),
            AllocFn::SizeOf { size } => {
                instructions.push(Asm::Pop(*size));
                instructions.push(Asm::Data(Data::Serialized {
                    data: Box::new(size.to_le_bytes()),
                }));
            }
            AllocFn::MemCopy => {
                instructions.push(Asm::Core(super::CoreAsm::Alloc(AllocAsm::MemCopy)))
            }
        }
        Ok(())
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for AllocAsm {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match self {
            AllocAsm::Len => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let len = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)?;
                let _ = stack.push_with(&len.to_le_bytes())?;
            }
            AllocAsm::Cap => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let cap = OpPrimitive::get_num_from::<u64>(address, stack, heap)?;

                let _ = stack.push_with(&cap.to_le_bytes())?;
            }
            AllocAsm::Free => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                if heap.free(address).is_ok() {
                    stack.push_with(&OK_SLICE)?;
                } else {
                    stack.push_with(&ERROR_SLICE)?;
                }
            }
            AllocAsm::Alloc => {
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
            AllocAsm::MemCopy => {
                let size = OpPrimitive::pop_num::<u64>(stack)? as usize;
                let source: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let destination: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let source_data = match source {
                    MemoryAddress::Heap { .. } => heap.read(source, size).ok(),
                    MemoryAddress::Stack { .. } => {
                        stack.read(source, size).map(|v| v.to_vec()).ok()
                    }
                    MemoryAddress::Global { .. } => {
                        stack.read_global(source, size).map(|v| v.to_vec()).ok()
                    }
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

        scheduler.next();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        test_extract_variable, test_extract_variable_with, test_statements,
        vm::{
            allocator::MemoryAddress,
            asm::operation::{GetNumFrom, OpPrimitive},
        },
    };

    #[test]
    fn valid_alloc() {
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            test_extract_variable_with(
                "res",
                |address, stack, heap| {
                    let address: MemoryAddress =
                        OpPrimitive::get_num_from::<u64>(address, stack, heap)
                            .expect("Deserialization should have succeeded")
                            .try_into()
                            .unwrap();

                    let res = OpPrimitive::get_num_from::<u64>(address, stack, heap)
                        .expect("Deserialization should have succeeded");

                    assert_eq!(res, 420);
                },
                scope_manager,
                stack,
                heap,
            );
            test_extract_variable_with(
                "point",
                |address, stack, heap| {
                    let x = OpPrimitive::get_num_from::<i64>(address, stack, heap)
                        .expect("Deserialization should have succeeded");

                    let y = OpPrimitive::get_num_from::<i64>(address.add(8), stack, heap)
                        .expect("Deserialization should have succeeded");
                    assert_eq!(x, 420);
                    assert_eq!(y, 69);
                },
                scope_manager,
                stack,
                heap,
            );
            test_extract_variable_with(
                "point2",
                |address, stack, heap| {
                    let address: MemoryAddress =
                        OpPrimitive::get_num_from::<u64>(address, stack, heap)
                            .expect("Deserialization should have succeeded")
                            .try_into()
                            .unwrap();
                    let x = OpPrimitive::get_num_from::<i64>(address, stack, heap)
                        .expect("Deserialization should have succeeded");

                    let y = OpPrimitive::get_num_from::<i64>(address.add(8), stack, heap)
                        .expect("Deserialization should have succeeded");
                    assert_eq!(x, 50);
                    assert_eq!(y, 100);
                },
                scope_manager,
                stack,
                heap,
            );
            true
        }

        test_statements(
            r##"
        let (res,err) = alloc(8) as (&u64,Error);
        *res = 420;

        struct Point {
            x : i64,
            y : i64,
        }

        let (point_any,err) = alloc(size_of(Point)) as (&Any, Error);

        let copy = Point {
            x : 420,
            y : 69,
        };
        memcpy(point_any, &copy, size_of(Point));

        let point = *point_any as Point;

        let (point2,err) = alloc(size_of(Point)) as (&Point,Error);

        (*point2).x = 50;
        (*point2).y = 100;

        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_free() {
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            assert_eq!(heap.allocated_size(), 0);
            true
        }

        test_statements(
            r##"
        let (res,err) = alloc(8) as (&u64,Error);
        free(res);

        struct Point {
            x : i64,
            y : i64,
        }

        let (point_any,err) = alloc(size_of(Point)) as (&Any, Error);
        free(point_any);

        let arr : Vec[i64] = vec(11,18);
        free(arr);

        let word = string("Hello World");
        free(word);

        let hmap : Map[i64]i64 = map(15);
        free_map(hmap);
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_len_cap() {
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<u64>("len1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            let res = test_extract_variable::<u64>("cap1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 10);
            let res = test_extract_variable::<u64>("len2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 11);
            let res = test_extract_variable::<u64>("cap2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 18);

            let res = test_extract_variable::<u64>("len3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 0);
            let res = test_extract_variable::<u64>("cap3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 0);
            let res = test_extract_variable::<u64>("len4", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 11);
            let res = test_extract_variable::<u64>("cap4", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 22);

            let res = test_extract_variable::<u64>("len5", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 0);
            let res = test_extract_variable::<u64>("cap5", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 0);
            let res = test_extract_variable::<u64>("len6", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 0);
            let res = test_extract_variable::<u64>("cap6", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1);
            true
        }

        test_statements(
            r##"
        let arr : Vec[i64] = vec(5);
        let arr2 : Vec[i64] = vec(11,18);
        let len1 = len(arr);
        let cap1 = cap(arr);
        let len2 = len(arr2);
        let cap2 = cap(arr2);
        
        let string1 = string("");
        let string2 = string("Hello World");
        let len3 = len(string1);
        let cap3 = cap(string1);
        let len4 = len(string2);
        let cap4 = cap(string2);
        
        let map1 : Map[i64]i64 = map();
        let map2 : Map[i64]i64 = map(15);
        let len5 = len(map1);
        let cap5 = cap(map1);
        let len6 = len(map2);
        let cap6 = cap(map2);
        "##,
            &mut engine,
            assert_fn,
        );
    }
}
