use num_traits::ToBytes;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use ulid::Ulid;

use crate::{
    ast::expressions::data::{Number, Primitive, VarID, Variable},
    semantic::{
        scope::{
            static_types::{ClosureType, NumberType, StaticType, StrSliceType},
            user_type_impl::{Enum, Union, UserType},
            var_impl::VarState,
            ScopeApi,
        },
        AccessLevel, Either, MutRc, SizeOf,
    },
    vm::{
        allocator::{
            stack::{Offset, UReg},
            MemoryAddress,
        },
        casm::{
            alloc::{Access, StackFrame},
            branch::{BranchIf, BranchTable, BranchTableExprInfo, Call, Goto, Label},
            locate::Locate,
            memcopy::MemCopy,
            operation::{Addition, OpPrimitive, Operation, OperationKind, Substraction},
            serialize::Serialized,
            Casm, CasmProgram,
        },
        platform::{self, GenerateCodePlatform, Lib},
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{ExprFlow, FnCall, IfExpr, MatchExpr, Pattern, PatternExpr, TryExpr};

impl<Scope: ScopeApi> GenerateCode<Scope> for ExprFlow<Scope> {
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
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for IfExpr<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
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

        let if_label = instructions.push_label("If".into());

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

impl<Scope: ScopeApi> GenerateCode<Scope> for MatchExpr<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
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
                StaticType::StrSlice(_) => None,
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

        let end_match_label = Label::gen();
        let match_label = instructions.push_label("Match".into());

        let mut cases: Vec<Ulid> = Vec::with_capacity(self.patterns.len());
        let mut table: Vec<(u64, Ulid)> = Vec::with_capacity(self.patterns.len());
        let mut switch: Vec<(Vec<u8>, Ulid)> = Vec::with_capacity(self.patterns.len());

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
                        table.push((idx as u64, label));
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
                        table.push((idx as u64, label));
                    }
                }
                Pattern::Primitive(value) => {
                    let data = match value {
                        Primitive::Number(data) => match data.get() {
                            Number::U8(data) => data.to_le_bytes().to_vec(),
                            Number::U16(data) => data.to_le_bytes().to_vec(),
                            Number::U32(data) => data.to_le_bytes().to_vec(),
                            Number::U64(data) => data.to_le_bytes().to_vec(),
                            Number::U128(data) => data.to_le_bytes().to_vec(),
                            Number::I8(data) => data.to_le_bytes().to_vec(),
                            Number::I16(data) => data.to_le_bytes().to_vec(),
                            Number::I32(data) => data.to_le_bytes().to_vec(),
                            Number::I64(data) => data.to_le_bytes().to_vec(),
                            Number::I128(data) => data.to_le_bytes().to_vec(),
                            Number::F64(data) => data.to_le_bytes().to_vec(),
                            _ => return Err(CodeGenerationError::UnresolvedError),
                        },
                        Primitive::Bool(data) => [*data as u8].to_vec(),
                        Primitive::Char(data) => {
                            let mut buffer = [0u8; 4];
                            let _ = data.encode_utf8(&mut buffer);
                            buffer.to_vec()
                        }
                    };
                    switch.push((data, label));
                }
                Pattern::String(value) => {
                    let data: Vec<u8> = value
                        .value
                        .chars()
                        .flat_map(|c| {
                            let mut buffer = [0u8; 4];
                            c.encode_utf8(&mut buffer);
                            buffer
                        })
                        .collect();
                    switch.push((data, label));
                }
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

            let info = match &expr_type {
                Either::Static(ref value) => match value.as_ref() {
                    StaticType::Primitive(value) => BranchTableExprInfo::Primitive(value.size_of()),
                    StaticType::StrSlice(value) => BranchTableExprInfo::Primitive(value.size_of()),
                    StaticType::String(_) => BranchTableExprInfo::String,
                    _ => return Err(CodeGenerationError::UnresolvedError),
                },
                Either::User(_) => return Err(CodeGenerationError::UnresolvedError),
            };

            instructions.push(Casm::Switch(BranchTable::Swith {
                info,
                table: switch,
                else_label,
            }))
        } else {
            // Switch with branch table statement
            // extrart variant from matched expression

            instructions.push(Casm::Switch(BranchTable::Table { table, else_label }))
        }
        for (idx, (PatternExpr { pattern, expr }, label)) in
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

impl<Scope: ScopeApi> GenerateCode<Scope> for TryExpr<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for FnCall<Scope> {
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
            platform_api.gencode(scope, instructions, params_size)
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
            let return_size = signature.size_of();

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
                        instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
                    }
                    // Call function
                    // Load param size
                    instructions.push(Casm::Serialize(Serialized {
                        data: (sig_params_size as u64).to_le_bytes().to_vec(),
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
                            instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
                            // Load Env heap address
                            instructions.push(Casm::Serialize(Serialized {
                                data: (16u64).to_le_bytes().to_vec(),
                            }));
                            instructions.push(Casm::Operation(Operation {
                                kind: OperationKind::Addition(Addition {
                                    left: OpPrimitive::Number(NumberType::U64),
                                    right: OpPrimitive::Number(NumberType::U64),
                                }),
                            }));
                            instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
                            instructions.push(Casm::Serialize(Serialized {
                                data: (16u64).to_le_bytes().to_vec(),
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
                            instructions.push(Casm::Serialize(Serialized {
                                data: (16u64).to_le_bytes().to_vec(),
                            }));
                            instructions.push(Casm::Operation(Operation {
                                kind: OperationKind::Addition(Addition {
                                    left: OpPrimitive::Number(NumberType::U64),
                                    right: OpPrimitive::Number(NumberType::U64),
                                }),
                            }));
                            instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
                            instructions.push(Casm::Serialize(Serialized {
                                data: (16u64).to_le_bytes().to_vec(),
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
                    instructions.push(Casm::Serialize(Serialized {
                        data: (sig_params_size).to_le_bytes().to_vec(),
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

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(420))));

        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions_else);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(69))));
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

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(420))));

        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions_else);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(69))));
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

        let result: Struct<Scope> = user_type
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

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(420))));
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
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
                                        .into(),
                                ),
                            ),
                            (
                                "y".into(),
                                Either::Static(
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
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
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
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

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(420))));
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
                        fields: vec![
                            (
                                "x".into(),
                                Either::Static(
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
                                        .into(),
                                ),
                            ),
                            (
                                "y".into(),
                                Either::Static(
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
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
                                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
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

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(27))));
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

        let scope = Scope::new();
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

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(420))));
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

        let scope = Scope::new();
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

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(69))));
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

        let scope = Scope::new();
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

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(420))));
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

        let scope = Scope::new();
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

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(69))));
    }
}
