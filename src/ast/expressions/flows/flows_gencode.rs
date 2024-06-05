use num_traits::ToBytes;

use ulid::Ulid;

use crate::semantic::scope::scope::Scope;
use crate::vm::allocator::stack::UReg;
use crate::vm::casm::alloc::StackFrame;
use crate::vm::casm::branch::BranchTry;
use crate::vm::casm::data;
use crate::vm::platform::stdlib::strings::{JoinCasm, StringsCasm, ToStrCasm};
use crate::vm::platform::stdlib::StdCasm;
use crate::vm::platform::LibCasm;
use crate::{arw_read, e_static};
use crate::{
    ast::expressions::data::{Number, Primitive},
    semantic::{
        scope::{
            static_types::{ClosureType, NumberType, StaticType},
            user_type_impl::{Enum, Union, UserType},
            var_impl::VarState,
        },
        ArcMutex, Either, SizeOf, TypeOf,
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

use super::{ExprFlow, FCall, IfExpr, MatchExpr, Pattern, PatternExpr, TryExpr};

impl GenerateCode for ExprFlow {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            ExprFlow::If(value) => value.gencode(scope, instructions),
            ExprFlow::Match(value) => value.gencode(scope, instructions),
            ExprFlow::Try(value) => value.gencode(scope, instructions),
            ExprFlow::SizeOf(value, _metadata) => {
                let value = value
                    .type_of(&crate::arw_read!(
                        scope,
                        CodeGenerationError::ConcurrencyError
                    )?)
                    .map_err(|_| CodeGenerationError::UnresolvedError)?;

                instructions.push(Casm::Data(Data::Serialized {
                    data: (value.size_of() as u64).to_le_bytes().into(),
                }));
                Ok(())
            }
            ExprFlow::FCall(value) => value.gencode(scope, instructions),
        }
    }
}

impl GenerateCode for IfExpr {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
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

        let _if_label = instructions.push_label("if".to_string().into());

        let _ = self.condition.gencode(scope, instructions)?;

        instructions.push(Casm::If(BranchIf { else_label }));
        instructions.push(Casm::Goto(Goto {
            label: Some(end_if_scope_label),
        }));
        instructions.push_label_id(if_scope_label, "if_scope".to_string().into());

        let _ = self.then_branch.gencode(scope, instructions)?;

        instructions.push_label_id(end_if_scope_label, "end_if_scope".to_string().into());
        instructions.push(Casm::Call(Call::From {
            label: if_scope_label,
            param_size: 0,
        }));
        instructions.push(Casm::Pop(9)); /* Pop the unused return size and return flag */
        instructions.push(Casm::Goto(Goto {
            label: Some(end_ifelse_label),
        }));

        instructions.push_label_id(else_label, "else".to_string().into());
        instructions.push(Casm::Goto(Goto {
            label: Some(end_else_scope_label),
        }));
        instructions.push_label_id(else_scope_label, "else_scope".to_string().into());

        let _ = self.else_branch.gencode(scope, instructions)?;

        instructions.push_label_id(end_else_scope_label, "end_else_scope".to_string().into());
        instructions.push(Casm::Call(Call::From {
            label: else_scope_label,
            param_size: 0,
        }));
        instructions.push(Casm::Pop(9)); /* Pop the unused return size and return flag */
        instructions.push(Casm::Goto(Goto {
            label: Some(end_ifelse_label),
        }));

        instructions.push_label_id(end_ifelse_label, "end_if_else".to_string().into());

        Ok(())
    }
}

impl GenerateCode for MatchExpr {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
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
        let _match_label = instructions.push_label("Match".to_string().into());

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
                        let mut data: Vec<u8> = value.value.as_bytes().to_vec();
                        data.extend_from_slice(&(data.len() as u64).to_le_bytes());
                        dump_data.push(data.into());
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
            let scope_label = instructions.push_label("Scope".to_string().into());
            let _ = expr.gencode(scope, instructions)?;

            // let param_size = expr
            //     .parameters_size()
            //     .map_err(|_| CodeGenerationError::UnresolvedError)?;
            let scope = expr
                .scope()
                .map_err(|_| CodeGenerationError::UnresolvedError)?;
            let borrowed_scope = arw_read!(scope, CodeGenerationError::ConcurrencyError)?;
            let param_size = borrowed_scope
                .vars()
                .filter_map(|(v, _)| {
                    (v.state.get() == VarState::Parameter).then(|| v.type_sig.size_of())
                })
                .sum::<usize>();
            instructions.push_label_id(end_scope_label, "End_Scope".to_string().into());
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
                instructions.push_label_id(else_label.unwrap(), "else_case".to_string().into());
                let end_scope_label = Label::gen();
                instructions.push(Casm::Goto(Goto {
                    label: Some(end_scope_label),
                }));
                let scope_label = instructions.push_label("Scope".to_string().into());
                let _ = else_branch.gencode(scope, instructions)?;

                instructions.push_label_id(end_scope_label, "End_Scope".to_string().into());
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

        instructions.push_label_id(end_match_label, "end_match_else".to_string().into());
        Ok(())
    }
}

impl GenerateCode for TryExpr {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(return_size) = self.metadata.signature().map(|t| t.size_of()) else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let else_label = Label::gen();
        let try_scope_label = Label::gen();
        let end_try_scope_label = Label::gen();
        let else_scope_label = Label::gen();
        let recover_else_label = Label::gen();
        let end_else_scope_label = Label::gen();
        let end_tryelse_label = Label::gen();
        let end_try_label = Label::gen();

        let try_label = instructions.push_label("try".to_string().into());
        // instructions.push(Casm::Mem(Mem::GetReg(UReg::R2)));
        // instructions.push(Casm::Mem(Mem::LabelOffset(else_label)));
        instructions.push(Casm::Try(BranchTry::StartTry {
            else_label: recover_else_label,
        }));
        instructions.push(Casm::Goto(Goto {
            label: Some(end_try_scope_label),
        }));
        instructions.push_label_id(try_scope_label, "try_scope".to_string().into());

        let _ = self.try_branch.gencode(scope, instructions)?;

        instructions.push_label_id(end_try_scope_label, "end_try_scope".to_string().into());
        instructions.push(Casm::Call(Call::From {
            label: try_scope_label,
            param_size: 0,
        }));
        instructions.push(Casm::Pop(9)); /* Pop the unused return size and return flag */

        if self.pop_last_err.get() {
            /* Pop the error */
            instructions.push(Casm::If(BranchIf {
                else_label: end_try_label,
            }));
            instructions.push(Casm::Pop(return_size)); // discard error value
            instructions.push(Casm::Goto(Goto {
                label: Some(else_label),
            }))
        }
        instructions.push_label_id(end_try_label, "end_try".to_string().into());
        instructions.push(Casm::Try(BranchTry::EndTry));

        instructions.push(Casm::Goto(Goto {
            label: Some(end_tryelse_label),
        }));
        instructions.push_label_id(recover_else_label, "recover_else".to_string().into());
        instructions.push(Casm::StackFrame(StackFrame::SoftClean));

        instructions.push_label_id(else_label, "else".to_string().into());
        instructions.push(Casm::Try(BranchTry::EndTry));
        instructions.push(Casm::Goto(Goto {
            label: Some(end_else_scope_label),
        }));
        instructions.push_label_id(else_scope_label, "else_scope".to_string().into());

        match &self.else_branch {
            Some(else_branch) => {
                let _ = else_branch.gencode(scope, instructions)?;
                instructions
                    .push_label_id(end_else_scope_label, "end_else_scope".to_string().into());
                instructions.push(Casm::Call(Call::From {
                    label: else_scope_label,
                    param_size: 0,
                }));
                instructions.push(Casm::Pop(9)); /* Pop the unused return size and return flag */
                instructions.push(Casm::Goto(Goto {
                    label: Some(end_tryelse_label),
                }));
            }
            None => {
                instructions
                    .push_label_id(end_else_scope_label, "end_else_scope".to_string().into());
                instructions.push(Casm::Goto(Goto {
                    label: Some(end_tryelse_label),
                }));
            }
        }

        instructions.push_label_id(end_tryelse_label, "end_try_else".to_string().into());
        Ok(())
    }
}
impl GenerateCode for FCall {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        for item in &self.value {
            match item {
                super::FormatItem::Str(string) => {
                    let str_bytes: Box<[u8]> = string.as_bytes().into();
                    let size = (&str_bytes).len() as u64;
                    instructions.push(Casm::Data(data::Data::Serialized { data: str_bytes }));
                    instructions.push(Casm::Data(data::Data::Serialized {
                        data: size.to_le_bytes().into(),
                    }));
                    instructions.push(Casm::Platform(LibCasm::Std(StdCasm::Strings(
                        StringsCasm::ToStr(ToStrCasm::ToStrStrSlice),
                    ))));
                }
                super::FormatItem::Expr(expr) => {
                    let _ = expr.gencode(scope, instructions)?;
                }
            }
        }
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::Strings(
            StringsCasm::Join(JoinCasm::NoSepFromSlice(Some(self.value.len()))),
        ))));
        Ok(())
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
                scope::Scope,
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
        let mut statement_then = IfExpr::parse(
            r##"
           if true then 420 else 69 
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = statement_then
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");
        // Code generation.
        let mut instructions_then = CasmProgram::default();
        statement_then
            .gencode(&scope, &mut instructions_then)
            .expect("Code generation should have succeeded");

        assert!(instructions_then.len() > 0);
        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions_then);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
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
    fn valid_if_basic_else() {
        let mut statement_else = IfExpr::parse(
            r##"
           if false then 420 else 69 
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = statement_else
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions_else = CasmProgram::default();
        statement_else
            .gencode(&scope, &mut instructions_else)
            .expect("Code generation should have succeeded");

        assert!(instructions_else.len() > 0);
        // Execute the instructions.
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions_else);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        let memory = stack;
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
        let mut statement_then = IfExpr::parse(
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

        let scope = Scope::new();
        let _ = statement_then
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions_then = CasmProgram::default();
        statement_then
            .gencode(&scope, &mut instructions_then)
            .expect("Code generation should have succeeded");

        assert!(instructions_then.len() > 0);
        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions_then);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
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
    fn valid_if_basic_scope_else() {
        let mut statement_else = IfExpr::parse(
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

        let _ = statement_else
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.

        let mut instructions_else = CasmProgram::default();
        statement_else
            .gencode(&scope, &mut instructions_else)
            .expect("Code generation should have succeeded");

        assert!(instructions_else.len() > 0);
        // Execute the instructions.
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions_else);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        let memory = stack;
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
            id: "Point".to_string().into(),
            fields: {
                let mut res = Vec::new();
                res.push(("x".to_string().into(), p_num!(I64)));
                res.push(("y".to_string().into(), p_num!(I64)));
                res
            },
        };
        let mut statement_then = IfExpr::parse(
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

        let scope = Scope::new();
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Point".to_string().into(),
                UserType::Struct(user_type.clone()),
            )
            .expect("Registering of user type should have succeeded");
        let _ = statement_then
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions_then = CasmProgram::default();
        statement_then
            .gencode(&scope, &mut instructions_then)
            .expect("Code generation should have succeeded");
        assert!(instructions_then.len() > 0);
        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions_then);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);

        let result: Struct = user_type
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");

        for (r_id, res) in &result.fields {
            match res {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x.get() {
                        Number::I64(res) => {
                            if **r_id == "x" {
                                assert_eq!(420, res);
                            } else if **r_id == "y" {
                                assert_eq!(420, res);
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
    fn valid_if_complex_else() {
        let user_type = user_type_impl::Struct {
            id: "Point".to_string().into(),
            fields: {
                let mut res = Vec::new();
                res.push(("x".to_string().into(), p_num!(I64)));
                res.push(("y".to_string().into(), p_num!(I64)));
                res
            },
        };
        let mut statement_else = IfExpr::parse(
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
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Point".to_string().into(),
                UserType::Struct(user_type.clone()),
            )
            .expect("Registering of user type should have succeeded");
        let _ = statement_else
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions_else = CasmProgram::default();
        statement_else
            .gencode(&scope, &mut instructions_else)
            .expect("Code generation should have succeeded");
        assert!(instructions_else.len() > 0);
        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions_else);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);

        let result: Struct = user_type
            .deserialize_from(&data)
            .expect("Deserialization should have succeeded");

        for (r_id, res) in &result.fields {
            match res {
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
                    match x.get() {
                        Number::I64(res) => {
                            if **r_id == "x" {
                                assert_eq!(69, res);
                            } else if **r_id == "y" {
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
        let mut statement_then = Statement::parse(
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
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions_then = CasmProgram::default();
        statement_then
            .gencode(&scope, &mut instructions_then)
            .expect("Code generation should have succeeded");

        assert!(instructions_then.len() > 0);
        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions_then);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
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
    fn valid_match_union() {
        let user_type = user_type_impl::Union {
            id: "Geo".to_string().into(),
            variants: {
                let mut res = Vec::new();
                res.push((
                    "Point".to_string().into(),
                    user_type_impl::Struct {
                        id: "Point".to_string().into(),
                        fields: vec![
                            ("x".to_string().into(), p_num!(I64)),
                            ("y".to_string().into(), p_num!(I64)),
                        ],
                    },
                ));
                res.push((
                    "Axe".to_string().into(),
                    user_type_impl::Struct {
                        id: "Axe".to_string().into(),
                        fields: {
                            let mut res = Vec::new();
                            res.push(("x".to_string().into(), p_num!(I64)));
                            res
                        },
                    },
                ));
                res
            },
        };
        let mut statement = Statement::parse(
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
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(&"Geo".to_string().into(), UserType::Union(user_type))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
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
            id: "Geo".to_string().into(),
            values: vec![
                "Point".to_string().into(),
                "Axe".to_string().into(),
                "Other".to_string().into(),
            ],
        };
        let mut statement = Statement::parse(
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
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(&"Geo".to_string().into(), UserType::Enum(user_type))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
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
            id: "Geo".to_string().into(),
            values: vec![
                "Point".to_string().into(),
                "Axe".to_string().into(),
                "Other".to_string().into(),
            ],
        };
        let mut statement = Statement::parse(
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
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(&"Geo".to_string().into(), UserType::Enum(user_type))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
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
            id: "Geo".to_string().into(),
            variants: {
                let mut res = Vec::new();
                res.push((
                    "Point".to_string().into(),
                    user_type_impl::Struct {
                        id: "Point".to_string().into(),
                        fields: vec![
                            ("x".to_string().into(), p_num!(I64)),
                            ("y".to_string().into(), p_num!(I64)),
                        ],
                    },
                ));
                res.push((
                    "Axe".to_string().into(),
                    user_type_impl::Struct {
                        id: "Axe".to_string().into(),
                        fields: {
                            let mut res = Vec::new();
                            res.push(("x".to_string().into(), p_num!(I64)));
                            res
                        },
                    },
                ));
                res
            },
        };
        let mut statement = Statement::parse(
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
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(&"Geo".to_string().into(), UserType::Union(user_type))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 27));
    }

    #[test]
    fn valid_match_number() {
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
            id: "Geo".to_string().into(),
            variants: {
                let mut res = Vec::new();
                res.push((
                    "Point".to_string().into(),
                    user_type_impl::Struct {
                        id: "Point".to_string().into(),
                        fields: vec![("x".to_string().into(), p_num!(I64))],
                    },
                ));
                res.push((
                    "Axe".to_string().into(),
                    user_type_impl::Struct {
                        id: "Axe".to_string().into(),
                        fields: vec![("x".to_string().into(), p_num!(I64))],
                    },
                ));
                res.push((
                    "Other".to_string().into(),
                    user_type_impl::Struct {
                        id: "Axe".to_string().into(),
                        fields: vec![("x".to_string().into(), p_num!(I64))],
                    },
                ));
                res
            },
        };
        let mut statement = Statement::parse(
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
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(&"Geo".to_string().into(), UserType::Union(user_type))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
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
            id: "Geo".to_string().into(),
            values: vec![
                "Point".to_string().into(),
                "Axe".to_string().into(),
                "Other".to_string().into(),
            ],
        };
        let mut statement = Statement::parse(
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
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(&"Geo".to_string().into(), UserType::Enum(user_type))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let mut instructions = CasmProgram::default();
        statement
            .gencode(&scope, &mut instructions)
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }

    #[test]
    fn valid_try_tuple() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let res = try (10,Ok()) else 20;
            return res;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 10));
    }
    #[test]
    fn valid_try_tuple_else() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let res = try (10,Err()) else 20;
            return res;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 20));
    }

    #[test]
    fn valid_try_tuple_catch_err_string_access() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let buf = string("aaaabbbbcc");
            let res = try buf[16] else 'a';
            return res;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result =
            <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
                .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Char('a'));
    }

    #[test]
    fn valid_try_tuple_catch_err_strslice_access() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let buf = "aaaabbbbcc";
            let res = try buf[16] else 'a';
            return res;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result =
            <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
                .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Char('a'));
    }

    #[test]
    fn valid_try_tuple_catch_err_slice_access() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let buf = [1,5,3,8];
            let res = try buf[16] else 10;
            return res;
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
        assert_eq!(result, v_num!(I64, 10));
    }

    #[test]
    fn valid_try_tuple_catch_err_vec_access() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let buf = vec[1,5,3,8];
            let res = try buf[16] else 10;
            return res;
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
        assert_eq!(result, v_num!(I64, 10));
    }
}
