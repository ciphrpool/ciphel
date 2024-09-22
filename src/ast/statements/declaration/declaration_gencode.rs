use crate::ast::statements::assignation::AssignValue;
use crate::ast::utils::lexem;
use crate::semantic::scope::scope::{ScopeManager, Variable, VariableInfo};
use crate::{
    ast::statements::declaration::{DeclaredVar, PatternVar},
    semantic::SizeOf,
    vm::{
        allocator::MemoryAddress,
        casm::{alloc::Alloc, locate::Locate, mem::Mem, Casm, CasmProgram},
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{Declaration, TypedVar};

impl GenerateCode for Declaration {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        fn store_right_side(
            left: &DeclaredVar,
            right: &AssignValue,
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            scope_id: Option<u128>,
            instructions: &mut CasmProgram,
            context: &crate::vm::vm::CodeGenerationContext,
        ) -> Result<(), CodeGenerationError> {
            let first_variable_id = match left {
                DeclaredVar::Id { id: Some(id), .. }
                | DeclaredVar::Typed(TypedVar { id: Some(id), .. }) => *id,
                DeclaredVar::Pattern(PatternVar::StructFields { ids: Some(ids), .. })
                | DeclaredVar::Pattern(PatternVar::Tuple { ids: Some(ids), .. }) => {
                    *ids.first().ok_or(CodeGenerationError::UnresolvedError)?
                }
                _ => {
                    return Err(CodeGenerationError::UnresolvedError);
                }
            };

            let _ = right.gencode(scope_manager, scope_id, instructions, context)?;
            let Some(right_type) = (match right {
                crate::ast::statements::assignation::AssignValue::Block(value) => {
                    value.metadata.signature()
                }
                crate::ast::statements::assignation::AssignValue::Expr(value) => value.signature(),
            }) else {
                return Err(CodeGenerationError::UnresolvedError);
            };

            // memcopy the right side at the address of the first varairable offset
            // if there is multiple variable ( in the case of destructuring ) the variables are aligned and in order
            // and the right side is packed
            let Some(VariableInfo { address, .. }) =
                scope_manager.find_var_by_id(first_variable_id).ok()
            else {
                return Err(CodeGenerationError::UnresolvedError);
            };

            instructions.push(Casm::Mem(Mem::Store {
                size: right_type.size_of(),
                address: (*address)
                    .try_into()
                    .map_err(|_| CodeGenerationError::UnresolvedError)?,
            }));
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
            Declaration::RecClosure { left, right } => todo!(),
            // }
        }
    }
}

#[cfg(test)]
mod tests {

    use num_traits::Zero;

    use crate::{
        ast::{statements::Statement, TryParse},
        p_num,
        semantic::{
            scope::{
                scope::ScopeManager,
                static_types::{NumberType, PrimitiveType},
                user_type_impl::{self, UserType},
            },
            Resolve,
        },
        v_num,
        vm::{
            casm::CasmProgram,
            vm::{Executable, Runtime},
        },
    };

    use super::*;

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
    //     let user_type = user_type_impl::Struct {
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
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = CasmProgram::default();
    //     statement
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::vm::CodeGenerationContext::default(),
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
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

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
    //     let user_type = user_type_impl::Struct {
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
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = CasmProgram::default();
    //     statement
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::vm::CodeGenerationContext::default(),
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
    //     let mut engine = crate::vm::vm::NoopGameEngine {};
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
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = CasmProgram::default();
    //     statement
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::vm::CodeGenerationContext::default(),
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
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

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
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = CasmProgram::default();
    //     statement
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::vm::CodeGenerationContext::default(),
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
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);
    //     let engine = crate::vm::vm::NoopGameEngine {};

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
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = CasmProgram::default();
    //     statement
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::vm::CodeGenerationContext::default(),
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
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

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
