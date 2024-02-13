use num_traits::ToBytes;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use ulid::Ulid;

use crate::{
    semantic::{
        scope::{
            static_types::StaticType,
            user_type_impl::{Enum, Union, UserType},
            ScopeApi,
        },
        AccessLevel, Either, MutRc, SizeOf,
    },
    vm::{
        allocator::{stack::Offset, MemoryAddress},
        casm::{
            alloc::{Access, StackFrame},
            branch::{BranchIf, BranchTable, Call, Goto, Label},
            locate::Locate,
            memcopy::MemCopy,
            serialize::Serialized,
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{ExprFlow, FnCall, IfExpr, MatchExpr, Pattern, PatternExpr, TryExpr};

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
        let Some(return_size) = self.metadata.signature().map(|t| t.size_of()) else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let else_label = Label::gen();
        let if_scope_label = Label::gen();
        let end_if_scope_label = Label::gen();
        let else_scope_label = Label::gen();
        let end_else_scope_label = Label::gen();
        let end_ifelse_label = Label::gen();
        {
            let mut borrowed = instructions
                .as_ref()
                .try_borrow_mut()
                .map_err(|_| CodeGenerationError::Default)?;

            let if_label = borrowed.push_label("If".into());
        }
        let _ = self.condition.gencode(scope, &instructions)?;
        {
            let mut borrowed = instructions
                .as_ref()
                .try_borrow_mut()
                .map_err(|_| CodeGenerationError::Default)?;
            borrowed.push(Casm::If(BranchIf { else_label }));
            borrowed.push(Casm::Goto(Goto {
                label: end_if_scope_label,
            }));
            borrowed.push_label_id(if_scope_label, "if_scope".into());
        }
        let _ = self.then_branch.gencode(scope, &instructions)?;
        {
            let mut borrowed = instructions
                .as_ref()
                .try_borrow_mut()
                .map_err(|_| CodeGenerationError::Default)?;
            borrowed.push_label_id(end_if_scope_label, "end_if_scope".into());
            borrowed.push(Casm::Call(Call {
                label: if_scope_label,
                return_size,
                param_size: 0,
            }));
            borrowed.push(Casm::Goto(Goto {
                label: end_ifelse_label,
            }));
        }
        {
            let mut borrowed = instructions
                .as_ref()
                .try_borrow_mut()
                .map_err(|_| CodeGenerationError::Default)?;
            borrowed.push_label_id(else_label, "else".into());
            borrowed.push(Casm::Goto(Goto {
                label: end_else_scope_label,
            }));
            borrowed.push_label_id(else_scope_label, "else_scope".into());
        }
        let _ = self.else_branch.gencode(scope, &instructions)?;
        {
            let mut borrowed = instructions
                .as_ref()
                .try_borrow_mut()
                .map_err(|_| CodeGenerationError::Default)?;

            borrowed.push_label_id(end_else_scope_label, "end_else_scope".into());
            borrowed.push(Casm::Call(Call {
                label: else_scope_label,
                return_size,
                param_size: 0,
            }));
            borrowed.push(Casm::Goto(Goto {
                label: end_ifelse_label,
            }));

            borrowed.push_label_id(end_ifelse_label, "end_if_else".into());
        }

        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for MatchExpr<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        let Some(return_size) = self.metadata.signature().map(|t| t.size_of()) else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let Some(expr_type) = self.expr.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let exhaustive_cases = match expr_type {
            Either::Static(ref value) => match value.as_ref() {
                StaticType::Primitive(_) => None,
                StaticType::String(_) => None,
                _ => return Err(CodeGenerationError::UnresolvedError),
            },
            Either::User(ref value) => match value.as_ref() {
                UserType::Struct(_) => return Err(CodeGenerationError::UnresolvedError),
                UserType::Enum(Enum { id, values }) => Some(values.clone()),
                UserType::Union(Union { id, variants }) => {
                    Some(variants.iter().map(|(v, _)| v).cloned().collect())
                }
            },
        };
        let mut borrowed = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;

        let end_match_label = Label::gen();
        // borrowed.push(Casm::Goto(Goto {
        //     label: end_match_label,
        // }));
        let match_label = borrowed.push_label("Match".into());
        drop(borrowed);

        let mut cases: Vec<Ulid> = Vec::with_capacity(self.patterns.len());
        let mut table: HashMap<u64, Ulid> = HashMap::with_capacity(self.patterns.len());

        for PatternExpr { pattern, .. } in &self.patterns {
            let label: Ulid = Label::gen();
            cases.push(label);
            match pattern {
                Pattern::Enum { value, .. } => {
                    if let Some(idx) = exhaustive_cases
                        .as_ref()
                        .map(|e| {
                            e.iter()
                                .enumerate()
                                .find_map(|(idx, id)| (id == value).then(|| idx))
                        })
                        .flatten()
                    {
                        table.insert(idx as u64, label);
                    }
                }
                Pattern::Union { variant, .. } => {
                    if let Some(idx) = exhaustive_cases
                        .as_ref()
                        .map(|e| {
                            e.iter()
                                .enumerate()
                                .find_map(|(idx, id)| (id == variant).then(|| idx))
                        })
                        .flatten()
                    {
                        table.insert(idx as u64, label);
                    }
                }
                _ => {}
            }
        }
        let else_label = match &self.else_branch {
            Some(_) => Some(Label::gen()),
            None => None,
        };
        // gencode of matched expression
        let _ = self.expr.gencode(scope, instructions)?;

        if table.len() == 0 {
            // Switch with branch if statements
        } else {
            // Switch with branch table statement
            // extrart variant from matched expression
            let mut borrowed = instructions
                .as_ref()
                .try_borrow_mut()
                .map_err(|_| CodeGenerationError::Default)?;
            // borrowed.push(Casm::Access(Access::Static {
            //     address: MemoryAddress::Stack {
            //         offset: Offset::ST(-8),
            //     },
            //     size: 8,
            // }));
            borrowed.push(Casm::Switch(BranchTable { table, else_label }))
        }
        for (idx, (PatternExpr { pattern, expr }, label)) in
            self.patterns.iter().zip(cases).enumerate()
        {
            let mut borrowed = instructions
                .as_ref()
                .try_borrow_mut()
                .map_err(|_| CodeGenerationError::Default)?;
            borrowed.push_label_id(label, format!("match_case_{}", idx).into());
            let end_scope_label = Label::gen();
            borrowed.push(Casm::Goto(Goto {
                label: end_scope_label,
            }));
            let scope_label = borrowed.push_label("Scope".into());
            drop(borrowed);
            let _ = expr.gencode(scope, instructions)?;

            let param_size = expr
                .parameters_size()
                .map_err(|_| CodeGenerationError::UnresolvedError)?;

            let mut borrowed = instructions
                .as_ref()
                .try_borrow_mut()
                .map_err(|_| CodeGenerationError::Default)?;

            borrowed.push_label_id(end_scope_label, "End_Scope".into());
            borrowed.push(Casm::Locate(Locate {
                address: MemoryAddress::Stack {
                    offset: Offset::FP(0),
                    level: AccessLevel::Direct,
                },
            }));
            borrowed.push(Casm::MemCopy(MemCopy::TakeToStack { size: param_size }));
            borrowed.push(Casm::Call(Call {
                label: scope_label,
                return_size,
                param_size,
            }));
            borrowed.push(Casm::Goto(Goto {
                label: end_match_label,
            }));
        }
        match &self.else_branch {
            Some(else_branch) => {
                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push_label_id(else_label.unwrap(), "else_case".into());
                drop(borrowed);
                let _ = else_branch.gencode(scope, instructions)?;
            }
            None => {}
        }
        let mut borrowed = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;
        borrowed.push_label_id(end_match_label, "end_match_else".into());
        Ok(())
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
            statements::Statement,
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

    #[test]
    fn valid_if_complex_outvar() {
        let statement_then = Statement::parse(
            r##"
        let x = {
            let y = true;
            return if y then 420 else 69;
        };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = statement_then
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions_then = Rc::new(RefCell::new(CasmProgram::default()));
        statement_then
            .gencode(&scope, &instructions_then)
            .expect("Code generation should have succeeded");

        let instructions_then = instructions_then.as_ref().take();
        assert!(instructions_then.len() > 0);
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
    }

    #[test]
    fn valid_match_union() {
        let user_type = user_type_impl::Union {
            id: "Geo".into(),
            variants: {
                let mut res = Vec::new();
                res.push((
                    "Point".into(),
                    user_type_impl::Struct {
                        id: "Point".into(),
                        fields: vec![
                            (
                                "x".into(),
                                Either::Static(
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64))
                                        .into(),
                                ),
                            ),
                            (
                                "y".into(),
                                Either::Static(
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64))
                                        .into(),
                                ),
                            ),
                        ],
                    },
                ));
                res.push((
                    "Axe".into(),
                    user_type_impl::Struct {
                        id: "Axe".into(),
                        fields: {
                            let mut res = Vec::new();
                            res.push((
                                "x".into(),
                                Either::Static(
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64))
                                        .into(),
                                ),
                            ));
                            res
                        },
                    },
                ));
                res
            },
        };
        let statement = Statement::parse(
            r##"
            let x:u64 = {
                let geo = Geo::Point {
                    x : 420,
                    y: 69,
                };
                let z = 27;
                return match geo {
                    case Geo::Point {x,y} => z,
                    case Geo::Axe {x} => x,
                };
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_type(&"Geo".into(), UserType::Union(user_type))
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
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");

        let data = clear_stack!(memory);
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Number::U64(27)));
    }
}
