use num_traits::ToBytes;

use ulid::Ulid;

use crate::semantic::scope::scope_impl::Scope;
use crate::{
    ast::expressions::data::{Number, Primitive},
    semantic::{
        scope::{
            static_types::{ClosureType, NumberType, StaticType},
            user_type_impl::{Enum, Union, UserType},
            var_impl::VarState,
        },
        Either, MutRc, SizeOf, TypeOf,
    },
    vm::{
        casm::{
            alloc::Access,
            branch::{BranchIf, BranchTable, Call, Goto, Label},
            data::Data,
            mem::Mem,
            operation::{Addition, OpPrimitive, Operation, OperationKind, Substraction},
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{ExprFlow, FnCall, IfExpr, MatchExpr, Pattern, PatternExpr, TryExpr};

impl GenerateCode for ExprFlow {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            ExprFlow::If(value) => value.gencode(scope, instructions),
            ExprFlow::Match(value) => value.gencode(scope, instructions),
            ExprFlow::Try(value) => value.gencode(scope, instructions),
            ExprFlow::Call(value) => value.gencode(scope, instructions),
            ExprFlow::SizeOf(value, _metadata) => {
                let value = value
                    .type_of(&scope.borrow())
                    .map_err(|_| CodeGenerationError::UnresolvedError)?;

                instructions.push(Casm::Data(Data::Serialized {
                    data: (value.size_of() as u64).to_le_bytes().into(),
                }));
                Ok(())
            }
        }
    }
}

impl GenerateCode for IfExpr {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(_return_size) = self.metadata.signature().map(|t| t.size_of()) else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let else_label = Label::gen();
        let if_scope_label = Label::gen();
        let end_if_scope_label = Label::gen();
        let else_scope_label = Label::gen();
        let end_else_scope_label = Label::gen();
        let end_ifelse_label = Label::gen();

        let _if_label = instructions.push_label("If".into());

        let _ = self.condition.gencode(scope, &instructions)?;

        instructions.push(Casm::If(BranchIf { else_label }));
        instructions.push(Casm::Goto(Goto {
            label: Some(end_if_scope_label),
        }));
        instructions.push_label_id(if_scope_label, "if_scope".into());

        let _ = self.then_branch.gencode(scope, &instructions)?;

        instructions.push_label_id(end_if_scope_label, "end_if_scope".into());
        instructions.push(Casm::Call(Call::From {
            label: if_scope_label,
            param_size: 0,
        }));
        instructions.push(Casm::Pop(9)); /* Pop the unused return size and return flag */
        instructions.push(Casm::Goto(Goto {
            label: Some(end_ifelse_label),
        }));

        instructions.push_label_id(else_label, "else".into());
        instructions.push(Casm::Goto(Goto {
            label: Some(end_else_scope_label),
        }));
        instructions.push_label_id(else_scope_label, "else_scope".into());

        let _ = self.else_branch.gencode(scope, &instructions)?;

        instructions.push_label_id(end_else_scope_label, "end_else_scope".into());
        instructions.push(Casm::Call(Call::From {
            label: else_scope_label,
            param_size: 0,
        }));
        instructions.push(Casm::Pop(9)); /* Pop the unused return size and return flag */
        instructions.push(Casm::Goto(Goto {
            label: Some(end_ifelse_label),
        }));

        instructions.push_label_id(end_ifelse_label, "end_if_else".into());

        Ok(())
    }
}

impl GenerateCode for MatchExpr {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(_return_size) = self.metadata.signature().map(|t| t.size_of()) else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let Some(expr_type) = self.expr.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let exhaustive_cases = match expr_type {
            Either::Static(ref value) => match value.as_ref() {
                StaticType::Primitive(_) => None,
                StaticType::String(_) => None,
                StaticType::StrSlice(_) => None,
                _ => return Err(CodeGenerationError::UnresolvedError),
            },
            Either::User(ref value) => match value.as_ref() {
                UserType::Struct(_) => return Err(CodeGenerationError::UnresolvedError),
                UserType::Enum(Enum { id: _, values }) => Some(values.clone()),
                UserType::Union(Union { id: _, variants }) => {
                    Some(variants.iter().map(|(v, _)| v).cloned().collect())
                }
            },
        };

        let end_match_label = Label::gen();
        let _match_label = instructions.push_label("Match".into());

        let mut cases: Vec<Ulid> = Vec::with_capacity(self.patterns.len());

        let mut dump_data: Vec<Box<[u8]>> = Vec::with_capacity(self.patterns.len());

        // let mut table: Vec<(u64, Ulid)> = Vec::with_capacity(self.patterns.len());
        // let mut switch: Vec<(Vec<u8>, Ulid)> = Vec::with_capacity(self.patterns.len());

        let switch_size = match &expr_type {
            Either::User(value) => match value.as_ref() {
                UserType::Enum(_) | UserType::Union(_) => 8,
                _ => expr_type.size_of(),
            },
            _ => expr_type.size_of(),
        };

        for PatternExpr { patterns, .. } in &self.patterns {
            let label: Ulid = Label::gen();
            for pattern in patterns {
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
                            dump_data.push((idx as u64).to_le_bytes().into());
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
                            dump_data.push((idx as u64).to_le_bytes().into());
                        }
                    }
                    Pattern::Primitive(value) => {
                        let data = match value {
                            Primitive::Number(data) => match data.get() {
                                Number::U8(data) => data.to_le_bytes().into(),
                                Number::U16(data) => data.to_le_bytes().into(),
                                Number::U32(data) => data.to_le_bytes().into(),
                                Number::U64(data) => data.to_le_bytes().into(),
                                Number::U128(data) => data.to_le_bytes().into(),
                                Number::I8(data) => data.to_le_bytes().into(),
                                Number::I16(data) => data.to_le_bytes().into(),
                                Number::I32(data) => data.to_le_bytes().into(),
                                Number::I64(data) => data.to_le_bytes().into(),
                                Number::I128(data) => data.to_le_bytes().into(),
                                Number::F64(data) => data.to_le_bytes().into(),
                                _ => return Err(CodeGenerationError::UnresolvedError),
                            },
                            Primitive::Bool(data) => [*data as u8].into(),
                            Primitive::Char(data) => {
                                let mut buffer = [0u8; 4];
                                let _ = data.encode_utf8(&mut buffer);
                                buffer.into()
                            }
                        };
                        dump_data.push(data);
                    }
                    Pattern::String(value) => {
                        let data = value.value.as_bytes().into();
                        // TODO : Maybe add size after data
                        dump_data.push(data);
                    }
                }
            }
        }

        let else_label = match &self.else_branch {
            Some(_) => Some(Label::gen()),
            None => None,
        };

        let dump_data_label = instructions.push_data(Data::Dump {
            data: dump_data.into(),
        });
        let table_data_label = instructions.push_data(Data::Table {
            data: cases.clone().into(),
        });

        // gencode of matched expression
        let _ = self.expr.gencode(scope, instructions)?;

        instructions.push(Casm::Switch(BranchTable::Swith {
            size: Some(switch_size),
            data_label: Some(dump_data_label),
            else_label: else_label,
        }));
        instructions.push(Casm::Switch(BranchTable::Table {
            table_label: Some(table_data_label),
            else_label: else_label,
        }));

        for (idx, (PatternExpr { patterns: _, expr }, label)) in
            self.patterns.iter().zip(cases).enumerate()
        {
            instructions.push_label_id(label, format!("match_case_{}", idx).into());
            let end_scope_label = Label::gen();
            instructions.push(Casm::Goto(Goto {
                label: Some(end_scope_label),
            }));
            let scope_label = instructions.push_label("Scope".into());
            let _ = expr.gencode(scope, instructions)?;

            // let param_size = expr
            //     .parameters_size()
            //     .map_err(|_| CodeGenerationError::UnresolvedError)?;
            let param_size = expr
                .scope()
                .map(|s| {
                    s.as_ref()
                        .borrow()
                        .vars()
                        .filter_map(|(v, _)| {
                            (v.state.get() == VarState::Parameter).then(|| v.type_sig.size_of())
                        })
                        .sum()
                })
                .map_err(|_| CodeGenerationError::UnresolvedError)?;
            instructions.push_label_id(end_scope_label, "End_Scope".into());
            instructions.push(Casm::Call(Call::From {
                label: scope_label,
                param_size,
            }));
            instructions.push(Casm::Pop(9)); /* Pop the unused return size and return flag */
            instructions.push(Casm::Goto(Goto {
                label: Some(end_match_label),
            }));
        }
        match &self.else_branch {
            Some(else_branch) => {
                instructions.push_label_id(else_label.unwrap(), "else_case".into());
                let end_scope_label = Label::gen();
                instructions.push(Casm::Goto(Goto {
                    label: Some(end_scope_label),
                }));
                let scope_label = instructions.push_label("Scope".into());
                let _ = else_branch.gencode(scope, instructions)?;

                instructions.push_label_id(end_scope_label, "End_Scope".into());
                instructions.push(Casm::Call(Call::From {
                    label: scope_label,
                    param_size: 0,
                }));
                instructions.push(Casm::Pop(9)); /* Pop the unused return size and return flag */
                instructions.push(Casm::Goto(Goto {
                    label: Some(end_match_label),
                }));
            }
            None => {}
        }

        instructions.push_label_id(end_match_label, "end_match_else".into());
        Ok(())
    }
}

impl GenerateCode for TryExpr {
    fn gencode(
        &self,
        _scope: &MutRc<Scope>,
        _instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl GenerateCode for FnCall {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let params_size: usize = self
            .params
            .iter()
            .map(|p| p.signature().map_or(0, |s| s.size_of()))
            .sum();

        if let Some(platform_api) = self.platform.as_ref().borrow().as_ref() {
            for param in &self.params {
                let _ = param.gencode(scope, instructions)?;
            }
            platform_api.gencode(scope, instructions)
        } else {
            let Some(Either::Static(fn_sig)) = self.fn_var.signature() else {
                return Err(CodeGenerationError::UnresolvedError);
            };
            let Some(signature) = self.metadata.signature() else {
                return Err(CodeGenerationError::UnresolvedError);
            };
            let sig_params_size = match fn_sig.as_ref() {
                StaticType::Closure(value) => value.scope_params_size,
                StaticType::StaticFn(value) => value.scope_params_size,
                _ => return Err(CodeGenerationError::UnresolvedError),
            };
            let _return_size = signature.size_of();

            match fn_sig.as_ref() {
                StaticType::Closure(ClosureType { closed: false, .. })
                | StaticType::StaticFn(_) => {
                    /* Call static function */
                    // Load Param
                    for param in &self.params {
                        let _ = param.gencode(scope, instructions)?;
                    }
                    let _ = self.fn_var.gencode(scope, instructions)?;
                    if let Some(8) = sig_params_size.checked_sub(params_size) {
                        // Load function address
                        instructions.push(Casm::MemCopy(Mem::Dup(8)));
                    }
                    // Call function
                    // Load param size
                    instructions.push(Casm::Data(Data::Serialized {
                        data: (sig_params_size as u64).to_le_bytes().into(),
                    }));

                    instructions.push(Casm::Call(Call::Stack));
                    instructions.push(Casm::Pop(9)); /* Pop the unused return size and return flag */
                }
                StaticType::Closure(ClosureType { closed: true, .. }) => {
                    // Load Param
                    for param in &self.params {
                        let _ = param.gencode(scope, instructions)?;
                    }

                    let _ = self.fn_var.gencode(scope, instructions)?;
                    match sig_params_size.checked_sub(params_size) {
                        Some(16) => {
                            /* Rec and closed */
                            /* PARAMS + [8] heap pointer to fn + [8] env heap pointer + [8] function pointer ( instruction offset stored in the heap)*/
                            instructions.push(Casm::MemCopy(Mem::Dup(8)));
                            // Load Env heap address
                            instructions.push(Casm::Data(Data::Serialized {
                                data: (16u64).to_le_bytes().into(),
                            }));
                            instructions.push(Casm::Operation(Operation {
                                kind: OperationKind::Addition(Addition {
                                    left: OpPrimitive::Number(NumberType::U64),
                                    right: OpPrimitive::Number(NumberType::U64),
                                }),
                            }));
                            instructions.push(Casm::MemCopy(Mem::Dup(8)));
                            instructions.push(Casm::Data(Data::Serialized {
                                data: (16u64).to_le_bytes().into(),
                            }));
                            instructions.push(Casm::Operation(Operation {
                                kind: OperationKind::Substraction(Substraction {
                                    left: OpPrimitive::Number(NumberType::U64),
                                    right: OpPrimitive::Number(NumberType::U64),
                                }),
                            }));
                            instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
                        }
                        Some(8) => {
                            /* closed */
                            /* PARAMS + [8] env heap pointer + [8] function pointer ( instruction offset stored in the heap)*/
                            // Load Env heap address
                            instructions.push(Casm::Data(Data::Serialized {
                                data: (16u64).to_le_bytes().into(),
                            }));
                            instructions.push(Casm::Operation(Operation {
                                kind: OperationKind::Addition(Addition {
                                    left: OpPrimitive::Number(NumberType::U64),
                                    right: OpPrimitive::Number(NumberType::U64),
                                }),
                            }));
                            instructions.push(Casm::MemCopy(Mem::Dup(8)));
                            instructions.push(Casm::Data(Data::Serialized {
                                data: (16u64).to_le_bytes().into(),
                            }));
                            instructions.push(Casm::Operation(Operation {
                                kind: OperationKind::Substraction(Substraction {
                                    left: OpPrimitive::Number(NumberType::U64),
                                    right: OpPrimitive::Number(NumberType::U64),
                                }),
                            }));
                            instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
                        }
                        _ => return Err(CodeGenerationError::UnresolvedError),
                    }

                    // Call function

                    // Load param size
                    // instructions.push(Casm::MemCopy(MemCopy::GetReg(UReg::R3))); // env size
                    instructions.push(Casm::Data(Data::Serialized {
                        data: (sig_params_size).to_le_bytes().into(),
                    }));

                    instructions.push(Casm::Call(Call::Stack));
                    instructions.push(Casm::Pop(9)); /* Pop the unused return size and return flag */
                }
                _ => {
                    return Err(CodeGenerationError::UnresolvedError);
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive, Struct},
                Atomic, Expression,
            },
            statements::Statement,
            TryParse,
        },
        clear_stack, compile_statement, p_num,
        semantic::{
            scope::{
                scope_impl::Scope,
                static_types::{NumberType, PrimitiveType, StaticType},
                user_type_impl::{self, UserType},
            },
            Either, Resolve,
        },
        v_num,
        vm::{
            allocator::Memory,
            vm::{DeserializeFrom, Executable, Runtime},
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
        let instructions_then = CasmProgram::default();
        statement_then
            .gencode(&scope, &instructions_then)
            .expect("Code generation should have succeeded");
        let instructions_else = CasmProgram::default();
        statement_else
            .gencode(&scope, &instructions_else)
            .expect("Code generation should have succeeded");

        assert!(instructions_then.len() > 0);
        assert!(instructions_else.len() > 0);
        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions_then);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));

        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions_else);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 69));
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
        let instructions_then = CasmProgram::default();
        statement_then
            .gencode(&scope, &instructions_then)
            .expect("Code generation should have succeeded");
        let instructions_else = CasmProgram::default();
        statement_else
            .gencode(&scope, &instructions_else)
            .expect("Code generation should have succeeded");

        assert!(instructions_then.len() > 0);
        assert!(instructions_else.len() > 0);
        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions_then);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));

        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions_else);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 69));
    }

    #[test]
    fn valid_if_complex() {
        let user_type = user_type_impl::Struct {
            id: "Point".into(),
            fields: {
                let mut res = Vec::new();
                res.push(("x".into(), p_num!(I64)));
                res.push(("y".into(), p_num!(I64)));
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
        let instructions_then = CasmProgram::default();
        statement_then
            .gencode(&scope, &instructions_then)
            .expect("Code generation should have succeeded");
        let instructions_else = CasmProgram::default();
        statement_else
            .gencode(&scope, &instructions_else)
            .expect("Code generation should have succeeded");

        assert!(instructions_then.len() > 0);
        assert!(instructions_else.len() > 0);
        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions_then);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result: Struct = user_type
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
                                assert_eq!(420, res);
                            }
                        }
                        _ => assert!(false, "Expected i64"),
                    }
                }
                _ => assert!(false, "Expected i64"),
            }
        }

        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions_else);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result: Struct = user_type
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");

        for (r_id, res) in &result.fields {
            match res {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x.get() {
                        Number::I64(res) => {
                            if r_id == "x" {
                                assert_eq!(69, res);
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
        let instructions_then = CasmProgram::default();
        statement_then
            .gencode(&scope, &instructions_then)
            .expect("Code generation should have succeeded");

        assert!(instructions_then.len() > 0);
        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions_then);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
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
                        fields: vec![("x".into(), p_num!(I64)), ("y".into(), p_num!(I64))],
                    },
                ));
                res.push((
                    "Axe".into(),
                    user_type_impl::Struct {
                        id: "Axe".into(),
                        fields: {
                            let mut res = Vec::new();
                            res.push(("x".into(), p_num!(I64)));
                            res
                        },
                    },
                ));
                res
            },
        };
        let statement = Statement::parse(
            r##"
            let x = {
                let geo = Geo::Point {
                    x : 420,
                    y: 69,
                };
                let z = 27;
                return match geo {
                    case Geo::Point {x,y} => x,
                    case Geo::Axe {x} => z,
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
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }

    #[test]
    fn valid_match_enum() {
        let user_type = user_type_impl::Enum {
            id: "Geo".into(),
            values: vec!["Point".into(), "Axe".into(), "Other".into()],
        };
        let statement = Statement::parse(
            r##"
            let x = {
                let geo = Geo::Point;
                let z = 27;
                return match geo {
                    case Geo::Point => 420,
                    case Geo::Axe => 69,
                    case Geo::Other => 69,
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
            .register_type(&"Geo".into(), UserType::Enum(user_type))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }

    #[test]
    fn valid_match_enum_else() {
        let user_type = user_type_impl::Enum {
            id: "Geo".into(),
            values: vec!["Point".into(), "Axe".into(), "Other".into()],
        };
        let statement = Statement::parse(
            r##"
            let x = {
                let geo = Geo::Other;
                let z = 27;
                return match geo {
                    case Geo::Point => 420,
                    case Geo::Axe => 420,
                    else => 69,
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
            .register_type(&"Geo".into(), UserType::Enum(user_type))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 69));
    }

    #[test]
    fn valid_match_union_else() {
        let user_type = user_type_impl::Union {
            id: "Geo".into(),
            variants: {
                let mut res = Vec::new();
                res.push((
                    "Point".into(),
                    user_type_impl::Struct {
                        id: "Point".into(),
                        fields: vec![("x".into(), p_num!(I64)), ("y".into(), p_num!(I64))],
                    },
                ));
                res.push((
                    "Axe".into(),
                    user_type_impl::Struct {
                        id: "Axe".into(),
                        fields: {
                            let mut res = Vec::new();
                            res.push(("x".into(), p_num!(I64)));
                            res
                        },
                    },
                ));
                res
            },
        };
        let statement = Statement::parse(
            r##"
            let x = {
                let geo = Geo::Point {
                    x : 420,
                    y: 69,
                };
                let z = 27;
                return match geo {
                    case Geo::Axe {x} => x,
                    else => z,
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
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 27));
    }

    #[test]
    fn valid_match_number() {
        let statement = Statement::parse(
            r##"
            let x = match 69 {
                case 69 => 420,
                else => 69
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
    fn valid_match_number_else() {
        let statement = Statement::parse(
            r##"
            let x = match 420 {
                case 69 => 420,
                else => 69
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
        assert_eq!(result, v_num!(I64, 69));
    }

    #[test]
    fn valid_match_string() {
        let statement = Statement::parse(
            r##"
            let x = match "Hello world" {
                case "Hello world" => 420,
                else => 69
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
    fn valid_match_string_else() {
        let statement = Statement::parse(
            r##"
            let x = match "CipherPool" {
                case "Hello world" => 420,
                else => 69
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
        assert_eq!(result, v_num!(I64, 69));
    }

    #[test]
    fn valid_match_multiple_case_strslice() {
        let statement = Statement::parse(
            r##"
            let x = match "CipherPool" {
                case "Hello world" | "CipherPool" => 420,
                else => 69
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
    fn valid_match_multiple_case_num() {
        let statement = Statement::parse(
            r##"
            let x = match 500 {
                case 86 | 500 => 420,
                else => 69
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
    fn valid_match_union_mult() {
        let user_type = user_type_impl::Union {
            id: "Geo".into(),
            variants: {
                let mut res = Vec::new();
                res.push((
                    "Point".into(),
                    user_type_impl::Struct {
                        id: "Point".into(),
                        fields: vec![("x".into(), p_num!(I64))],
                    },
                ));
                res.push((
                    "Axe".into(),
                    user_type_impl::Struct {
                        id: "Axe".into(),
                        fields: vec![("x".into(), p_num!(I64))],
                    },
                ));
                res.push((
                    "Other".into(),
                    user_type_impl::Struct {
                        id: "Axe".into(),
                        fields: vec![("x".into(), p_num!(I64))],
                    },
                ));
                res
            },
        };
        let statement = Statement::parse(
            r##"
            let x = {
                let geo = Geo::Point {
                    x : 420,
                };
                let z = 27;
                return match geo {
                    case Geo::Axe {x} | Geo::Point {x} => x,
                    else => z,
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
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }

    #[test]
    fn valid_match_enum_mult() {
        let user_type = user_type_impl::Enum {
            id: "Geo".into(),
            values: vec!["Point".into(), "Axe".into(), "Other".into()],
        };
        let statement = Statement::parse(
            r##"
            let x = {
                let geo = Geo::Axe;
                let z = 27;
                return match geo {
                    case Geo::Point | Geo::Axe => 420,
                    else => 69,
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
            .register_type(&"Geo".into(), UserType::Enum(user_type))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }
}
