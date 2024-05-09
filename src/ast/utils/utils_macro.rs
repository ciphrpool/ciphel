#[macro_export]
macro_rules! e_static {
    ($type_def:expr) => {
        Either::Static($type_def.into())
    };
}

#[macro_export]
macro_rules! e_user {
    ($type_def:expr) => {
        Either::User($type_def.into())
    };
}

#[macro_export]
macro_rules! p_num {
    ($num:ident) => {
        Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::$num)).into())
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
        e_static!(StaticType::Tuple(TupleType(vec![
            $value,
            e_static!(StaticType::Error)
        ])))
    };
}

#[macro_export]
macro_rules! resolve_metadata {
    ($metadata:expr,$self:expr,$scope:expr,$context:expr) => {{
        let mut borrowed_metadata = $metadata
            .info
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| SemanticError::Default)?;
        *borrowed_metadata = Info::Resolved {
            context: $context.clone(),
            signature: Some($self.type_of(&$scope.borrow())?),
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
        let expr = $expr_type::parse($expr_str.into())
            .expect("Parsing should have succeeded")
            .1;

        // Create a new block.
        let scope = Scope::new();
        // Perform semantic check.
        expr.resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        expr.gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");
        assert!(instructions.len() > 0);

        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::<crate::vm::vm::NoopGameEngine>::new();
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
        let expr = $expr_type::parse($expr_str.into())
            .expect("Parsing should have succeeded")
            .1;

        // Create a new block.
        let scope = Scope::new();
        let _ = scope
            .as_ref()
            .borrow_mut()
            .register_type(
                &$user_type.id.clone(),
                UserType::$expr_type($user_type.clone().into()),
            )
            .expect("Type registering should have succeeded");
        // Perform semantic check.
        expr.resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        expr.gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");
        assert!(instructions.len() > 0);

        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::<crate::vm::vm::NoopGameEngine>::new();
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
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        $statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");
        assert!(instructions.len() > 0);
        let (mut runtime, mut heap, mut stdio) =
            crate::vm::vm::Runtime::<crate::vm::vm::NoopGameEngine>::new();
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
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        $statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let (mut runtime, mut heap, mut stdio) =
            Runtime::<crate::vm::vm::StdoutTestGameEngine>::new();
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
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        $statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let (mut runtime, mut heap, mut stdio) = Runtime::<crate::vm::vm::NoopGameEngine>::new();
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
        let expr = Expression::parse($expr.into()).expect("Parsing should have succeeded").1;

        let scope = Scope::new();
        let _ = expr
            .resolve(&scope, &None, &None)
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        expr.gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");

        // Execute the instructions.
        let (mut runtime, mut heap, mut stdio) = Runtime::<crate::vm::vm::NoopGameEngine>::new();
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
        let expr = Expression::parse($expr.into()).expect("Parsing should have succeeded").1;

        let scope = Scope::new();
        let _ = expr
            .resolve(&scope, &None, &None)
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        expr.gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");

        // Execute the instructions.
        let (mut runtime, mut heap, mut stdio) = Runtime::<crate::vm::vm::NoopGameEngine>::new();
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
