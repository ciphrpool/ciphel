use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::statements::assignation::AssignValue,
    semantic::{scope::ScopeApi, MutRc, SizeOf},
    vm::{
        casm::{
            alloc::StackFrame,
            branch::{Call, Goto, Label},
            memcopy::MemCopy,
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{Assignation, Assignee};

impl<Scope: ScopeApi> GenerateCode<Scope> for Assignation<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        let _ = &self.right.gencode(scope, instructions)?;
        // match &self.right {
        //     AssignValue::Scope(right) => {

        //     },
        //     AssignValue::Expr(right) => {
        //         let _ = right.gencode(scope, instructions)?;
        //         self.left.gencode(scope, instructions)
        //     }
        // }

        self.left.gencode(scope, instructions)
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Assignee<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Assignee::Variable(variable) => {
                // Push the address of the variable on the stack
                let _ = variable.locate(scope, instructions)?;
                let mut borrowed_instructions = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                let var_size = {
                    let Some(var_type) = variable.signature() else {
                        return Err(CodeGenerationError::UnresolvedError);
                    };
                    var_type.size_of()
                };
                borrowed_instructions.push(Casm::MemCopy(MemCopy::TakeToStack { size: var_size }))
            }
            Assignee::PtrAccess(_) => todo!(),
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for AssignValue<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        match self {
            AssignValue::Scope(value) => {
                let mut borrowed_instructions = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;

                let end_scope_label = Label::gen();
                borrowed_instructions.push(Casm::Goto(Goto {
                    label: end_scope_label,
                }));
                let scope_label = borrowed_instructions.push_label("Scope".into());
                drop(borrowed_instructions);
                value.gencode(scope, instructions)?;

                let mut borrowed_instructions = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;

                let size = value
                    .metadata
                    .signature()
                    .map(|t| t.size_of())
                    .ok_or(CodeGenerationError::UnresolvedError)?;

                borrowed_instructions.push_label_id(end_scope_label, "End_Scope".into());
                borrowed_instructions.push(Casm::Call(Call {
                    label: scope_label,
                    return_size: size,
                    param_size: 0,
                }));
            }
            AssignValue::Expr(value) => value.gencode(scope, instructions)?,
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use num_traits::Zero;

    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive, Slice, Struct, Tuple},
                Atomic, Expression,
            },
            statements::{declaration, Statement},
            TryParse,
        },
        clear_stack,
        semantic::{
            scope::{
                scope_impl::Scope,
                static_types::{NumberType, PrimitiveType, SliceType, StaticType, TupleType},
                user_type_impl::{self, UserType},
            },
            Either, Resolve,
        },
        vm::{
            allocator::Memory,
            vm::{DeserializeFrom, Executable},
        },
    };

    use super::*;

    #[test]
    fn valid_assignation_in_scope() {
        let statement = Statement::parse(
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
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);
        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(420))));
    }

    #[test]
    fn valid_assignation_general_scope() {
        let declaration = Statement::parse(
            r##"
            let x:u64;
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let statement = Statement::parse(
            r##"
        x = 420;
    "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = declaration
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        declaration
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);
        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(420))));
    }

    #[test]
    fn valid_assignation_struct_in_scope() {
        let user_type = user_type_impl::Struct {
            id: "Point".into(),
            fields: {
                let mut res = Vec::new();
                res.push((
                    "x".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                ));
                res.push((
                    "y".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                ));
                res
            },
        };
        let statement = Statement::parse(
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
        let _ = scope
            .borrow_mut()
            .register_type(&"Point".into(), UserType::Struct(user_type.clone()))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);

        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let result: Struct<Scope> = user_type
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");
        for (r_id, res) in &result.fields {
            match res {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x.get() {
                        Number::I64(res) => {
                            if r_id == "x" {
                                assert_eq!(420, res);
                            } else if r_id == "y" {
                                assert_eq!(69, res);
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
        let statement = Statement::parse(
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
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);

        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let result: Tuple<Scope> = TupleType(vec![
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()),
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()),
        ])
        .deserialize_from(&data)
        .expect("Deserialization should have succeeded");
        let result: Vec<Option<u64>> = result
            .value
            .into_iter()
            .map(|e| match e {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x.get() {
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
        let statement = Statement::parse(
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
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);
        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let result: Slice<Scope> = SliceType {
            size: 4,
            item_type: Box::new(Either::Static(
                StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
            )),
        }
        .deserialize_from(&data)
        .expect("Deserialization should have succeeded");
        let result: Vec<Option<u64>> = result
            .value
            .into_iter()
            .map(|e| match e {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x.get() {
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
            id: "Point".into(),
            fields: {
                let mut res = Vec::new();
                res.push((
                    "x".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                ));
                res.push((
                    "y".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                ));
                res
            },
        };
        let statement = Statement::parse(
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
        let _ = scope
            .borrow_mut()
            .register_type(&"Point".into(), UserType::Struct(user_type.clone()))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);
        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let result: Struct<Scope> = user_type
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");

        for (r_id, res) in &result.fields {
            match res {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x.get() {
                        Number::I64(res) => {
                            if r_id == "x" {
                                assert_eq!(420, res);
                            } else if r_id == "y" {
                                assert_eq!(69, res);
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
            id: "Point".into(),
            fields: {
                let mut res = Vec::new();
                res.push((
                    "x".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                ));
                res.push((
                    "y".into(),
                    Either::Static(
                        StaticType::Slice(SliceType {
                            size: 4,
                            item_type: Box::new(Either::Static(
                                StaticType::Tuple(TupleType(vec![
                                    Either::Static(
                                        StaticType::Primitive(PrimitiveType::Number(
                                            NumberType::U64,
                                        ))
                                        .into(),
                                    ),
                                    Either::Static(
                                        StaticType::Primitive(PrimitiveType::Number(
                                            NumberType::U64,
                                        ))
                                        .into(),
                                    ),
                                ]))
                                .into(),
                            )),
                        })
                        .into(),
                    ),
                ));
                res
            },
        };
        let statement = Statement::parse(
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
        let _ = scope
            .borrow_mut()
            .register_type(&"Point".into(), UserType::Struct(user_type.clone()))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);

        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let result: Struct<Scope> = user_type
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
                            ))) => match x.get() {
                                Number::U64(n) => Some(n),
                                _ => None,
                            },
                            _ => None,
                        },
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                            match x.get() {
                                Number::U64(n) => Some(n),
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
            id: "Point3D".into(),
            fields: {
                let mut res = Vec::new();
                res.push((
                    "x".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                ));
                res.push((
                    "y".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                ));
                res
            },
        };
        let user_type_point = user_type_impl::Struct {
            id: "Point".into(),
            fields: {
                let mut res = Vec::new();
                res.push((
                    "x".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                ));
                res.push((
                    "y".into(),
                    Either::User(Rc::new(UserType::Struct(user_type_point3d.clone()))),
                ));
                res
            },
        };
        let statement = Statement::parse(
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
        let _ = scope
            .borrow_mut()
            .register_type(&"Point".into(), UserType::Struct(user_type_point.clone()))
            .expect("Registering of user type should have succeeded");
        let _ = scope
            .borrow_mut()
            .register_type(
                &"Point3D".into(),
                UserType::Struct(user_type_point3d.clone()),
            )
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);

        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let result: Struct<Scope> = user_type_point
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");
        for (r_id, res) in &result.fields {
            if r_id == "y" {
                match res {
                    Expression::Atomic(Atomic::Data(Data::Struct(Struct {
                        id,
                        fields,
                        metadata,
                    }))) => {
                        for (r_id, res) in fields {
                            if r_id == "y" {
                                match res {
                                    Expression::Atomic(Atomic::Data(Data::Primitive(
                                        Primitive::Number(x),
                                    ))) => match x.get() {
                                        Number::I64(n) => assert_eq!(n, 69),
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
}
