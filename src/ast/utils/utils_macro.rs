#[macro_export]
macro_rules! e_static {
    ($type_def:expr) => {
        crate::semantic::EType::Static($type_def.into())
    };
}

// #[macro_export]
// macro_rules! arw_read {
//     ($var:expr,$err:expr) => {
//         $var.try_read().map_err(|_| {
//             // panic!("Concucurrency Read error");
//             $err
//         })
//     };
// }

#[macro_export]
macro_rules! am_read {
    ($var:expr,$err:expr) => {
        $var.try_read().map_err(|_| {
            // panic!("Concucurrency Read error");
            $err
        })
    };
}

#[macro_export]
macro_rules! am_write {
    ($var:expr,$err:expr) => {
        $var.try_read().map_err(|_| {
            // panic!("Concucurrency Read error");
            $err
        })
    };
}

// #[macro_export]
// macro_rules! arw_write {
//     ($var:expr,$err:expr) => {
//         $var.try_write().map_err(|_| {
//             // panic!("Concucurrency Write error");
//             $err
//         })
//     };
// }
#[macro_export]
macro_rules! arw_new {
    ($var:expr) => {
        Arc::new(RwLock::new($var))
    };
}

#[macro_export]
macro_rules! e_user {
    ($type_def:expr) => {
        crate::semantic::EType::User($type_def)
    };
}

#[macro_export]
macro_rules! p_num {
    ($num:ident) => {
        crate::semantic::EType::Static(crate::semantic::scope::static_types::StaticType::Primitive(
            crate::semantic::scope::static_types::PrimitiveType::Number(
                crate::semantic::scope::static_types::NumberType::$num,
            ),
        ))
    };
}

#[macro_export]
macro_rules! v_num {
    ($type_def:ident,$num:expr) => {
        crate::ast::expressions::data::Primitive::Number(
            crate::ast::expressions::data::Number::$type_def($num),
        )
    };
}

#[macro_export]
macro_rules! err_tuple {
    ($value:expr) => {
        e_static!(crate::semantic::scope::static_types::StaticType::Tuple(
            crate::semantic::scope::static_types::TupleType(vec![
                $value,
                e_static!(StaticType::Error)
            ])
        ))
    };
}

#[macro_export]
macro_rules! clear_stack {
    ($memory:ident) => {{
        use num_traits::Zero;
        let top = $memory.top();
        let data = $memory
            .pop(top)
            .expect("Read should have succeeded")
            .to_owned();
        assert!($memory.top().is_zero());
        data
    }};
}
#[macro_export]
macro_rules! assert_number {
    ($expr:ident,$data:ident,$num_type:ident) => {
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(crate::semantic::scope::static_types::NumberType::$num_type),
            &$data,
        )
        .expect("Deserialization should have succeeded");

        // Assert and return the result.
        assert_eq!(result, $expr);
    };
}

// #[macro_export]
// macro_rules! compile_expression {
//     ($expr_type:ident,$expr_str:expr) => {{
//         // Parse the expression.
//         let mut expr = $expr_type::parse($expr_str.into())
//             .expect("Parsing should have succeeded")
//             .1;

//         let mut engine = crate::vm::external::test::NoopGameEngine {};
//         // Create a new block.
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         // Perform semantic check.
//         expr.resolve::<crate::vm::external::test::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Semantic resolution should have succeeded");

//         // Code generation.
//         let mut instructions = Program::default();
//         expr.gencode::<E>(
//             &mut scope_manager,
//             None,
//             &mut instructions,
//             &crate::vm::CodeGenerationContext::default(),
//         )
//         .expect("Code generation should have succeeded");
//         assert!(instructions.len() > 0);

//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         (expr, data)
//     }};
// }

// #[macro_export]
// macro_rules! compile_expression_with_type {
//     ($expr_type:ident,$expr_str:expr,$user_type_str:expr,$user_type:ident) => {{
//         let mut engine = crate::vm::external::test::NoopGameEngine {};
//         // Parse the expression.
//         let mut expr = $expr_type::parse($expr_str.into())
//             .expect("Parsing should have succeeded")
//             .1;

//         // Create a new block.
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = scope_manager
//             .register_type(
//                 $user_type_str,
//                 UserType::$expr_type($user_type.clone()),
//                 None,
//             )
//             .expect("Type registering should have succeeded");
//         // Perform semantic check.
//         expr.resolve::<crate::vm::external::test::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Semantic resolution should have succeeded");

//         // Code generation.
//         let mut instructions = Program::default();
//         expr.gencode::<E>(
//             &mut scope_manager,
//             None,
//             &mut instructions,
//             &crate::vm::CodeGenerationContext::default(),
//         )
//         .expect("Code generation should have succeeded");
//         assert!(instructions.len() > 0);

//         // Execute the instructions.

//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);
//         (expr, data)
//     }};
// }

// #[macro_export]
// macro_rules! compile_statement {
//     ($statement:ident) => {{
//         let mut engine = crate::vm::vm::DbgGameEngine {};
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = $statement
//             .resolve::<crate::vm::vm::DbgGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Semantic resolution should have succeeded");

//         // Code generation.
//         let mut instructions = Program::default();
//         $statement
//             .gencode::<E>(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");
//         assert!(instructions.len() > 0);
//         let (mut runtime, mut heap, mut stdio) = crate::vm::vm::Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);
//         dbg!(&program.main);
//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = crate::clear_stack!(memory);
//         data.to_owned()
//     }};
// }
// #[macro_export]
// macro_rules! compile_statement_for_stdout {
//     ($statement:ident) => {{
//         let mut engine = crate::vm::vm::StdoutTestGameEngine { out: String::new() };
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = $statement
//             .resolve::<crate::vm::vm::StdoutTestGameEngine>(
//                 &mut scope_manager,
//                 None,
//                 &None,
//                 &mut (),
//             )
//             .expect("Resolution should have succeeded");
//         // Code generation.
//         let mut instructions = Program::default();
//         $statement
//             .gencode::<E>(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::CodeGenerationContext::default(),
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

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let output = engine.out;
//         output
//     }};
// }
// #[macro_export]
// macro_rules! compile_statement_for_string {
//     ($statement:ident) => {{
//         let mut engine = crate::vm::external::test::NoopGameEngine {};
//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = $statement
//             .resolve::<crate::vm::external::test::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
//             .expect("Semantic resolution should have succeeded");

//         // Code generation.
//         let mut instructions = Program::default();
//         $statement
//             .gencode::<E>(
//                 &mut scope_manager,
//                 None,
//                 &mut instructions,
//                 &crate::vm::CodeGenerationContext::default(),
//             )
//             .expect("Code generation should have succeeded");

//         assert!(instructions.len() > 0);
//         let (mut runtime, mut heap, mut stdio) = Runtime::new();
//         let tid = runtime
//             .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
//             .expect("Thread spawn_with_scopeing should have succeeded");
//         let (_, stack, program) = runtime
//             .get_mut(crate::vm::vm::Player::P1, tid)
//             .expect("Thread should exist");
//         program.merge(instructions);

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
//             .read(
//                 crate::vm::allocator::MemoryAddress::Heap {
//                     offset: heap_address,
//                 },
//                 8,
//             )
//             .expect("length should be readable");
//         let length = u64::from_le_bytes(
//             TryInto::<[u8; 8]>::try_into(&data_length[0..8])
//                 .expect("heap address should be deserializable"),
//         ) as usize;

//         let data = heap
//             .read(
//                 crate::vm::allocator::MemoryAddress::Heap {
//                     offset: heap_address,
//                 },
//                 length + 16,
//             )
//             .expect("length should be readable");
//         let result = <StringType as DeserializeFrom>::deserialize_from(&StringType(), &data)
//             .expect("Deserialization should have succeeded")
//             .value;
//         result
//     }};
// }

// #[macro_export]
// macro_rules! eval_and_compare {
//     ($expr:expr, $expected:expr,$size:ident) => {{
//         let mut engine = crate::vm::external::test::NoopGameEngine {};

//         let mut expr = Expression::parse($expr.into())
//             .expect("Parsing should have succeeded")
//             .1;

//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = expr
//             .resolve::<crate::vm::external::test::NoopGameEngine>(&mut scope_manager, None, &None, &mut None)
//             .expect("Semantic resolution should have succeeded");

//         // Code generation.
//         let mut instructions = Program::default();
//         expr.gencode::<E>(
//             &mut scope_manager,
//             None,
//             &mut instructions,
//             &crate::vm::CodeGenerationContext::default(),
//         )
//         .expect("Code generation should have succeeded");

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

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);

//         let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
//             &PrimitiveType::Number(NumberType::$size),
//             &data,
//         )
//         .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result, $expected,
//             "Result does not match the expected value"
//         );
//     }};
// }

// #[macro_export]
// macro_rules! eval_and_compare_bool {
//     ($expr:expr, $expected:expr) => {{
//         let mut engine = crate::vm::external::test::NoopGameEngine {};

//         let mut expr = Expression::parse($expr.into())
//             .expect("Parsing should have succeeded")
//             .1;

//         let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
//         let _ = expr
//             .resolve::<crate::vm::external::test::NoopGameEngine>(&mut scope_manager, None, &None, &mut None)
//             .expect("Semantic resolution should have succeeded");

//         // Code generation.
//         let mut instructions = Program::default();
//         expr.gencode::<E>(
//             &mut scope_manager,
//             None,
//             &mut instructions,
//             &crate::vm::CodeGenerationContext::default(),
//         )
//         .expect("Code generation should have succeeded");

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

//         program
//             .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
//             .expect("Execution should have succeeded");
//         let memory = stack;
//         let data = clear_stack!(memory);

//         let result =
//             <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Bool, &data)
//                 .expect("Deserialization should have succeeded");

//         assert_eq!(
//             result, $expected,
//             "Result does not match the expected value"
//         );
//     }};
// }
