use crate::vm::{asm::branch::BranchTry, CodeGenerationContext, CodeGenerationError, GenerateCode};

use crate::{
    semantic::SizeOf,
    vm::asm::{
        branch::{BranchIf, Goto, Label},
        Asm,
    },
};

use super::{CallStat, Flow, IfStat, MatchStat, TryStat};

impl GenerateCode for Flow {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            Flow::If(value) => value.gencode::<E>(scope_manager, scope_id, instructions, context),
            Flow::Match(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Flow::Try(value) => value.gencode::<E>(scope_manager, scope_id, instructions, context),
            Flow::Printf(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Flow::Call(value) => value.gencode::<E>(scope_manager, scope_id, instructions, context),
        }
    }
}

impl GenerateCode for CallStat {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let _ = self
            .call
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;
        let Some(return_type) = self.call.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let size = return_type.size_of();

        if size != 0 {
            instructions.push(Asm::Pop(size));
        }
        Ok(())
    }
}

impl GenerateCode for IfStat {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let mut else_label = Label::gen();
        let end_label = Label::gen();

        let _ = self
            .condition
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;

        instructions.push(Asm::If(BranchIf { else_label }));
        let _ = self
            .then_branch
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;
        instructions.push(Asm::Goto(Goto {
            label: Some(end_label),
        }));

        for (condition, block) in &self.else_if_branches {
            instructions.push_label_by_id(else_label, "else_if".to_string().into());

            else_label = Label::gen();

            let _ = condition.gencode::<E>(scope_manager, scope_id, instructions, context)?;

            instructions.push(Asm::If(BranchIf { else_label }));

            let _ = block.gencode::<E>(scope_manager, scope_id, instructions, context)?;
            instructions.push(Asm::Goto(Goto {
                label: Some(end_label),
            }));
        }

        instructions.push_label_by_id(else_label, "else".to_string().into());
        if let Some(block) = &self.else_branch {
            let _ = block.gencode::<E>(scope_manager, scope_id, instructions, context)?;
            instructions.push(Asm::Goto(Goto {
                label: Some(end_label),
            }));
        }

        instructions.push_label_by_id(end_label, "end_if".to_string().into());
        Ok(())
    }
}

impl GenerateCode for MatchStat {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let break_label = Label::gen();
        instructions.push_label("start_match".to_string());
        let _ = self
            .expr
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;

        match &self.cases {
            crate::ast::expressions::flows::Cases::Primitive { cases } => {
                for case in cases {
                    case.gencode::<E>(
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
                    case.gencode::<E>(
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
                    case.gencode::<E>(
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
                    case.gencode::<E>(
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
            block.gencode::<E>(scope_manager, scope_id, instructions, context)?;
        }

        instructions.push_label_by_id(break_label, "end_match".to_string());

        Ok(())
    }
}

impl GenerateCode for TryStat {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let else_label = Label::gen();
        let end_try_label = Label::gen();
        let recover_else_label = Label::gen();

        instructions.push(Asm::Try(BranchTry::StartTry {
            else_label: recover_else_label,
        }));

        let _ = self
            .try_branch
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;

        instructions.push(Asm::Goto(Goto {
            label: Some(end_try_label),
        }));

        instructions.push_label_by_id(recover_else_label, "recover_else".to_string().into());

        instructions.push_label_by_id(else_label, "else".to_string().into());
        instructions.push(Asm::Try(BranchTry::EndTry));

        if let Some(block) = &self.else_branch {
            block.gencode::<E>(scope_manager, scope_id, instructions, context)?;
        }

        instructions.push_label_by_id(end_try_label, "end_try".to_string().into());
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{test_extract_variable, test_statements};

    #[test]
    fn valid_if() {
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            let res = test_extract_variable::<i64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            true
        }

        test_statements(
            r##"
        let cond = true;
        let res1 = 0;
        if cond {
            res1 = 5;
        }
        
        let res2 = 0;
        if false {
            res2 = 1;
        } else {
            res2 = 2;
        }

        let res3 = 0;
        let x = 2;
        if x == 1 {
            res3 = 1;
        } else if x == 2 {
            res3 = 2;
        } else if x == 3 {
            res3 = 3;
        } else {
            res3 = 4;
        }

        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_if_with_inner_vars() {
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            true
        }

        test_statements(
            r##"
        let cond = true;
        let res1 = 0;
        if cond {
            let x = 2;
            res1 = x;
        }
        

        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_match() {
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("res4", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("res5", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("res6", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("res7", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("res8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("res9", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            true
        }

        test_statements(
            r##"

        let test1 = 1;
        let res1 = 0;
        match test1 {
            case 1 => {
                res1 = 2;
            }
            else => {
                res1 = 5;
            }
        }

        let test2 = 3;
        let res2 = 0;
        match test2 {
            case 1 => {
                res2 = 5;
            },
            case 2 => {
                res2 = 5;
            },
            case 3 => {
                res2 = 2;
            },
            else => {
                res2 = 5;
            }
        }

        let test3 = 4;
        let res3 = 0;
        match test3 {
            case 1 => {
                res3 = 5;
            },
            case 3 => {
                res3 = 5;
            },
            case 3 => {
                res3 = 5;
            },
            else => {
                res3 = 2;
            }
        }

        enum ETest{
            TEST1,
            TEST2,
        }
        
        let test4 = ETest::TEST1;
        let res4 = 0;
        match test4 {
            case ETest::TEST1 => {
                res4 = 2;
            },
            case ETest::TEST2 => {
                res4 = 5;
            },
        }

        
        enum ETest2 {
            TEST1,
            TEST2,
            TEST3,
        }
        
        let test5 = ETest2::TEST3;
        let res5 = 0;
        match test5 {
            case ETest2::TEST1 => {
                res5 = 5;
            },
            case ETest2::TEST2 => {
                res5 = 5;
            },
            else => {
                res5 = 2;
            }
        }
 
        let test6 = "test";
        let res6 = 1;
        match test6 {
            case "test" => {
                res6 = 2;
            },
            else => {
                res6 = 5;
            }
        }
 
        let test7 = "success";
        let res7 = 1;
        match test7 {
            case "test" => {
                res7 = 5;
            },
            case "error" => {
                res7 = 5;
            },
            else => {
                res7 = 2;
            }
        }

        let test8 = 3;
        let res8 = 0;
        match test8 {
            case 1 | 3 => {
                res8 = 2;
            },
            else => {
                res8 = 5;
            }
        }

        union Test {
            Point {
                x : i64,
                y : i64,
            },
            Point2 {
                x : u32,
                y : i64,
            }
        }
        let test9 = Test::Point2 { x : 1, y : 5 };
        let res9 = 0;

        match test9 {
            case Test::Point { x, y } => {
                res9 = 2;
            },
            case Test::Point2 { x, y } => {
                res9 = y;
            }
        }

        "##,
            &mut engine,
            assert_fn,
        );
    }
}
