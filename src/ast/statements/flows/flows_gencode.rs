use std::{cell::RefCell, rc::Rc};

use ulid::Ulid;

use crate::{
    ast::{
        expressions::{
            data::{Number, Primitive},
            flows::Pattern,
            Expression,
        },
        statements::scope::{scope_gencode::inner_scope_gencode, Scope},
    },
    semantic::{
        scope::{
            static_types::StaticType,
            user_type_impl::{Enum, Union, UserType},
            var_impl::VarState,
            ScopeApi,
        },
        Either, MutRc, SizeOf,
    },
    vm::{
        casm::{
            alloc::StackFrame,
            branch::{BranchIf, BranchTable, BranchTableExprInfo, Call, Goto, Label},
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{CallStat, Flow, IfStat, MatchStat, PatternStat, TryStat};

impl<Scope: ScopeApi> GenerateCode<Scope> for Flow<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Flow::If(value) => value.gencode(scope, instructions),
            Flow::Match(value) => value.gencode(scope, instructions),
            Flow::Try(value) => value.gencode(scope, instructions),
            Flow::Call(value) => value.gencode(scope, instructions),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for CallStat<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        self.call.gencode(scope, instructions)
    }
}

impl<InnerScope: ScopeApi> GenerateCode<InnerScope> for IfStat<InnerScope> {
    fn gencode(
        &self,
        scope: &MutRc<InnerScope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let mut else_if_labels: Vec<Ulid> = Vec::default();
        let else_label = match &self.else_branch {
            Some(_) => Some(Label::gen()),
            None => None,
        };
        let end_if_label = Label::gen();

        for (_, _) in &self.else_if_branches {
            else_if_labels.push(Label::gen());
        }

        let _ = self.condition.gencode(scope, &instructions)?;

        match &self.else_if_branches.first() {
            Some(_) => {
                instructions.push(Casm::If(BranchIf {
                    else_label: *else_if_labels.first().unwrap_or(&end_if_label),
                }));
            }
            None => {
                instructions.push(Casm::If(BranchIf {
                    else_label: else_label.unwrap_or(end_if_label),
                }));
            }
        }
        // let _ = self.then_branch.gencode(scope, &instructions)?;
        let _ = inner_scope_gencode(scope, &self.then_branch, None, false, instructions)?;

        for pair in self
            .else_if_branches
            .iter()
            .zip(&else_if_labels)
            .collect::<Vec<(&(Expression<InnerScope>, Scope<InnerScope>), &Ulid)>>()
            .windows(2)
        {
            let ((cond_1, scope_1), label_1) = &pair[0];
            let ((_, _), label_2) = &pair[1];
            instructions.push_label_id(**label_1, "else_if".into());
            let _ = cond_1.gencode(scope, &instructions)?;
            instructions.push(Casm::If(BranchIf {
                else_label: **label_2,
            }));
            // let _ = scope_1.gencode(scope, instructions)?;
            let _ = inner_scope_gencode(scope, &scope_1, None, false, instructions)?;
        }
        if let Some((cond, s)) = &self.else_if_branches.last() {
            instructions.push_label_id(*else_if_labels.last().unwrap(), "else_if".into());
            let _ = cond.gencode(scope, &instructions)?;
            instructions.push(Casm::If(BranchIf {
                else_label: else_label.unwrap_or(end_if_label),
            }));
            // let _ = s.gencode(scope, instructions)?;
            let _ = inner_scope_gencode(scope, &s, None, false, instructions)?;
        }

        if let Some(s) = &self.else_branch {
            instructions.push_label_id(else_label.unwrap(), "else".into());
            let _ = inner_scope_gencode(scope, &s, None, false, instructions)?;
        }

        instructions.push_label_id(end_if_label, "end_if".into());
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for MatchStat<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
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

        for PatternStat { pattern, .. } in &self.patterns {
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
                    let data: Vec<u8> = value.value.as_bytes().to_vec();
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
        for (idx, (PatternStat { pattern, scope: s }, label)) in
            self.patterns.iter().zip(cases).enumerate()
        {
            instructions.push_label_id(label, format!("match_case_{}", idx).into());
            let param_size = s
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

            let _ = inner_scope_gencode(scope, &s, Some(param_size), false, instructions)?;
            instructions.push(Casm::Goto(Goto {
                label: Some(end_match_label),
            }));
        }
        match &self.else_branch {
            Some(else_branch) => {
                instructions.push_label_id(else_label.unwrap(), "else_case".into());

                let _ = inner_scope_gencode(scope, &else_branch, None, false, instructions)?;
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

impl<Scope: ScopeApi> GenerateCode<Scope> for TryStat<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    use crate::ast::expressions::data::{Number, Primitive};
    use crate::ast::TryParse;
    use crate::semantic::scope::static_types::{NumberType, PrimitiveType};
    use crate::semantic::Resolve;
    use crate::vm::vm::{DeserializeFrom, Runtime};
    use crate::{ast::statements::Statement, semantic::scope::scope_impl::Scope};
    use crate::{clear_stack, compile_statement, v_num};

    #[test]
    fn valid_if() {
        let statement = Statement::parse(
            r##"
        let x = {
            let var = 0;
            if var == 0 {
                var = 420;
            }

            return var;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }

    #[test]
    fn valid_if_else_if() {
        let statement = Statement::parse(
            r##"
        let x = {
            let var = 1;
            if var == 0 {
                var = 420;
            } else if var == 1 {
                var = 69;
            }

            return var;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 69));
    }

    #[test]
    fn valid_if_else() {
        let statement = Statement::parse(
            r##"
        let x = {
            let var = 1;
            if var == 0 {
                var = 420;
            } else {
                var = 69;
            }

            return var;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 69));
    }

    #[test]
    fn valid_match_primitive() {
        let statement = Statement::parse(
            r##"
        let x = {
            let var = 1;

            match var {
                case 2 => {
                    var = 420;
                }
                case 1 => {
                    var = 420;
                }
                else => {
                    var = 69;
                }
            }

            return var;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }
    #[test]
    fn valid_match_primitive_else() {
        let statement = Statement::parse(
            r##"
        let x = {
            let var = 3;

            match var {
                case 1 => {
                    var = 420;
                }
                else => {
                    var = 69;
                }
            }

            return var;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 69));
    }

    #[test]
    fn valid_match_strslice() {
        let statement = Statement::parse(
            r##"
        let x = {
            let var = "Hello";
            let res = 0;
            match var {
                case "Hello" => {
                    res = 420;
                }
                else => {
                    res = 69;
                }
            }
            return res;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }

    #[test]
    fn valid_match_strslice_other() {
        let statement = Statement::parse(
            r##"
        let x = {
            let var = "Hello";
            let res = 0;
            match var {
                case "World" => {
                    res = 69;
                }
                case "Hello" => {
                    res = 420;
                }
                else => {
                    res = 69;
                }
            }
            return res;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }

    #[test]
    fn valid_match_strslice_else() {
        let statement = Statement::parse(
            r##"
        let x = {
            let var = "World";
            let res = 0;
            match var {
                case "Hello" => {
                    res = 420;
                }
                else => {
                    res = 69;
                }
            }
            return res;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 69));
    }

    #[test]
    fn valid_match_enum() {
        let statement = Statement::parse(
            r##"
        let x = {
            enum Sport {
                Foot,
                Volley,
                Basket
            }
            let var = Sport::Volley;
            let res = 0;
            match var {
                case Sport::Foot => {
                    res = 69;
                }
                case Sport::Volley => {
                    res = 420;
                }
                else => {
                    res = 69;
                }
            }
            return res;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }

    #[test]
    fn valid_match_enum_else() {
        let statement = Statement::parse(
            r##"
        let x = {
            enum Sport {
                Foot,
                Volley,
                Basket
            }
            let var = Sport::Volley;
            let res = 0;
            match var {
                case Sport::Foot => {
                    res = 420;
                }
                else => {
                    res = 69;
                }
            }
            return res;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 69));
    }

    #[test]
    fn valid_match_union() {
        let statement = Statement::parse(
            r##"
        let x = {
            union Sport {
                Foot{x:i64},
                Basket{}
            }
            let var = Sport::Foot{x:420};
            let res = 0;
            match var {
                case Sport::Foot{x} => {
                    res = x;
                }
                else => {
                    res = 69;
                }
            }
            return res;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }

    #[test]
    fn valid_match_union_else() {
        let statement = Statement::parse(
            r##"
        let x = {
            union Sport {
                Foot{x:i64},
                Basket{}
            }
            let var = Sport::Basket{};
            let res = 0;
            match var {
                case Sport::Foot{x} => {
                    res = 420;
                }
                else => {
                    res = 69;
                }
            }
            return res;
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 69));
    }
}
