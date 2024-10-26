use crate::e_static;
use crate::semantic::scope::static_types::{StaticType, StringType, POINTER_SIZE};
use crate::semantic::{ResolveCore, TypeOf};

use crate::vm::allocator::{align, MemoryAddress};
use crate::vm::asm::operation::{OpPrimitive, PopNum};
use crate::vm::asm::Asm;
use crate::vm::core::lexem;

use crate::vm::core::CoreAsm;
use crate::vm::runtime::RuntimeError;
use crate::vm::scheduler::Executable;
use crate::vm::stdio::StdIO;
use crate::vm::GenerateCode;
use crate::{
    ast::expressions::Expression,
    semantic::{EType, Resolve, SemanticError},
};

use super::string::STRING_HEADER;
use super::PathFinder;

#[derive(Debug, Clone, PartialEq)]
pub enum IOFn {
    Scan,
    Print { for_string: bool },
    Println { for_string: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub enum IOAsm {
    PrintStr,
    PrintString,
    PrintlnStr,
    PrintlnString,

    Flush,
    Flushln,

    Scan,
    RequestScan,
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for IOAsm {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            IOAsm::PrintStr => stdio.push_asm_lib(engine, "print"),
            IOAsm::PrintString => stdio.push_asm_lib(engine, "print"),
            IOAsm::PrintlnStr => stdio.push_asm_lib(engine, "println"),
            IOAsm::PrintlnString => stdio.push_asm_lib(engine, "println"),
            IOAsm::Flushln => stdio.push_asm_lib(engine, "flushln"),
            IOAsm::Flush => stdio.push_asm_lib(engine, "flush"),
            IOAsm::Scan => stdio.push_asm_lib(engine, "scan"),
            IOAsm::RequestScan => stdio.push_asm_lib(engine, "rscan"),
        }
    }
}

impl crate::vm::AsmWeight for IOAsm {
    fn weight(&self) -> crate::vm::Weight {
        match self {
            IOAsm::PrintStr | IOAsm::PrintString | IOAsm::PrintlnStr | IOAsm::PrintlnString => {
                crate::vm::Weight::ZERO
            }
            IOAsm::Flush => crate::vm::Weight::EXTREME,
            IOAsm::Flushln => crate::vm::Weight::EXTREME,
            IOAsm::Scan => crate::vm::Weight::END,
            IOAsm::RequestScan => crate::vm::Weight::END,
        }
    }
}

impl PathFinder for IOFn {
    fn find(path: &[String], name: &str) -> Option<Self>
    where
        Self: Sized,
    {
        if (path.len() == 1 && path[0] == lexem::IO) || path.len() == 0 {
            return match name {
                lexem::PRINT => Some(IOFn::Print { for_string: false }),
                lexem::PRINTLN => Some(IOFn::Println { for_string: false }),
                lexem::SCAN => Some(IOFn::Scan),
                _ => None,
            };
        }
        None
    }
}

impl ResolveCore for IOFn {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        match self {
            IOFn::Print { for_string } => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let param = parameters.first_mut().unwrap();
                let _ = param.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                match param.type_of(&scope_manager, scope_id)? {
                    EType::Static(StaticType::StrSlice(_)) => {
                        *for_string = false;
                    }
                    EType::Static(StaticType::String(_)) => {
                        *for_string = true;
                    }
                    _ => return Err(SemanticError::IncompatibleTypes),
                }
                Ok(e_static!(StaticType::Unit))
            }
            IOFn::Println { for_string } => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let param = parameters.first_mut().unwrap();
                let _ = param.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                match param.type_of(&scope_manager, scope_id)? {
                    EType::Static(StaticType::StrSlice(_)) => {
                        *for_string = false;
                    }
                    EType::Static(StaticType::String(_)) => {
                        *for_string = true;
                    }
                    _ => return Err(SemanticError::IncompatibleTypes),
                }
                Ok(e_static!(StaticType::Unit))
            }
            IOFn::Scan => {
                if parameters.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                Ok(e_static!(StaticType::String(StringType())))
            }
        }
    }
}

impl GenerateCode for IOFn {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            IOFn::Print { for_string } => {
                if *for_string {
                    instructions.push(Asm::Core(CoreAsm::IO(IOAsm::PrintString)));
                } else {
                    instructions.push(Asm::Core(CoreAsm::IO(IOAsm::PrintStr)));
                }
                Ok(())
            }
            IOFn::Println { for_string } => {
                if *for_string {
                    instructions.push(Asm::Core(CoreAsm::IO(IOAsm::PrintlnString)));
                } else {
                    instructions.push(Asm::Core(CoreAsm::IO(IOAsm::PrintlnStr)));
                }
                Ok(())
            }
            IOFn::Scan => {
                instructions.push(Asm::Core(super::CoreAsm::IO(IOAsm::RequestScan)));
                instructions.push(Asm::Core(super::CoreAsm::IO(IOAsm::Scan)));
                Ok(())
            }
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for IOAsm {
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
            IOAsm::PrintStr => {
                let address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let words = OpPrimitive::get_string_from(address, stack, heap)?;
                stdio.stdout.push(&words);
                stdio.stdout.flush(engine);

                scheduler.next();
                Ok(())
            }
            IOAsm::PrintString => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let words = OpPrimitive::get_string_from(address.add(POINTER_SIZE), stack, heap)?;
                stdio.stdout.push(&words);
                stdio.stdout.flush(engine);

                scheduler.next();
                Ok(())
            }
            IOAsm::PrintlnStr => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let words = OpPrimitive::get_string_from(address, stack, heap)?;
                stdio.stdout.push(&words);
                stdio.stdout.flushln(engine);

                scheduler.next();
                Ok(())
            }
            IOAsm::PrintlnString => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let words = OpPrimitive::get_string_from(address.add(POINTER_SIZE), stack, heap)?;
                stdio.stdout.push(&words);
                stdio.stdout.flushln(engine);

                scheduler.next();
                Ok(())
            }
            IOAsm::Flush => {
                stdio.stdout.flush(engine);
                scheduler.next();
                Ok(())
            }
            IOAsm::Flushln => {
                stdio.stdout.flushln(engine);
                scheduler.next();
                Ok(())
            }
            IOAsm::Scan => {
                let res = stdio.stdin.read(engine);
                if let Some(content) = res {
                    // Alloc and fill the string with the content, then ^push the address onto the stack
                    let len = content.len();
                    let cap = align(len as usize);
                    let alloc_size = cap + STRING_HEADER;

                    let len_bytes = len.to_le_bytes().as_slice().to_vec();
                    let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                    let address = heap.alloc(alloc_size as usize)?;

                    let data = content.as_bytes();
                    /* Write capacity */
                    let _ = heap.write(address, &cap_bytes)?;
                    /* Write len */
                    let _ = heap.write(address.add(8), &len_bytes)?;

                    /* Write slice */
                    let _ = heap.write(address.add(16), &data.to_vec())?;

                    let address: u64 = address.into(stack);
                    let _ = stack.push_with(&address.to_le_bytes())?;
                    scheduler.next();
                    Ok(())
                } else {
                    // the program instruction cursor is not incremented, therefore when content will be available in the stdin
                    // the instruction will be run again and only then the program cursor will get incremented
                    fn callback<E: crate::vm::external::Engine>(
                        response: crate::vm::signal::SignalResult<E>,
                        stack: &mut crate::vm::allocator::stack::Stack,
                    ) -> Result<(), RuntimeError> {
                        Ok(())
                    }
                    let _ = signal_handler.notify(
                        crate::vm::signal::Signal::WaitSTDIN,
                        stack,
                        engine,
                        context.tid.clone(),
                        callback::<E>,
                    )?;
                    Ok(())
                }
            }
            IOAsm::RequestScan => {
                stdio.stdin.request(context.tid.clone(), engine);
                fn callback<E: crate::vm::external::Engine>(
                    response: crate::vm::signal::SignalResult<E>,
                    stack: &mut crate::vm::allocator::stack::Stack,
                ) -> Result<(), RuntimeError> {
                    Ok(())
                }
                let _ = signal_handler.notify(
                    crate::vm::signal::Signal::WaitSTDIN,
                    stack,
                    engine,
                    context.tid.clone(),
                    callback::<E>,
                )?;
                scheduler.next();
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::statements::parse_statements,
        semantic::Resolve,
        test_extract_variable_with, test_statements,
        vm::{
            allocator::{heap::Heap, MemoryAddress},
            asm::operation::{GetNumFrom, OpPrimitive},
            external::test::DefaultProcessID,
            runtime::{Runtime, Thread, ThreadContext, ThreadState},
            scheduler::QueuePolicy,
            stdio::StdIO,
            GenerateCode,
        },
    };

    fn nil(
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        stack: &crate::vm::allocator::stack::Stack,
        heap: &crate::vm::allocator::heap::Heap,
    ) -> bool {
        true
    }
    #[test]
    fn valid_print() {
        let mut engine = crate::vm::external::test::StdoutTestEngine { out: String::new() };
        test_statements(
            r##"
        print("Hello World");
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "Hello World");

        test_statements(
            r##"
        io::print("你好世界");
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "你好世界");

        test_statements(
            r##"
        let text = string("lorem ipsum");
        core::io::print(text);
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "lorem ipsum");

        test_statements(
            r##"
        let text = string("lorem ipsum");
        println(text);
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "lorem ipsum\n");
    }

    #[test]
    fn valid_printf() {
        let mut engine = crate::vm::external::test::StdoutTestEngine { out: String::new() };

        test_statements(
            r##"
                printf("Hello World");
                "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "Hello World\n");

        for ptype in [
            "u128", "u64", "u32", "u16", "u8", "i128", "i64", "i32", "i16", "i8",
        ] {
            test_statements(
                &format!(
                    r##"
                    let x = 5{0};
                    printf("x = {{x}}");
                    "##,
                    ptype
                ),
                &mut engine,
                nil,
            );
            assert_eq!(engine.out, "x = 5\n");
        }

        test_statements(
            r##"
                let chara = 'a';
                printf("chara = {chara}");
                "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "chara = 'a'\n");

        test_statements(
            r##"
                let text = "Hello World";
                printf("text = {text}");
                "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "text = \"Hello World\"\n");

        test_statements(
            r##"
                let text = string("Hello World");
                printf("text = {text}");
                "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "text = \"Hello World\"\n");

        test_statements(
            r##"
        let tuple = (2,string("Hello World"));
        printf("tuple = {tuple}");
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "tuple = (2, \"Hello World\")\n");

        test_statements(
            r##"
        struct Point {
            x : i64,
            y : i64,
        }
        let point = Point { x: 1, y:2};
        printf("point = {point}");
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "point = Point { x: 1, y: 2 }\n");

        test_statements(
            r##"
        union Test {
            Point {
                x : i64,
                y : i64,
            },
            Test {
                x : u32,
            }
        }
        let test_union = Test::Point { x: 1, y:2 };
        printf("test_union = {test_union}");
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "test_union = Test::Point { x: 1, y: 2 }\n");

        test_statements(
            r##"
        enum Test {
            TEST1,
            TEST2,
        }
        let test_enum = Test::TEST2;
        printf("test_enum = {test_enum}");
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "test_enum = Test::TEST2\n");

        test_statements(
            r##"

        let lambda = (x:i64) -> {x + 1};
        printf("lambda = {lambda}");
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "lambda = (i64) -> i64\n");

        test_statements(
            r##"

        let closure = move (x:i64) -> {x + 1};
        printf("closure = {closure}");
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "closure = closed(i64) -> i64\n");

        test_statements(
            r##"

        fn function(x:i64) -> i64 {
            x + 1
        }
        printf("function = {function}");
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "function = fn(i64) -> i64\n");

        test_statements(
            r##"

        fn function(x:i64) -> i64 {
            x + 1
        }
        printf("function = {function}");
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "function = fn(i64) -> i64\n");

        test_statements(
            r##"
        printf("unit = {unit}");
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "unit = unit\n");

        test_statements(
            r##"
        let err = Ok();
        printf("err = {err}");
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "err = OK\n");

        test_statements(
            r##"
        let err = Err();
        printf("err = {err}");
        "##,
            &mut engine,
            nil,
        );
        assert_eq!(engine.out, "err = ERROR\n");

        test_statements(
            r##"
        let arr = [1,2,3];
        printf("arr = {arr}");
        "##,
            &mut engine,
            nil,
        );
        assert!(engine.out.starts_with("arr = [3]i64"));

        test_statements(
            r##"
        let arr = vec[1,2,3];
        printf("arr = {arr}");
        "##,
            &mut engine,
            nil,
        );
        assert!(engine.out.starts_with("arr = Vec[i64]"));

        test_statements(
            r##"
        let arr = map {
            1:2,
        };
        printf("arr = {arr}");
        "##,
            &mut engine,
            nil,
        );

        test_statements(
            r##"
        let x = 2;
        let addr = &x;

        printf("addr = {addr}");
        "##,
            &mut engine,
            nil,
        );
        assert!(engine.out.starts_with("addr = &i64"));
    }

    pub fn compile_for<E: crate::vm::external::Engine>(
        input: &str,
        tid: &E::TID,
        runtime: &mut Runtime<E, QueuePolicy>,
    ) {
        let mut statements = parse_statements::<E::PID, E::TID>(input.into(), 0)
            .expect("Parsing should have succeeded");

        let ThreadContext {
            scope_manager,
            program,
            ..
        } = runtime
            .context_of(&tid)
            .expect("Thread should have been found");

        for statement in statements.iter_mut() {
            statement
                .resolve::<E>(scope_manager, None, &None, &mut ())
                .expect(&format!("Resulotion should have succeeded {:?}", statement));
        }

        for statement in statements {
            statement
                .gencode::<E>(
                    scope_manager,
                    None,
                    program,
                    &crate::vm::CodeGenerationContext::default(),
                )
                .expect(&format!(
                    "Code generation should have succeeded {:?}",
                    statement
                ));
        }
    }

    #[test]
    fn valid_scan() {
        let mut engine = crate::vm::external::test::StdinTestEngine {
            out: String::new(),
            in_buf: String::new(),
        };

        let mut heap = Heap::new();
        let mut stdio = StdIO::default();
        let mut runtime = Runtime::default();

        let tid_1 = runtime
            .spawn(DefaultProcessID::default(), &mut engine)
            .expect("Spawning should have succeeded");

        compile_for(
            r##"
        let res = scan();
        "##,
            &tid_1,
            &mut runtime,
        );

        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        if ThreadState::WAITING_STDIN
            != *runtime
                .snapshot()
                .states
                .get(&tid_1)
                .expect("Thread should exist")
        {
            panic!("Thread should have been waiting");
        }

        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        if ThreadState::WAITING_STDIN
            != *runtime
                .snapshot()
                .states
                .get(&tid_1)
                .expect("Thread should exist")
        {
            panic!("Thread should have been waiting");
        }

        engine.in_buf.push_str("Hello World");
        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        if ThreadState::IDLE
            != *runtime
                .snapshot()
                .states
                .get(&tid_1)
                .expect("Thread should exist")
        {
            panic!("Thread should have been waiting");
        }
        {
            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_1)
                .expect("Thread should have been found");
            test_extract_variable_with(
                "res",
                |address, stack, heap| {
                    let address: MemoryAddress =
                        OpPrimitive::get_num_from::<u64>(address, stack, heap)
                            .expect("Deserialization should have succeeded")
                            .try_into()
                            .unwrap();
                    let address = address.add(8);

                    let text = OpPrimitive::get_string_from(address, stack, heap)
                        .expect("Deserialization should have succeeded");

                    assert_eq!(text, "Hello World");
                },
                scope_manager,
                stack,
                &heap,
            )
        }
    }

    // #[test]
    // fn valid_scan() {
    //     let mut engine = StdinTestEngine {
    //         out: String::new(),
    //         in_buf: String::new(),
    //     };
    //     let mut ciphel = Ciphel::new();
    //     let tid = ciphel
    //         .start_arena(&mut engine)
    //         .expect("starting should not fail");

    //     let src = r##"

    //     let res = scan();
    //     println(res);

    //     "##;

    //     ciphel
    //         .compile::<StdinTestEngine>(crate::vm::vm::Player::P1, tid, src)
    //         .expect("Compilation should have succeeded");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     engine.in_buf = "Hello World".to_string().into();
    //     ciphel.run(&mut engine).expect("no error should arise");

    //     let output = engine.out;
    //     assert_eq!(&output, "Hello World\n")
    // }
}
