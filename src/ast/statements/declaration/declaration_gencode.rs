use crate::ast::statements::assignation::AssignValue;
use crate::semantic::scope::scope::VariableInfo;
use crate::vm::{CodeGenerationError, GenerateCode};
use crate::{
    ast::statements::declaration::{DeclaredVar, PatternVar},
    semantic::SizeOf,
    vm::asm::{mem::Mem, Asm},
};

use super::{Declaration, TypedVar};

impl GenerateCode for Declaration {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        fn store_right_side<E: crate::vm::external::Engine>(
            left: &DeclaredVar,
            right: &AssignValue,
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            scope_id: Option<u128>,
            instructions: &mut crate::vm::program::Program<E>,
            context: &crate::vm::CodeGenerationContext,
        ) -> Result<(), crate::vm::CodeGenerationError> {
            let Some(right_type) = (match right {
                AssignValue::Block(value) => value.metadata.signature(),
                AssignValue::Expr(value) => value.signature(),
            }) else {
                return Err(CodeGenerationError::UnresolvedError);
            };

            let _ = right.gencode::<E>(scope_manager, scope_id, instructions, context)?;

            match left {
                DeclaredVar::Id { id: Some(id), .. }
                | DeclaredVar::Typed(TypedVar { id: Some(id), .. }) => {
                    let Some(VariableInfo { address, .. }) = scope_manager.find_var_by_id(*id).ok()
                    else {
                        return Err(CodeGenerationError::UnresolvedError);
                    };
                    instructions.push(Asm::Mem(Mem::Store {
                        size: right_type.size_of(),
                        address: (*address)
                            .try_into()
                            .map_err(|_| CodeGenerationError::UnresolvedError)?,
                    }));
                }
                DeclaredVar::Pattern(PatternVar::Tuple { ids: Some(ids), .. })
                | DeclaredVar::Pattern(PatternVar::StructFields { ids: Some(ids), .. }) => {
                    for id in ids.iter().rev() {
                        let Some(VariableInfo { address, ctype, .. }) =
                            scope_manager.find_var_by_id(*id).ok()
                        else {
                            return Err(CodeGenerationError::UnresolvedError);
                        };
                        instructions.push(Asm::Mem(Mem::Store {
                            size: ctype.size_of(),
                            address: (*address)
                                .try_into()
                                .map_err(|_| CodeGenerationError::UnresolvedError)?,
                        }));
                    }
                }
                _ => {
                    return Err(CodeGenerationError::UnresolvedError);
                }
            }
            Ok(())
        }

        match self {
            Declaration::Declared(TypedVar { id, .. }) => {
                let Some(id) = id else {
                    return Err(CodeGenerationError::UnresolvedError);
                };

                if scope_manager.is_var_global(*id) {
                    let _ = scope_manager.alloc_global_var_by_id(*id)?;
                }
                Ok(())
            }
            Declaration::Assigned { left, right } => {
                // Alloc the variables
                match left {
                    DeclaredVar::Id { id: Some(id), .. }
                    | DeclaredVar::Typed(TypedVar { id: Some(id), .. }) => {
                        if scope_manager.is_var_global(*id) {
                            let _ = scope_manager.alloc_global_var_by_id(*id)?;
                        }
                    }
                    DeclaredVar::Pattern(PatternVar::StructFields { ids: Some(ids), .. })
                    | DeclaredVar::Pattern(PatternVar::Tuple { ids: Some(ids), .. }) => {
                        for id in ids {
                            if scope_manager.is_var_global(*id) {
                                let _ = scope_manager.alloc_global_var_by_id(*id)?;
                            }
                        }
                    }
                    _ => {
                        return Err(CodeGenerationError::UnresolvedError);
                    }
                }
                store_right_side(left, right, scope_manager, None, instructions, context)
            }
            Declaration::RecClosure {
                name,
                id,
                signature,
                right,
            } => {
                let Some(id) = id else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.metadata.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                if scope_manager.is_var_global(*id) {
                    let _ = scope_manager.alloc_global_var_by_id(*id)?;
                }

                let _ = right.gencode::<E>(scope_manager, scope_id, instructions, context)?;
                let Some(VariableInfo { address, .. }) = scope_manager.find_var_by_id(*id).ok()
                else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                instructions.push(Asm::Mem(Mem::Store {
                    size: right_type.size_of(),
                    address: (*address)
                        .try_into()
                        .map_err(|_| CodeGenerationError::UnresolvedError)?,
                }));
                Ok(())
            }
            Declaration::RecLambda {
                name,
                id,
                signature,
                right,
            } => {
                let Some(id) = id else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.metadata.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                if scope_manager.is_var_global(*id) {
                    let _ = scope_manager.alloc_global_var_by_id(*id)?;
                }

                let _ = right.gencode::<E>(scope_manager, scope_id, instructions, context)?;
                let Some(VariableInfo { address, .. }) = scope_manager.find_var_by_id(*id).ok()
                else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                instructions.push(Asm::Mem(Mem::Store {
                    size: right_type.size_of(),
                    address: (*address)
                        .try_into()
                        .map_err(|_| CodeGenerationError::UnresolvedError)?,
                }));
                Ok(())
            } // }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{test_extract_variable, test_statements};

    #[test]
    fn valid_declaration() {
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1);
            let res = test_extract_variable::<u32>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3);
            let res = test_extract_variable::<u8>("res4", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 4);

            let res = test_extract_variable::<i64>("res5", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);

            let res = test_extract_variable::<i64>("res6", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 6);

            let res = test_extract_variable::<i64>("x", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 7);

            let res = test_extract_variable::<i64>("y", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 8);

            let res = test_extract_variable::<i64>("a", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 9);

            let res = test_extract_variable::<u32>("b", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 10);

            let res = test_extract_variable::<i64>("c", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 11);
            true
        }

        test_statements(
            r##"

        let res1 = 1;
        let res2:u32 = 2;
        let res3 = {
            let x = 1;
            x + 2
        };
        let res4 : u8 = {
            let x = 3u8;
            x + 1
        };

        let (res5,res6) = (5,6);

        struct Point {
            x : i64,
            y : i64,
        }

        let Point {x,y} = Point {x:7,y:8};

        struct Test {
            a : i64,
            b : u32,
            c : i64,
        }
        
        let Test {a,b,c} = Test {a:9,b:10,c:11};
        
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_rec_functions() {
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 55);

            let res = test_extract_variable::<i64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 15);

            let res = test_extract_variable::<i64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 15);
            true
        }

        test_statements(
            r##"
        fn fibonacci(x:u64) -> u64 {
            if x == 0u64 {
                return 0;
            } else if x == 1u64 or x == 2u64 {
                return 1;
            }
            return fibonacci(x-1) + fibonacci(x-2);
        }
        let res1 = fibonacci(10);

        let rec lambda1 : (u64) -> u64 = (x) -> {
            if x == 0 {
                return 0;
            }
            return x + lambda1(x - 1);
        };

        let res2 = lambda1(5);

        let rec closure1 : (u64) -> u64 = (x) -> {
            if x == 0 {
                return 0;
            }
            return x + closure1(x - 1);
        };

        let res3 = closure1(5);
        "##,
            &mut engine,
            assert_fn,
        );
    }

    // #[test]
    // fn valid_declaration_inplace_in_scope() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let x:u64 = 420;
    //         return x;
    //     };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 420));
    // }

    // #[test]
    // fn valid_declaration_underscore() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let _ = 420;
    //         return 69;
    //     };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 69));
    // }

    // #[test]
    // fn valid_declaration_inplace_tuple_in_scope() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let (x,y) = {
    //         let (x,y) = (420,69);
    //         return (x,y);
    //     };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let x = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data[0..8],
    //     )
    //     .expect("Deserialization should have succeeded");
    //     let y = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data[8..16],
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(x, v_num!(I64, 420));
    //     assert_eq!(y, v_num!(I64, 69));
    // }

    // #[test]
    // fn valid_declaration_tuple_underscore() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let (x,y) = {
    //         let (x,_) = (420,69);
    //         return (x,70);
    //     };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let x = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data[0..8],
    //     )
    //     .expect("Deserialization should have succeeded");
    //     let y = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data[8..16],
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(x, v_num!(I64, 420));
    //     assert_eq!(y, v_num!(I64, 70));
    // }

    // #[test]
    // fn valid_declaration_inplace_tuple_general_scope() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let (x,y) = (420,69);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let x = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data[0..8],
    //     )
    //     .expect("Deserialization should have succeeded");
    //     let y = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data[8..16],
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(x, v_num!(I64, 420));
    //     assert_eq!(y, v_num!(I64, 69));
    // }

    // #[test]
    // fn valid_declaration_inplace_struct_in_scope() {
    //     let user_type = user_types::Struct {
    //         id: "Point".to_string().into(),
    //         fields: {
    //             let mut res = Vec::new();
    //             res.push(("x".to_string().into(), p_num!(I64)));
    //             res.push(("y".to_string().into(), p_num!(I64)));
    //             res
    //         },
    //     };
    //     let mut statement = Statement::parse(
    //         r##"
    //     let (x,y) = {
    //         let Point {x,y} = Point {
    //             x : 420,
    //             y : 69,
    //         };
    //         return (x,y);
    //     };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager
    //         .register_type("Point", UserType::Struct(user_type), None)
    //         .expect("Registering of user type should have succeeded");
    //     let _ = statement
    //         .resolve::<crate::vm::external::test::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = Program::default();
    //     statement
    //         .gencode::<E>(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);
    //     // Execute the instructions.

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::external::test::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);
    //     let x = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data[0..8],
    //     )
    //     .expect("Deserialization should have succeeded");
    //     let y = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data[8..16],
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(x, v_num!(I64, 420));
    //     assert_eq!(y, v_num!(I64, 69));
    // }

    // #[test]
    // fn valid_declaration_inplace_struct_general_scope() {
    //     let user_type = user_types::Struct {
    //         id: "Point".to_string().into(),
    //         fields: {
    //             let mut res = Vec::new();
    //             res.push(("x".to_string().into(), p_num!(I64)));
    //             res.push(("y".to_string().into(), p_num!(I64)));
    //             res
    //         },
    //     };
    //     let mut statement = Statement::parse(
    //         r##"
    //         let Point {x,y} = Point {
    //             x : 420,
    //             y : 69,
    //         };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager
    //         .register_type("Point", UserType::Struct(user_type), None)
    //         .expect("Registering of user type should have succeeded");
    //     let _ = statement
    //         .resolve::<crate::vm::external::test::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = Program::default();
    //     statement
    //         .gencode::<E>(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);
    //     // Execute the instructions.

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::external::test::NoopGameEngine {};
    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let x = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data[0..8],
    //     )
    //     .expect("Deserialization should have succeeded");
    //     let y = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data[8..16],
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(x, v_num!(I64, 420));
    //     assert_eq!(y, v_num!(I64, 69));
    // }

    // #[test]
    // fn valid_shadowing_same_type() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let var = 5;
    //             var = 6;
    //             let var = var + 4;
    //             return var + 10;
    //         };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();

    //     let _ = statement
    //         .resolve::<crate::vm::external::test::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = Program::default();
    //     statement
    //         .gencode::<E>(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);
    //     // Execute the instructions.

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::external::test::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let value = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data[0..8],
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(value, v_num!(I64, 20));
    // }

    // #[test]
    // fn valid_shadowing_different_type() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let var = 5u8;
    //             var = 6u8;
    //             let var = var as i64 + 4;
    //             return var + 10;
    //         };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();

    //     let _ = statement
    //         .resolve::<crate::vm::external::test::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = Program::default();
    //     statement
    //         .gencode::<E>(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);
    //     // Execute the instructions.

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::external::test::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);
    //     let engine = crate::vm::external::test::NoopGameEngine {};

    //     let value = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data[0..8],
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(value, v_num!(I64, 20));
    // }

    // #[test]
    // fn valid_shadowing_outer_scope() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let var = 5u8;
    //             let outer = {
    //                 let var = 10;
    //                 return var;
    //             };
    //             return outer + 10;
    //         };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();

    //     let _ = statement
    //         .resolve::<crate::vm::external::test::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = Program::default();
    //     statement
    //         .gencode::<E>(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);
    //     // Execute the instructions.

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::external::test::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let value = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data[0..8],
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(value, v_num!(I64, 20));
    // }
}
