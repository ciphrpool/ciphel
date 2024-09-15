use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::SizeOf;
use crate::vm::casm::branch::BranchIf;

use crate::vm::vm::CodeGenerationContext;
use crate::vm::{
    casm::{
        branch::{Goto, Label},
        mem::Mem,
        Casm, CasmProgram,
    },
    vm::{CodeGenerationError, GenerateCode},
};

use super::{ForLoop, Loop, WhileLoop};

impl GenerateCode for Loop {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Loop::For(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Loop::While(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Loop::Loop(value) => {
                let start_label = Label::gen();
                let end_label = Label::gen();
                let break_label = Label::gen();
                let continue_label = Label::gen();

                instructions.push_label_id(start_label, "start_loop".to_string().into());

                value.gencode(
                    scope_manager,
                    scope_id,
                    instructions,
                    &CodeGenerationContext {
                        return_label: context.return_label.clone(),
                        break_label: Some(break_label),
                        continue_label: Some(continue_label),
                        ..Default::default()
                    },
                )?;

                instructions.push(Casm::Goto(Goto {
                    label: Some(start_label),
                }));

                instructions.push_label_id(break_label, "break_loop".to_string().into());
                instructions.push(Casm::Goto(Goto {
                    label: Some(end_label),
                }));
                instructions.push_label_id(continue_label, "continue_loop".to_string().into());
                instructions.push(Casm::Goto(Goto {
                    label: Some(start_label),
                }));
                instructions.push_label_id(end_label, "end_loop".to_string().into());

                Ok(())
            }
        }
    }
}

impl GenerateCode for ForLoop {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let return_label = Label::gen();
        let break_label = Label::gen();
        let continue_label = Label::gen();

        todo!();
        self.block.gencode(
            scope_manager,
            scope_id,
            instructions,
            &CodeGenerationContext {
                return_label: context.return_label.clone(),
                break_label: Some(break_label),
                continue_label: Some(continue_label),
                ..Default::default()
            },
        )?;

        // instructions.push_label_id(break_label, "break_block".to_string().into());
        // instructions.push(Casm::StackFrame(StackFrame::SoftClean));
        // instructions.push(Casm::Goto(Goto {
        //     label: context.break_label,
        // }));
        // instructions.push_label_id(continue_label, "continue_block".to_string().into());
        // instructions.push(Casm::StackFrame(StackFrame::SoftClean));
        // instructions.push(Casm::Goto(Goto {
        //     label: context.continue_label,
        // }));
        // instructions.push_label_id(return_label, "return_block".to_string().into());
        // instructions.push(Casm::Goto(Goto {
        //     label: context.return_label,
        // }));
        Ok(())
    }
}

impl GenerateCode for WhileLoop {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let start_label = Label::gen();
        let end_label = Label::gen();
        let break_label = Label::gen();
        let continue_label = Label::gen();

        instructions.push_label_id(start_label, "start_while".to_string().into());

        let _ = self
            .condition
            .gencode(scope_manager, scope_id, instructions, context)?;

        instructions.push(Casm::If(BranchIf {
            else_label: end_label,
        }));
        self.block.gencode(
            scope_manager,
            scope_id,
            instructions,
            &CodeGenerationContext {
                return_label: context.return_label.clone(),
                break_label: Some(break_label),
                continue_label: Some(continue_label),
                ..Default::default()
            },
        )?;

        instructions.push(Casm::Goto(Goto {
            label: Some(start_label),
        }));

        instructions.push_label_id(break_label, "break_loop".to_string().into());
        instructions.push(Casm::Goto(Goto {
            label: Some(end_label),
        }));
        instructions.push_label_id(continue_label, "continue_loop".to_string().into());
        instructions.push(Casm::Goto(Goto {
            label: Some(start_label),
        }));

        instructions.push_label_id(end_label, "end_while".to_string().into());

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ast::TryParse;
    use crate::semantic::Resolve;
    use crate::{
        ast::{expressions::data::Primitive, statements::Statement},
        semantic::scope::{
            scope::ScopeManager,
            static_types::{NumberType, PrimitiveType},
        },
        vm::vm::DeserializeFrom,
    };
    use crate::{compile_statement, v_num};
    // cargo.exe test --package ciphel --lib -- ast::statements::loops::loops_gencode::tests::valid_loop --show-output --nocapture
    #[test]
    fn valid_loop() {
        let mut statement = Statement::parse(
            r##"
            let x = {
                let i:u64 = 0;
                loop {
                    i = i + 1;
                    if i >= 3u64 {
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
        assert_eq!(result, v_num!(I64, 3));
    }

    #[test]
    fn valid_while() {
        let mut statement = Statement::parse(
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
    fn valid_for() {
        let mut statement = Statement::parse(
            r##"
            let x = {
                let res:u64 = 0;
                for(let i:u64 = 0;i < 10;i = i + 1) {
                    res = res + i;
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
        assert_eq!(result, v_num!(U64, 10));
    }

    // #[test]
    // fn valid_for_range_inclusive() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res:u64 = 0;
    //             for i in 0u64..=10u64 {
    //                 res = i;
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
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 10));
    // }

    // #[test]
    // fn valid_for_slice() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res = 0;
    //             for i in [1,2,3,4] {
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

    // #[test]
    // fn valid_for_slice_assigned() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res = 0;
    //             let tab = [1,2,3,4];
    //             for i in tab {
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

    // #[test]
    // fn valid_for_str_slice() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res = 'a';
    //             for i in "abc" {
    //                 res = i;
    //             }
    //             return res;
    //         };
    //         "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(result, Primitive::Char('c'));
    // }

    // #[test]
    // fn valid_for_str_slice_complex() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res = 'a';
    //             for i in "世世e世世" {
    //                 res = i;
    //             }
    //             return res;
    //         };
    //         "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(result, Primitive::Char('世'));
    // }

    // #[test]
    // fn valid_for_vec() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res = 0;
    //             for i in vec[1,2,3,4] {
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

    // #[test]
    // fn valid_for_string() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res = 'a';
    //             for i in string("abc") {
    //                 res = i;
    //             }
    //             return res;
    //         };
    //         "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(result, Primitive::Char('c'));
    // }

    // #[test]
    // fn valid_for_double() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res:u64 = 0;
    //             for i in 0u64..=2u64 {
    //                 for j in 0u64..=2u64 {
    //                     res = res + i + j;
    //                 }
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
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 18));
    // }

    // #[test]
    // fn valid_for_early_returns() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res:u64 = 0;
    //             for i in 5u64..=10u64 {
    //                 return i;
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
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 5));
    // }

    // #[test]
    // fn valid_for_early_returns_conditional() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res:u64 = 0;
    //             for i in 5u64..=10u64 {
    //                 if i == 5u64 {
    //                     return i;
    //                 }
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
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 5));
    // }

    // #[test]
    // fn valid_for_break() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res:u64 = 0;
    //             for i in 5u64..=10u64 {
    //                 res = i;
    //                 break;
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
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 5));
    // }

    // #[test]
    // fn valid_for_break_conditional() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res:u64 = 0;
    //             for i in 5u64..=10u64 {
    //                 res = i;
    //                 if i == 7u64 {
    //                     break;
    //                 }
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
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 7));
    // }

    // #[test]
    // fn valid_for_continue() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res:u64 = 0;
    //             for i in 5u64..=10u64 {
    //                 continue;
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
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 0));
    // }

    // #[test]
    // fn valid_for_continue_conditional() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res:u64 = 0;
    //             for i in 5u64..=10u64 {
    //                 if i != 7u64 {
    //                     continue;
    //                 }
    //                 res = i;
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
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 7));
    // }

    // #[test]
    // fn valid_for_double_continue() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res:u64 = 0;
    //             for i in 5u64..=10u64 {
    //                 for j in 5u64..=10u64 {
    //                     if i != 7u64 {
    //                         continue;
    //                     }
    //                     res = res + i + j;
    //                 }
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
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 87));
    // }
    // #[test]
    // fn valid_for_str_slice_with_padding() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let arr:str<10> = "abc";
    //             let res = 'a';
    //             for i in &arr {
    //                 res = i;
    //             }
    //             return res;
    //         };
    //         "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(result, Primitive::Char('c'));
    // }
    // #[test]
    // fn valid_for_addr_str_slice() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let arr = "abc";
    //             let res = 'a';
    //             for i in &arr {
    //                 res = i;
    //             }
    //             return res;
    //         };
    //         "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(result, Primitive::Char('c'));
    // }

    // #[test]
    // fn valid_for_addr_string() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let arr = string("abc");
    //             let res = 'a';
    //             for i in &arr {
    //                 res = i;
    //             }
    //             return res;
    //         };
    //         "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(result, Primitive::Char('c'));
    // }

    // #[test]
    // fn valid_for_addr_vec() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let arr = vec[1,2,3,4];
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

    // #[test]
    // fn valid_for_addr_slice() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let arr = [1,2,3,4];
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

    // #[test]
    // fn valid_for_double_addr_slice() {
    //     let mut statement = Statement::parse(
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
