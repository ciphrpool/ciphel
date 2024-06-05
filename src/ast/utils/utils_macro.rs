#[macro_export]
macro_rules! e_static {
    ($type_def:expr) => {
        crate::semantic::Either::Static($type_def.into())
    };
}

#[macro_export]
macro_rules! arw_read {
    ($var:expr,$err:expr) => {
        $var.try_read().map_err(|_| {
            panic!("Concucurrency Read error");
            $err
        })
    };
}

#[macro_export]
macro_rules! arw_write {
    ($var:expr,$err:expr) => {
        $var.try_write().map_err(|_| {
            panic!("Concucurrency Write error");
            $err
        })
    };
}
#[macro_export]
macro_rules! arw_new {
    ($var:expr) => {
        Arc::new(RwLock::new($var))
    };
}

#[macro_export]
macro_rules! e_user {
    ($type_def:expr) => {
        crate::semantic::Either::User($type_def.into())
    };
}

#[macro_export]
macro_rules! p_num {
    ($num:ident) => {
        crate::semantic::Either::Static(
            crate::semantic::scope::static_types::StaticType::Primitive(
                crate::semantic::scope::static_types::PrimitiveType::Number(
                    crate::semantic::scope::static_types::NumberType::$num,
                ),
            )
            .into(),
        )
    };
}

#[macro_export]
macro_rules! v_num {
    ($type_def:ident,$num:expr) => {
        crate::ast::expressions::data::Primitive::Number(Cell::new(
            crate::ast::expressions::data::Number::$type_def($num),
        ))
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
macro_rules! resolve_metadata {
    ($info:expr,$self:expr,$scope:expr,$context:expr) => {{
        $info = crate::semantic::Info::Resolved {
            context: $context.clone(),
            signature: Some(
                $self.type_of(&crate::arw_read!($scope, SemanticError::ConcurrencyError)?)?,
            ),
        };
    }};
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
            &PrimitiveType::Number(NumberType::$num_type),
            &$data,
        )
        .expect("Deserialization should have succeeded");

        // Assert and return the result.
        assert_eq!(result, $expr);
    };
}

#[macro_export]
macro_rules! compile_expression {
    ($expr_type:ident,$expr_str:expr) => {{
        // Parse the expression.
        let mut expr = $expr_type::parse($expr_str.into())
            .expect("Parsing should have succeeded")
            .1;

        // Create a new block.
        let scope = Scope::new();
        // Perform semantic check.
        expr.resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        expr.gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");
        assert!(instructions.len() > 0);

        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions);

        let mut engine = crate::vm::vm::NoopGameEngine {};
        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);
        (expr, data)
    }};
}

#[macro_export]
macro_rules! compile_expression_with_type {
    ($expr_type:ident,$expr_str:expr,$user_type:ident) => {{
        // Parse the expression.
        let mut expr = $expr_type::parse($expr_str.into())
            .expect("Parsing should have succeeded")
            .1;

        // Create a new block.
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, crate::SemanticError::ConcurrencyError)
            .unwrap()
            .register_type(
                &$user_type.id.clone(),
                UserType::$expr_type($user_type.clone().into()),
            )
            .expect("Type registering should have succeeded");
        // Perform semantic check.
        expr.resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        expr.gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");
        assert!(instructions.len() > 0);

        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);
        (expr, data)
    }};
}

#[macro_export]
macro_rules! compile_statement {
    ($statement:ident) => {{
        let scope = Scope::new();
        let _ = $statement
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        $statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");
        assert!(instructions.len() > 0);
        let (mut runtime, mut heap, mut stdio) = crate::vm::vm::Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions);
        let mut engine = crate::vm::vm::NoopGameEngine {};
        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = crate::clear_stack!(memory);
        data.to_owned()
    }};
}
#[macro_export]
macro_rules! compile_statement_for_stdout {
    ($statement:ident) => {{
        let scope = Scope::new();
        let _ = $statement
            .resolve(&scope, &None, &mut ())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        $statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions);

        let mut engine = crate::vm::vm::StdoutTestGameEngine { out: String::new() };
        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        let output = engine.out;
        output
    }};
}
#[macro_export]
macro_rules! compile_statement_for_string {
    ($statement:ident) => {{
        let scope = Scope::new();
        let _ = $statement
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        $statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
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
            .expect("Deserialization should have succeeded")
            .value;
        result
    }};
}

#[macro_export]
macro_rules! eval_and_compare {
    ($expr:expr, $expected:expr,$size:ident) => {{
        // Assuming `Expression`, `Scope`, `CasmProgram`, `Memory`, and `Primitive` are defined in the context.
        let mut expr = Expression::parse($expr.into()).expect("Parsing should have succeeded").1;

        let scope = Scope::new();
        let _ = expr
            .resolve(&scope, &None, &mut None)
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        expr.gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");

        // Execute the instructions.
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap,&mut  stdio,&mut engine)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::$size),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(result, $expected, "Result does not match the expected value");
    }};
}

#[macro_export]
macro_rules! eval_and_compare_bool {
    ($expr:expr, $expected:expr) => {{
        // Assuming `Expression`, `Scope`, `CasmProgram`, `Memory`, and `Primitive` are defined in the context.
        let mut expr = Expression::parse($expr.into()).expect("Parsing should have succeeded").1;

        let scope = Scope::new();
        let _ = expr
            .resolve(&scope, &None, &mut None)
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        expr.gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");

        // Execute the instructions.
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap,&mut  stdio,&mut engine)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Bool,
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(result, $expected, "Result does not match the expected value");
    }};
}
