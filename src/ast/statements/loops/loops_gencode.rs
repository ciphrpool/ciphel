use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::SizeOf;
use crate::vm::asm::branch::BranchIf;

use crate::vm::asm::{
    branch::{Goto, Label},
    mem::Mem,
    Asm,
};
use crate::vm::{CodeGenerationContext, GenerateCode};

use super::{ForInit, ForLoop, Loop, WhileLoop};

impl GenerateCode for Loop {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            Loop::For(value) => value.gencode::<E>(scope_manager, scope_id, instructions, context),
            Loop::While(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Loop::Loop(value) => {
                let start_label = Label::gen();
                let end_label = Label::gen();
                let break_label = Label::gen();
                let continue_label = Label::gen();

                instructions.push_label_by_id(start_label, "start_loop".to_string().into());

                value.gencode::<E>(
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

                instructions.push(Asm::Goto(Goto {
                    label: Some(start_label),
                }));

                instructions.push_label_by_id(break_label, "break_loop".to_string().into());
                instructions.push(Asm::Goto(Goto {
                    label: Some(end_label),
                }));
                instructions.push_label_by_id(continue_label, "continue_loop".to_string().into());
                instructions.push(Asm::Goto(Goto {
                    label: Some(start_label),
                }));
                instructions.push_label_by_id(end_label, "end_loop".to_string().into());

                Ok(())
            }
        }
    }
}

impl GenerateCode for ForLoop {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let break_label = Label::gen();
        let continue_label = Label::gen();
        let end_label = Label::gen();
        let start_label = Label::gen();
        let epilog_label = Label::gen();

        for index in self.indices.iter() {
            match index {
                ForInit::Assignation(assignation) => {
                    let _ =
                        assignation.gencode::<E>(scope_manager, scope_id, instructions, context)?;
                }
                ForInit::Declaration(declaration) => {
                    let _ =
                        declaration.gencode::<E>(scope_manager, scope_id, instructions, context)?;
                }
            }
        }

        instructions.push_label_by_id(start_label, "start_loop".to_string());

        if let Some(condition) = &self.condition {
            let _ = condition.gencode::<E>(scope_manager, scope_id, instructions, context)?;
            instructions.push(Asm::If(BranchIf {
                else_label: break_label,
            }));
        }

        self.block.gencode::<E>(
            scope_manager,
            scope_id,
            instructions,
            &CodeGenerationContext {
                return_label: context.return_label.clone(),
                break_label: Some(break_label),
                continue_label: Some(continue_label),
            },
        )?;

        // Loop epilog
        instructions.push_label_by_id(epilog_label, "epilog_loop".to_string());
        instructions.push(Asm::Goto(Goto {
            label: Some(continue_label),
        }));
        instructions.push_label_by_id(continue_label, "continue_loop".to_string());
        for increment in self.increments.iter() {
            let _ = increment.gencode::<E>(scope_manager, scope_id, instructions, context)?;
        }
        instructions.push(Asm::Goto(Goto {
            label: Some(start_label),
        }));
        instructions.push_label_by_id(break_label, "break_loop".to_string());
        instructions.push(Asm::Goto(Goto {
            label: Some(end_label),
        }));
        instructions.push_label_by_id(end_label, "end_loop".to_string());

        Ok(())
    }
}

impl GenerateCode for WhileLoop {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let start_label = Label::gen();
        let end_label = Label::gen();
        let break_label = Label::gen();
        let continue_label = Label::gen();
        let epilog_label = Label::gen();

        instructions.push_label_by_id(start_label, "start_while".to_string().into());

        let _ = self
            .condition
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;

        instructions.push(Asm::If(BranchIf {
            else_label: end_label,
        }));
        self.block.gencode::<E>(
            scope_manager,
            scope_id,
            instructions,
            &CodeGenerationContext {
                return_label: context.return_label.clone(),
                break_label: Some(break_label),
                continue_label: Some(continue_label),
            },
        )?;

        // Loop epilog
        instructions.push_label_by_id(epilog_label, "epilog_loop".to_string());
        instructions.push(Asm::Goto(Goto {
            label: Some(continue_label),
        }));
        instructions.push_label_by_id(continue_label, "continue_loop".to_string());
        instructions.push(Asm::Goto(Goto {
            label: Some(start_label),
        }));

        instructions.push_label_by_id(break_label, "break_loop".to_string());
        instructions.push(Asm::Goto(Goto {
            label: Some(end_label),
        }));
        instructions.push_label_by_id(end_label, "end_loop".to_string());

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
    };
    use crate::{test_extract_variable, test_statements, v_num};

    #[test]
    fn valid_for() {
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 45);
            let res = test_extract_variable::<i64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 15);
            let res = test_extract_variable::<i64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 20);
            let res = test_extract_variable::<i64>("res4", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 150);
            true
        }

        test_statements(
            r##"

        let res1 = 0;
        for ( let i = 0; i < 10; i = i + 1) {
            res1 = res1 + i;
        }


        let res2 = 0;
        for ( let i = 0; i < 10; i = i + 1) {
            res2 = res2 + i;
            if i >= 5 {
                break;
            }
        }
        
        let res3 = 0;
        for ( let i = 0; i < 10; i = i + 1) {
            if i % 2 != 0{
                continue;
            } else {
                res3 = res3 + i;
            }
        }


                
        let res4 = 0;
        for ( let i = 0; i < 10; i = i + 1) {
            for ( let j = 0; j < 10; j = j + 1) {
                res4 = res4 + j;
                if j >= 5 {
                    break;
                }
            }
        }
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_while() {
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 45);
            let res = test_extract_variable::<i64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 15);
            let res = test_extract_variable::<i64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 20);
            true
        }

        test_statements(
            r##"

        let res1 = 0;
        let i = 0;
        while i < 10 {
            res1 = res1 + i;
            i = i + 1;
        }


        let res2 = 0;
        let i = 0;
        while i < 10 {
            res2 = res2 + i;
            if i >= 5 {
                break;
            }
            i = i + 1;
        }
        
        let res3 = 0;
        let i = 0;
        while i < 9 {
            i = i + 1;
            if i % 2 != 0{
                continue;
            } else {
                res3 = res3 + i;
            }
        }
        
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_loop() {
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 45);

            true
        }

        test_statements(
            r##"

        let res1 = 0;
        let i = 0;
        loop {
            res1 = res1 + i;
            i = i + 1;
            if i >= 10 {
                break;
            }
        }
        "##,
            &mut engine,
            assert_fn,
        );
    }
    // cargo.exe test --package ciphel --lib -- ast::statements::loops::loops_gencode::tests::valid_loop --show-output --nocapture
    // #[test]
    // fn valid_loop() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let i:u64 = 0;
    //             loop {
    //                 i = i + 1;
    //                 if i >= 3u64 {
    //                     break;
    //                 }
    //             }
    //             return i;
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
    //     assert_eq!(result, v_num!(I64, 3));
    // }

    // #[test]
    // fn valid_while() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let i:u64 = 0;
    //             while i < 10u64 {
    //                 i = i + 1;
    //             }
    //             return i;
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
    // fn valid_for() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let res:u64 = 0;
    //             for(let i:u64 = 0;i < 10;i = i + 1) {
    //                 res = res + i;
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
    //     assert_eq!(result, v_num!(U64, 10));
    // }

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
