use std::{cell::RefCell, rc::Rc};

use num_traits::ToBytes;

use crate::{
    semantic::{scope::ScopeApi, MutRc, SizeOf},
    vm::{
        casm::{
            alloc::StackFrame,
            branch::{BranchIf, Label},
            serialize::Serialized,
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{ExprFlow, FnCall, IfExpr, MatchExpr, TryExpr};

impl<Scope: ScopeApi> GenerateCode<Scope> for ExprFlow<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        match self {
            ExprFlow::If(value) => value.gencode(scope, instructions),
            ExprFlow::Match(value) => value.gencode(scope, instructions),
            ExprFlow::Try(value) => value.gencode(scope, instructions),
            ExprFlow::Call(value) => value.gencode(scope, instructions),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for IfExpr<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        let _ = self.condition.gencode(scope, &instructions)?;

        let Some(return_size) = self.metadata.signature().map(|t| t.size_of()) else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let mut borrowed = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;
        let else_label = Label::gen();
        borrowed.push(Casm::If(BranchIf { else_label }));
        let mock_program = Rc::new(RefCell::new(CasmProgram::default()));
        let _ = self.then_branch.gencode(scope, &mock_program)?;
        let then_code_length = mock_program.as_ref().borrow().len();
        let mock_program = Rc::new(RefCell::new(CasmProgram::default()));
        let _ = self.else_branch.gencode(scope, &mock_program)?;
        let else_code_length = mock_program.as_ref().borrow().len();

        borrowed.push(Casm::StackFrame(StackFrame::Set {
            return_size,
            cursor_offset: then_code_length + else_code_length + 3,
        }));
        drop(borrowed);

        let _ = self.then_branch.gencode(scope, &instructions)?;

        let mut borrowed = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;

        borrowed.push_label_id(else_label, "else".into());
        borrowed.push(Casm::StackFrame(StackFrame::Set {
            return_size,
            cursor_offset: else_code_length + 1,
        }));
        drop(borrowed);
        let _ = self.else_branch.gencode(scope, &instructions)?;

        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for MatchExpr<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for TryExpr<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for FnCall<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive, Struct},
                Atomic, Expression,
            },
            TryParse,
        },
        clear_stack,
        semantic::{
            scope::{
                scope_impl::Scope,
                static_types::{NumberType, PrimitiveType, StaticType},
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
    fn valid_if_basic() {
        let statement_then = IfExpr::parse(
            r##"
           if true then 420 else 69 
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let statement_else = IfExpr::parse(
            r##"
           if false then 420 else 69 
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = statement_then
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");
        let _ = statement_else
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions_then = Rc::new(RefCell::new(CasmProgram::default()));
        statement_then
            .gencode(&scope, &instructions_then)
            .expect("Code generation should have succeeded");
        let instructions_else = Rc::new(RefCell::new(CasmProgram::default()));
        statement_else
            .gencode(&scope, &instructions_else)
            .expect("Code generation should have succeeded");

        let instructions_then = instructions_then.as_ref().take();
        let instructions_else = instructions_else.as_ref().take();
        assert!(instructions_then.len() > 0);
        assert!(instructions_else.len() > 0);
        // Execute the instructions.
        let memory = Memory::new();
        instructions_then
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Number::U64(420)));

        let memory = Memory::new();
        instructions_else
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Number::U64(69)));
    }

    #[test]
    fn valid_if_basic_scope() {
        let statement_then = IfExpr::parse(
            r##"
           if true then { 
               let x = 420;
               return x;
           } else 69 
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let statement_else = IfExpr::parse(
            r##"
           if false then 420 else { 
            let x = 69;
            return x;
            } 
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = statement_then
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");
        let _ = statement_else
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions_then = Rc::new(RefCell::new(CasmProgram::default()));
        statement_then
            .gencode(&scope, &instructions_then)
            .expect("Code generation should have succeeded");
        let instructions_else = Rc::new(RefCell::new(CasmProgram::default()));
        statement_else
            .gencode(&scope, &instructions_else)
            .expect("Code generation should have succeeded");

        let instructions_then = instructions_then.as_ref().take();
        let instructions_else = instructions_else.as_ref().take();
        assert!(instructions_then.len() > 0);
        assert!(instructions_else.len() > 0);
        // Execute the instructions.
        let memory = Memory::new();
        instructions_then
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Number::U64(420)));

        let memory = Memory::new();
        instructions_else
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Number::U64(69)));
    }

    #[test]
    fn valid_if_complex() {
        let user_type = user_type_impl::Struct {
            id: "Point".into(),
            fields: {
                let mut res = Vec::new();
                res.push((
                    "x".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    ),
                ));
                res.push((
                    "y".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    ),
                ));
                res
            },
        };
        let statement_then = IfExpr::parse(
            r##"
        if true then {
            let point:Point;
            point.x = 420;
            point.y = 420;
            return point;
        } else Point {
            x : 69,
            y : 69
        }
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let statement_else = IfExpr::parse(
            r##"
        if false then {
            let point:Point;
            point.x = 420;
            point.y = 420;
            return point;
        } else Point {
            x : 69,
            y : 69
        }
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
        let _ = statement_then
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");
        let _ = statement_else
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions_then = Rc::new(RefCell::new(CasmProgram::default()));
        statement_then
            .gencode(&scope, &instructions_then)
            .expect("Code generation should have succeeded");
        let instructions_else = Rc::new(RefCell::new(CasmProgram::default()));
        statement_else
            .gencode(&scope, &instructions_else)
            .expect("Code generation should have succeeded");

        let instructions_then = instructions_then.as_ref().take();
        let instructions_else = instructions_else.as_ref().take();
        assert!(instructions_then.len() > 0);
        assert!(instructions_else.len() > 0);
        // Execute the instructions.
        let memory = Memory::new();
        instructions_then
            .execute(&memory)
            .expect("Execution should have succeeded");

        let data = clear_stack!(memory);
        let result: Struct<Scope> = user_type
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");

        for (r_id, res) in &result.fields {
            match res {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                    Number::U64(res),
                )))) => {
                    if r_id == "x" {
                        assert_eq!(420, *res);
                    } else if r_id == "y" {
                        assert_eq!(420, *res);
                    }
                }
                _ => assert!(false, "Expected u64"),
            }
        }

        // Execute the instructions.
        let memory = Memory::new();
        instructions_else
            .execute(&memory)
            .expect("Execution should have succeeded");

        let data = clear_stack!(memory);
        let result: Struct<Scope> = user_type
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");

        for (r_id, res) in &result.fields {
            match res {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                    Number::U64(res),
                )))) => {
                    if r_id == "x" {
                        assert_eq!(69, *res);
                    } else if r_id == "y" {
                        assert_eq!(69, *res);
                    }
                }
                _ => assert!(false, "Expected u64"),
            }
        }
    }
}
