use crate::{
    ast::{expressions::operation::ListAccess, statements::assignation::AssignValue},
    semantic::SizeOf,
    vm::{
        casm::{
            branch::{Call, Goto, Label},
            mem::Mem,
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode, Locatable},
    },
};

use super::Assignation;
use crate::semantic::scope::scope::Scope;

impl GenerateCode for Assignation {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let _ = &self.right.gencode(scope, instructions)?;
        // self.left.gencode(scope, instructions)
        let (var_size, is_utf8_char) = {
            let Some(var_type) = self.left.signature() else {
                return Err(CodeGenerationError::UnresolvedError);
            };
            (
                var_type.size_of(),
                match &self.left {
                    crate::ast::expressions::Expression::ListAccess(ListAccess {
                        metadata,
                        ..
                    }) => match metadata
                        .context()
                        .ok_or(CodeGenerationError::UnresolvedError)?
                    {
                        crate::semantic::Either::Static(value) => match value.as_ref() {
                            crate::semantic::scope::static_types::StaticType::String(_) => true,
                            crate::semantic::scope::static_types::StaticType::StrSlice(_) => true,
                            _ => false,
                        },
                        _ => false,
                    },
                    _ => false,
                },
            )
        };
        if var_size == 0 {
            return Ok(());
        }
        let _ = self.left.locate(scope, instructions)?;
        if is_utf8_char {
            instructions.push(Casm::Mem(Mem::TakeUTF8Char));
        } else {
            instructions.push(Casm::Mem(Mem::Take { size: var_size }));
        }
        Ok(())
    }
}

impl GenerateCode for AssignValue {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            AssignValue::Scope(value) => {
                let end_scope_label = Label::gen();
                instructions.push(Casm::Goto(Goto {
                    label: Some(end_scope_label),
                }));
                let scope_label = instructions.push_label("Scope".to_string().into());

                value.gencode(scope, instructions)?;

                instructions.push_label_id(end_scope_label, "End_Scope".to_string().into());
                instructions.push(Casm::Call(Call::From {
                    label: scope_label,
                    param_size: 0,
                }));
                instructions.push(Casm::Pop(9)); /* Pop the unused return size and return flag */
            }
            AssignValue::Expr(value) => value.gencode(scope, instructions)?,
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use num_traits::Zero;

    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive, Slice, Struct, Tuple},
                Atomic, Expression,
            },
            statements::Statement,
            TryParse,
        },
        clear_stack, compile_statement, p_num,
        semantic::{
            scope::{
                scope::Scope,
                static_types::{NumberType, PrimitiveType, SliceType, StaticType, TupleType},
                user_type_impl::{self, UserType},
            },
            Either, Resolve,
        },
        v_num,
        vm::vm::{DeserializeFrom, Executable, Runtime},
    };

    use super::*;

    #[test]
    fn valid_assignation_in_scope() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let y:u64;
            y = 420;
            return y;
        };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }

    #[test]
    fn valid_assignation_general_scope() {
        let mut declaration = Statement::parse(
            r##"
            let x:u64;
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let mut statement = Statement::parse(
            r##"
        x = 420;
    "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();

        let _ = declaration
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions = CasmProgram::default();
        declaration
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        program.merge(instructions);
        let mut engine = crate::vm::vm::DbgGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine,tid)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }

    #[test]
    fn valid_assignation_struct_in_scope() {
        let user_type = user_type_impl::Struct {
            id: "Point".to_string().into(),
            fields: {
                let mut res = Vec::new();
                res.push(("x".to_string().into(), p_num!(I64)));
                res.push(("y".to_string().into(), p_num!(I64)));
                res
            },
        };
        let mut statement = Statement::parse(
            r##"
        let x = {
            let point:Point;
            point = Point {
                x : 420,
                y : 69,
            };
            return point;
        };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Point".to_string().into(),
                UserType::Struct(user_type.clone()),
            )
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);

        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        program.merge(instructions);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine,tid)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);

        let result: Struct = user_type
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");
        for (r_id, res) in &result.fields {
            match res {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x {
                        Number::I64(res) => {
                            if **r_id == "x" {
                                assert_eq!(420, *res);
                            } else if **r_id == "y" {
                                assert_eq!(69, *res);
                            }
                        }
                        _ => assert!(false, "Expected i64"),
                    }
                }
                _ => assert!(false, "Expected i64"),
            }
        }
    }

    #[test]
    fn valid_assignation_tuple_access_in_scope() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let x:(u64,u64);
            x.1 = 420;
            return x;
        };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);

        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        program.merge(instructions);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine,tid)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);

        let result: Tuple = TupleType(vec![p_num!(U64), p_num!(U64)])
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");
        let result: Vec<Option<u64>> = result
            .value
            .into_iter()
            .map(|e| match e {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x {
                        Number::U64(n) => Some(n),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect();
        assert_eq!(result, vec![Some(0), Some(420)]);
    }

    #[test]
    fn valid_assignation_slice_access_in_scope() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let x:[4]u64;
            x[1] = 420;
            return x;
        };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);

        let result: Slice = SliceType {
            size: 4,
            item_type: Box::new(p_num!(U64)),
        }
        .deserialize_from(&data)
        .expect("Deserialization should have succeeded");
        let result: Vec<Option<u64>> = result
            .value
            .into_iter()
            .map(|e| match e {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x {
                        Number::U64(n) => Some(n),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect();
        assert_eq!(result, vec![Some(0), Some(420), Some(0), Some(0)]);
    }

    #[test]
    fn valid_assignation_field_access_in_scope() {
        let user_type = user_type_impl::Struct {
            id: "Point".to_string().into(),
            fields: {
                let mut res = Vec::new();
                res.push(("x".to_string().into(), p_num!(I64)));
                res.push(("y".to_string().into(), p_num!(I64)));
                res
            },
        };
        let mut statement = Statement::parse(
            r##"
        let x = {
            let point:Point;
            point.x = 420;
            point.y = 69;
            return point;
        };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Point".to_string().into(),
                UserType::Struct(user_type.clone()),
            )
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        program.merge(instructions);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine,tid)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);

        let result: Struct = user_type
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");

        for (r_id, res) in &result.fields {
            match res {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x {
                        Number::I64(res) => {
                            if **r_id == "x" {
                                assert_eq!(420, *res);
                            } else if **r_id == "y" {
                                assert_eq!(69, *res);
                            }
                        }
                        _ => assert!(false, "Expected i64"),
                    }
                }
                _ => assert!(false, "Expected i64"),
            }
        }
    }

    #[test]
    fn valid_assignation_complex_in_scope() {
        let user_type = user_type_impl::Struct {
            id: "Point".to_string().into(),
            fields: {
                let mut res = Vec::new();
                res.push(("x".to_string().into(), p_num!(I64)));
                res.push((
                    "y".to_string().into(),
                    Either::Static(
                        StaticType::Slice(SliceType {
                            size: 4,
                            item_type: Box::new(Either::Static(
                                StaticType::Tuple(TupleType(vec![p_num!(U64), p_num!(U64)])).into(),
                            )),
                        })
                        .into(),
                    ),
                ));
                res
            },
        };
        let mut statement = Statement::parse(
            r##"
        let x = {
            let point:Point;
            point.y[1].1 = 69;
            return point;
        };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Point".to_string().into(),
                UserType::Struct(user_type.clone()),
            )
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);

        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        program.merge(instructions);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine,tid)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);

        let result: Struct = user_type
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");

        let (_, y) = &result.fields[1];
        match y {
            Expression::Atomic(Atomic::Data(Data::Slice(Slice { value, metadata }))) => {
                let result: Vec<Option<u64>> = value
                    .into_iter()
                    .map(|e| match e {
                        Expression::Atomic(Atomic::Data(Data::Tuple(Tuple {
                            value,
                            metadata,
                        }))) => match &value[1] {
                            Expression::Atomic(Atomic::Data(Data::Primitive(
                                Primitive::Number(x),
                            ))) => match x {
                                Number::U64(n) => Some(*n),
                                _ => None,
                            },
                            _ => None,
                        },
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                            match x {
                                Number::U64(n) => Some(*n),
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                    .collect();
                assert_eq!(result, vec![Some(0), Some(69), Some(0), Some(0)]);
            }
            _ => assert!(false, "Expected u64"),
        }
    }

    #[test]
    fn valid_assignation_double_field_in_scope() {
        let user_type_point3d = user_type_impl::Struct {
            id: "Point3D".to_string().into(),
            fields: {
                let mut res = Vec::new();
                res.push(("x".to_string().into(), p_num!(I64)));
                res.push(("y".to_string().into(), p_num!(I64)));
                res
            },
        };
        let user_type_point = user_type_impl::Struct {
            id: "Point".to_string().into(),
            fields: {
                let mut res = Vec::new();
                res.push(("x".to_string().into(), p_num!(I64)));
                res.push((
                    "y".to_string().into(),
                    Either::User(Arc::new(UserType::Struct(user_type_point3d.clone()))),
                ));
                res
            },
        };
        let mut statement = Statement::parse(
            r##"
        let x = {
            let point:Point;
            point.y.y = 69;
            return point;
        };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Point".to_string().into(),
                UserType::Struct(user_type_point.clone()),
            )
            .expect("Registering of user type should have succeeded");
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Point3D".to_string().into(),
                UserType::Struct(user_type_point3d.clone()),
            )
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);

        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        program.merge(instructions);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine,tid)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);

        let result: Struct = user_type_point
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");
        for (r_id, res) in &result.fields {
            if **r_id == "y" {
                match res {
                    Expression::Atomic(Atomic::Data(Data::Struct(Struct {
                        id,
                        fields,
                        metadata,
                    }))) => {
                        for (r_id, res) in fields {
                            if **r_id == "y" {
                                match res {
                                    Expression::Atomic(Atomic::Data(Data::Primitive(
                                        Primitive::Number(x),
                                    ))) => match x {
                                        Number::I64(n) => assert_eq!(*n, 69),
                                        _ => assert!(false, "Expected i64"),
                                    },
                                    _ => assert!(false, "Expected i64"),
                                }
                            }
                        }
                    }
                    _ => assert!(false, "Expected i64"),
                }
            }
        }
    }

    #[test]
    fn valid_assignation_ptr_access_complex() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let arr = vec[1,2,3,4];
            *(((arr as &Any) as u64 + 16) as &u64) = 2;
            return arr[0]; 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 2));
    }
}
