use crate::ast::statements::block::block_gencode::inner_block_gencode;
use crate::semantic::scope::scope_impl::Scope;
use crate::semantic::scope::type_traits::GetSubTypes;
use crate::semantic::SizeOf;
use crate::vm::casm::branch::BranchIf;

use crate::vm::vm::NextItem;
use crate::{
    semantic::MutRc,
    vm::{
        allocator::stack::UReg,
        casm::{
            alloc::StackFrame,
            branch::{Goto, Label},
            mem::Mem,
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{ForLoop, Loop, WhileLoop};

impl GenerateCode for Loop {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Loop::For(value) => value.gencode(scope, instructions),
            Loop::While(value) => value.gencode(scope, instructions),
            Loop::Loop(value) => {
                let start_label = Label::gen();
                let end_label = Label::gen();

                instructions.push(Casm::Mem(Mem::DumpRegisters));
                instructions.push(Casm::StackFrame(StackFrame::OpenWindow));
                instructions.push_label_id(start_label, "start_loop".into());
                instructions.push(Casm::Mem(Mem::LabelOffset(end_label)));
                instructions.push(Casm::Mem(Mem::SetReg(UReg::R4, None)));
                let _ = inner_block_gencode(scope, value, None, true, instructions)?;
                instructions.push(Casm::Goto(Goto {
                    label: Some(start_label),
                }));

                instructions.push_label_id(end_label, "end_loop".into());
                instructions.push(Casm::StackFrame(StackFrame::CloseWindow));
                instructions.push(Casm::Mem(Mem::RecoverRegisters));
                Ok(())
            }
        }
    }
}

impl GenerateCode for ForLoop {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(iterator_type) = self.iterator.expr.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let start_label = Label::gen();
        let next_label = Label::gen();
        let end_label = Label::gen();

        instructions.push(Casm::Mem(Mem::DumpRegisters));
        instructions.push(Casm::StackFrame(StackFrame::OpenWindow));

        let _ = self.iterator.expr.gencode(scope, instructions)?;

        /* init itertor index */
        let _ = iterator_type.init_address(instructions)?;
        let _ = iterator_type.init_index(instructions)?;

        instructions.push_label_id(start_label, "start_for".into());

        let _ = iterator_type.build_item(instructions, end_label)?;
        instructions.push(Casm::Mem(Mem::LabelOffset(end_label)));
        instructions.push(Casm::Mem(Mem::SetReg(UReg::R4, None)));

        instructions.push(Casm::Mem(Mem::LabelOffset(next_label)));
        instructions.push(Casm::Mem(Mem::SetReg(UReg::R3, None)));

        let Some(item_type) = iterator_type.get_item() else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let params_size = item_type.size_of();

        let _ = inner_block_gencode(scope, &self.scope, Some(params_size), true, instructions)?;

        instructions.push_label_id(next_label, "next_label".into());
        let _ = iterator_type.next(instructions)?;
        instructions.push(Casm::Goto(Goto {
            label: Some(start_label),
        }));

        instructions.push_label_id(end_label, "end_for".into());
        instructions.push(Casm::StackFrame(StackFrame::CloseWindow));
        instructions.push(Casm::Mem(Mem::RecoverRegisters));
        Ok(())
    }
}
// 1..10:-1
impl GenerateCode for WhileLoop {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let start_label = Label::gen();
        let end_label = Label::gen();

        instructions.push(Casm::Mem(Mem::DumpRegisters));
        instructions.push(Casm::StackFrame(StackFrame::OpenWindow));
        instructions.push_label_id(start_label, "start_while".into());
        instructions.push(Casm::Mem(Mem::LabelOffset(end_label)));
        instructions.push(Casm::Mem(Mem::SetReg(UReg::R4, None)));
        let _ = self.condition.gencode(scope, instructions)?;
        instructions.push(Casm::If(BranchIf {
            else_label: end_label,
        }));
        let _ = inner_block_gencode(scope, &self.scope, None, true, instructions)?;
        instructions.push(Casm::Goto(Goto {
            label: Some(start_label),
        }));

        instructions.push_label_id(end_label, "end_while".into());
        instructions.push(Casm::StackFrame(StackFrame::CloseWindow));
        instructions.push(Casm::Mem(Mem::RecoverRegisters));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::ast::TryParse;
    use crate::semantic::Resolve;
    use crate::{
        ast::{
            expressions::data::{Number, Primitive},
            statements::Statement,
        },
        clear_stack,
        semantic::scope::{
            scope_impl::Scope,
            static_types::{NumberType, PrimitiveType},
        },
        vm::vm::{DeserializeFrom, Runtime},
    };
    use crate::{compile_statement, v_num};

    #[test]
    fn valid_loop() {
        let statement = Statement::parse(
            r##"
            let x = {
                let i:u64 = 0;
                loop {
                    i = i + 1;
                    if i >= 10u64 {
                        break;
                    }
                }
                return i; 
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
    fn valid_while() {
        let statement = Statement::parse(
            r##"
            let x = {
                let i:u64 = 0;
                while i < 10u64 {
                    i = i + 1;
                }
                return i; 
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
    fn valid_for_range_inclusive() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res:u64 = 0;
                for i in 0u64..=10u64 {
                    res = i;
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
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 10));
    }

    #[test]
    fn valid_for_slice() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res = 0;
                for i in [1,2,3,4] {
                    res = res + i;
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
        assert_eq!(result, v_num!(I64, 10));
    }

    #[test]
    fn valid_for_slice_assigned() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res = 0;
                let tab = [1,2,3,4];
                for i in tab {
                    res = res + i;
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
        assert_eq!(result, v_num!(I64, 10));
    }

    #[test]
    fn valid_for_str_slice() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res = 'a';
                for i in "abc" {
                    res = i;
                }
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
        assert_eq!(result, Primitive::Char('c'));
    }

    #[test]
    fn valid_for_str_slice_complex() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res = 'a';
                for i in "世世e世世" {
                    res = i;
                }
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
        assert_eq!(result, Primitive::Char('世'));
    }

    #[test]
    fn valid_for_vec() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res = 0;
                for i in vec[1,2,3,4] {
                    res = res + i;
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
        assert_eq!(result, v_num!(I64, 10));
    }

    #[test]
    fn valid_for_string() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res = 'a';
                for i in string("abc") {
                    res = i;
                }
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
        assert_eq!(result, Primitive::Char('c'));
    }

    #[test]
    fn valid_for_double() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res:u64 = 0;
                for i in 0u64..=2u64 {
                    for j in 0u64..=2u64 {
                        res = res + i + j;
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
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 18));
    }

    #[test]
    fn valid_for_early_returns() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res:u64 = 0;
                for i in 5u64..=10u64 {
                    return i;
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
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 5));
    }

    #[test]
    fn valid_for_early_returns_conditional() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res:u64 = 0;
                for i in 5u64..=10u64 {
                    if i == 5u64 {
                        return i;
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
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 5));
    }

    #[test]
    fn valid_for_break() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res:u64 = 0;
                for i in 5u64..=10u64 {
                    res = i;
                    break;
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
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 5));
    }

    #[test]
    fn valid_for_break_conditional() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res:u64 = 0;
                for i in 5u64..=10u64 {
                    res = i;
                    if i == 7u64 {
                        break;
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
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 7));
    }

    #[test]
    fn valid_for_continue() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res:u64 = 0;
                for i in 5u64..=10u64 {
                    continue;
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
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 0));
    }

    #[test]
    fn valid_for_continue_conditional() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res:u64 = 0;
                for i in 5u64..=10u64 {
                    if i != 7u64 {
                        continue;
                    }
                    res = i;
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
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 7));
    }

    #[test]
    fn valid_for_double_continue() {
        let statement = Statement::parse(
            r##"
            let x = {
                let res:u64 = 0;
                for i in 5u64..=10u64 {
                    for j in 5u64..=10u64 {
                        if i != 7u64 {
                            continue;
                        }
                        res = res + i + j;
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
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 87));
    }

    #[test]
    fn valid_for_addr_str_slice() {
        let statement = Statement::parse(
            r##"
            let x = {
                let arr = "abc";
                let res = 'a';
                for i in &arr {
                    res = i;
                }
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
        assert_eq!(result, Primitive::Char('c'));
    }

    #[test]
    fn valid_for_addr_string() {
        let statement = Statement::parse(
            r##"
            let x = {
                let arr = string("abc");
                let res = 'a';
                for i in &arr {
                    res = i;
                }
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
        assert_eq!(result, Primitive::Char('c'));
    }

    #[test]
    fn valid_for_addr_vec() {
        let statement = Statement::parse(
            r##"
            let x = {
                let arr = vec[1,2,3,4];
                let res = 0;
                for i in &arr {
                    res = res + i;
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
        assert_eq!(result, v_num!(I64, 10));
    }

    #[test]
    fn valid_for_addr_slice() {
        let statement = Statement::parse(
            r##"
            let x = {
                let arr = [1,2,3,4];
                let res = 0;
                for i in &arr {
                    res = res + i;
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
        assert_eq!(result, v_num!(I64, 10));
    }

    // #[test]
    // fn valid_for_double_addr_slice() {
    //     let statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let arr = &[1,2,3,4];
    //             let res = 0;
    //             for i in &arr {
    //                 res = res + i;
    //             }
    //             return res;
    //         };
    //         "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 10));
    // }
}
