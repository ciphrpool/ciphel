use crate::{
    semantic::scope::scope::ScopeManager,
    vm::{
        casm::{branch::BranchTry, data::Data, mem::Mem},
        vm::CodeGenerationContext,
    },
};
use ulid::Ulid;

use crate::{
    ast::{
        expressions::{
            data::{Number, Primitive},
            Expression,
        },
        statements::block::Block,
    },
    semantic::{
        scope::{
            static_types::StaticType,
            user_type_impl::{Enum, Union, UserType},
        },
        EType, SizeOf,
    },
    vm::{
        casm::{
            branch::{BranchIf, BranchTable, Goto, Label},
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{CallStat, Flow, IfStat, MatchStat, TryStat};

impl GenerateCode for Flow {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Flow::If(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Flow::Match(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Flow::Try(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Flow::Call(value) => value.gencode(scope_manager, scope_id, instructions, context),
        }
    }
}

impl GenerateCode for CallStat {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let _ = self
            .call
            .gencode(scope_manager, scope_id, instructions, context)?;
        let Some(return_type) = self.call.signature() else {
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
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let mut else_label = Label::gen();
        let mut end_label = Label::gen();

        let _ = self
            .condition
            .gencode(scope_manager, scope_id, instructions, context)?;

        instructions.push(Casm::If(BranchIf { else_label }));
        let _ = self
            .then_branch
            .gencode(scope_manager, scope_id, instructions, context)?;
        instructions.push(Casm::Goto(Goto {
            label: Some(end_label),
        }));

        for (condition, block) in &self.else_if_branches {
            instructions.push_label_id(else_label, "else_if".to_string().into());

            else_label = Label::gen();

            let _ = condition.gencode(scope_manager, scope_id, instructions, context)?;

            instructions.push(Casm::If(BranchIf { else_label }));

            let _ = block.gencode(scope_manager, scope_id, instructions, context)?;
            instructions.push(Casm::Goto(Goto {
                label: Some(end_label),
            }));
        }

        instructions.push_label_id(else_label, "else".to_string().into());
        if let Some(block) = &self.else_branch {
            let _ = block.gencode(scope_manager, scope_id, instructions, context)?;
            instructions.push(Casm::Goto(Goto {
                label: Some(end_label),
            }));
        }

        instructions.push_label_id(end_label, "end_if".to_string().into());
        Ok(())
    }
}

impl GenerateCode for MatchStat {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let break_label = Label::gen();
        instructions.push_label("start_match".to_string());
        let _ = self
            .expr
            .gencode(scope_manager, scope_id, instructions, context)?;

        match &self.cases {
            crate::ast::expressions::flows::Cases::Primitive { cases } => {
                for case in cases {
                    case.gencode(
                        scope_manager,
                        scope_id,
                        instructions,
                        &CodeGenerationContext {
                            return_label: None,
                            break_label: Some(break_label),
                            continue_label: None,
                        },
                    )?;
                }
            }
            crate::ast::expressions::flows::Cases::String { cases } => {
                for case in cases {
                    case.gencode(
                        scope_manager,
                        scope_id,
                        instructions,
                        &CodeGenerationContext {
                            return_label: None,
                            break_label: Some(break_label),
                            continue_label: None,
                        },
                    )?;
                }
            }
            crate::ast::expressions::flows::Cases::Enum { cases } => {
                for case in cases {
                    case.gencode(
                        scope_manager,
                        scope_id,
                        instructions,
                        &CodeGenerationContext {
                            return_label: None,
                            break_label: Some(break_label),
                            continue_label: None,
                        },
                    )?;
                }
            }
            crate::ast::expressions::flows::Cases::Union { cases } => {
                for case in cases {
                    case.gencode(
                        scope_manager,
                        scope_id,
                        instructions,
                        &CodeGenerationContext {
                            return_label: None,
                            break_label: Some(break_label),
                            continue_label: None,
                        },
                    )?;
                }
            }
        }

        if let Some(block) = &self.else_branch {
            block.gencode(scope_manager, scope_id, instructions, context)?;
        }

        instructions.push_label_id(break_label, "end_match".to_string());

        Ok(())
    }
}

impl GenerateCode for TryStat {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let else_label = Label::gen();
        let end_try_label = Label::gen();
        let recover_else_label = Label::gen();

        instructions.push(Casm::Try(BranchTry::StartTry {
            else_label: recover_else_label,
        }));

        let _ = self
            .try_branch
            .gencode(scope_manager, scope_id, instructions, context)?;

        instructions.push(Casm::Goto(Goto {
            label: Some(end_try_label),
        }));

        instructions.push_label_id(recover_else_label, "recover_else".to_string().into());

        instructions.push_label_id(else_label, "else".to_string().into());
        instructions.push(Casm::Try(BranchTry::EndTry));

        if let Some(block) = &self.else_branch {
            block.gencode(scope_manager, scope_id, instructions, context)?;
        }

        instructions.push_label_id(end_try_label, "end_try".to_string().into());
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::ast::TryParse;
    use crate::semantic::scope::static_types::{NumberType, PrimitiveType};
    use crate::semantic::Resolve;
    use crate::vm::vm::DeserializeFrom;
    use crate::{ast::statements::Statement, semantic::scope::scope::ScopeManager};
    use crate::{compile_statement, v_num};

    #[test]
    fn valid_if() {
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
    fn robustness_if_else() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let var = 1;
            if var == 1 {
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
        assert_eq!(result, v_num!(I64, 420));
    }
    #[test]
    fn valid_match_primitive() {
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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
        let mut statement = Statement::parse(
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

    #[test]
    fn valid_try_catch_err() {
        let mut statement = Statement::parse(
            r##"
        let x = {

            let arr = vec[1,2,3,4];

            try {
                let x = arr[7];
            }

            return 1;
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
        assert_eq!(result, v_num!(I64, 1));
    }

    #[test]
    fn valid_try_catch_err_with_else() {
        let mut statement = Statement::parse(
            r##"
        let x = {

            let arr = vec[1,2,3,4];

            let res = 1;

            try {
                let x = arr[7];
            } else {
                res = 2;
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
        assert_eq!(result, v_num!(I64, 2));
    }

    #[test]
    fn valid_try_catch_no_err() {
        let mut statement = Statement::parse(
            r##"
        let x = {

            let arr = vec[1,2,3,4];

            let res = 1;

            try {
                let x = arr[2];
                res = 2;
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
        assert_eq!(result, v_num!(I64, 2));
    }

    #[test]
    fn valid_try_catch_no_err_with_else() {
        let mut statement = Statement::parse(
            r##"
        let x = {

            let arr = vec[1,2,3,4];

            let res = 1;

            try {
                let x = arr[2];
                res = 2;
            } else {
                res = 3;
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
        assert_eq!(result, v_num!(I64, 2));
    }

    #[test]
    fn valid_try_with_inner_try_catch_err() {
        let mut statement = Statement::parse(
            r##"
        let x = {

            let arr = vec[1,2,3,4];

            let res = 1;

            try {
                let x = try arr[7] else 2;
                res = x;
            } else {
                res = 3;
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
        assert_eq!(result, v_num!(I64, 2));
    }

    #[test]
    fn valid_try_with_inner_try_catch_no_err() {
        let mut statement = Statement::parse(
            r##"
        let x = {

            let arr = vec[1,2,3,4];

            let res = 1;

            try {
                let x = try arr[3] else 2;
                res = x;
            } else {
                res = 3;
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
        assert_eq!(result, v_num!(I64, 4));
    }

    #[test]
    fn valid_try_catch_err_with_inner_try_catch_no_err() {
        let mut statement = Statement::parse(
            r##"
        let x = {

            let arr = vec[1,2,3,4];

            let res = 1;

            try {
                let x = try arr[3] else 2;
                res = arr[7];
            } else {
                res = 3;
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
        assert_eq!(result, v_num!(I64, 3));
    }

    #[test]
    fn valid_try_early_return() {
        let mut statement = Statement::parse(
            r##"
        let x = {

            let arr = vec[1,2,3,4];

            let res = 1;

            try {
                return 7;
            } else {
                res = 3;
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
        assert_eq!(result, v_num!(I64, 7));
    }

    #[test]
    fn valid_try_early_return_with_inner_try_catch_err() {
        let mut statement = Statement::parse(
            r##"
        let x = {

            let arr = vec[1,2,3,4];

            let res = 1;

            try {
                try {
                    if arr[7] == 2 {
                        return 8;
                    }
                } else {
                    return 7;
                }
            } else {
                res = 3;
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
        assert_eq!(result, v_num!(I64, 7));
    }
    #[test]
    fn valid_try_early_return_with_inner_try_catch_no_err() {
        let mut statement = Statement::parse(
            r##"
        let x = {

            let arr = vec[1,2,3,4];

            let res = 1;

            try {
                try {
                    res = 2;
                } else {
                    if arr[7] == 2 {
                        return 8;
                    }
                }
            } else {
                res = 3;
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
        assert_eq!(result, v_num!(I64, 2));
    }
}
