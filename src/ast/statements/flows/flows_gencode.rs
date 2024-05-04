use crate::{semantic::scope::scope::Scope, vm::casm::data::Data};
use ulid::Ulid;

use crate::{
    ast::{
        expressions::{
            data::{Number, Primitive},
            flows::Pattern,
            Expression,
        },
        statements::block::{block_gencode::inner_block_gencode, Block},
    },
    semantic::{
        scope::{
            static_types::StaticType,
            user_type_impl::{Enum, Union, UserType},
            var_impl::VarState,
        },
        Either, MutRc, SizeOf,
    },
    vm::{
        casm::{
            branch::{BranchIf, BranchTable, Goto, Label},
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{CallStat, Flow, IfStat, MatchStat, PatternStat, TryStat};

impl GenerateCode for Flow {
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

impl GenerateCode for CallStat {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let _ = self.call.gencode(scope, instructions)?;
        let Some(return_type) = self.call.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let size = return_type.size_of();

        if size != 0 {
            instructions.push(Casm::Pop(size));
        }
        Ok(())
    }
}

impl GenerateCode for IfStat {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
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
        // let _ = self.then_branch.gencode(block, &instructions)?;
        let _ = inner_block_gencode(scope, &self.then_branch, None, false, instructions)?;

        for pair in self
            .else_if_branches
            .iter()
            .zip(&else_if_labels)
            .collect::<Vec<(&(Expression, Block), &Ulid)>>()
            .windows(2)
        {
            let ((cond_1, scope_1), label_1) = &pair[0];
            let ((_, _), label_2) = &pair[1];
            instructions.push_label_id(**label_1, "else_if".into());
            let _ = cond_1.gencode(scope, &instructions)?;
            instructions.push(Casm::If(BranchIf {
                else_label: **label_2,
            }));
            // let _ = scope_1.gencode(block, instructions)?;
            let _ = inner_block_gencode(scope, &scope_1, None, false, instructions)?;
        }
        if let Some((cond, s)) = &self.else_if_branches.last() {
            instructions.push_label_id(*else_if_labels.last().unwrap(), "else_if".into());
            let _ = cond.gencode(scope, &instructions)?;
            instructions.push(Casm::If(BranchIf {
                else_label: else_label.unwrap_or(end_if_label),
            }));
            // let _ = s.gencode(block, instructions)?;
            let _ = inner_block_gencode(scope, &s, None, false, instructions)?;
        }

        if let Some(s) = &self.else_branch {
            instructions.push_label_id(else_label.unwrap(), "else".into());
            let _ = inner_block_gencode(scope, &s, None, false, instructions)?;
        }

        instructions.push_label_id(end_if_label, "end_if".into());
        Ok(())
    }
}

impl GenerateCode for MatchStat {
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

        let switch_size = match &expr_type {
            Either::User(value) => match value.as_ref() {
                UserType::Enum(_) | UserType::Union(_) => 8,
                _ => expr_type.size_of(),
            },
            _ => expr_type.size_of(),
        };

        for PatternStat { patterns, .. } in &self.patterns {
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
        for (
            idx,
            (
                PatternStat {
                    patterns: _,
                    scope: s,
                },
                label,
            ),
        ) in self.patterns.iter().zip(cases).enumerate()
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

            let _ = inner_block_gencode(scope, &s, Some(param_size), false, instructions)?;
            instructions.push(Casm::Goto(Goto {
                label: Some(end_match_label),
            }));
        }
        match &self.else_branch {
            Some(else_branch) => {
                instructions.push_label_id(else_label.unwrap(), "else_case".into());

                let _ = inner_block_gencode(scope, &else_branch, None, false, instructions)?;
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

impl GenerateCode for TryStat {
    fn gencode(
        &self,
        _scope: &MutRc<Scope>,
        _instructions: &CasmProgram,
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
    use crate::{ast::statements::Statement, semantic::scope::scope::Scope};
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 69));
    }

    #[test]
    fn valid_match_union_mult() {
        let statement = Statement::parse(
            r##"
        let x = {
            union Sport {
                Foot{x:i64},
                Volley{x:i64},
                Basket{}
            }
            let var = Sport::Volley{x:420};
            let res = 0;
            match var {
                case Sport::Foot{x} | Sport::Volley{x} => {
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

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }
}
