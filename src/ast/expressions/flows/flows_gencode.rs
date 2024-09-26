use num_traits::ToBytes;

use std::fmt::Debug;
use ulid::Ulid;

use crate::ast::statements::block::BlockCommonApi;
use crate::ast::TryParse;
use crate::semantic::scope::scope::{ScopeManager, ScopeState, VariableInfo};
use crate::semantic::scope::static_types::POINTER_SIZE;
use crate::semantic::{EType, Resolve, SemanticError};
use crate::vm::asm::branch::{BranchTry, CloseFrame, Return};
use crate::vm::asm::data;
use crate::vm::asm::mem::Mem;
use crate::vm::asm::operation::{Equal, Operation, StrEqual};
use crate::vm::core::ERROR_VALUE;
use crate::vm::vm::CodeGenerationContext;
use crate::{
    ast::expressions::data::{Number, Primitive},
    semantic::{
        scope::{
            static_types::StaticType,
            user_types::{Enum, Union, UserType},
        },
        SizeOf, TypeOf,
    },
    vm::{
        asm::{
            branch::{BranchIf, Call, Goto, Label},
            data::Data,
            Asm, Program,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{
    EnumCase, ExprFlow, FCall, IfExpr, MatchExpr, PrimitiveCase, StringCase, TryExpr, UnionCase,
};

impl GenerateCode for ExprFlow {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            ExprFlow::If(value) => value.gencode(scope_manager, scope_id, instructions, context),
            ExprFlow::Match(value) => value.gencode(scope_manager, scope_id, instructions, context),
            ExprFlow::Try(value) => value.gencode(scope_manager, scope_id, instructions, context),
            ExprFlow::SizeOf(value, _metadata) => {
                let value = value
                    .type_of(scope_manager, scope_id)
                    .map_err(|_| CodeGenerationError::UnresolvedError)?;

                instructions.push(Asm::Data(Data::Serialized {
                    data: (value.size_of() as u64).to_le_bytes().into(),
                }));
                Ok(())
            }
            ExprFlow::FCall(value) => value.gencode(scope_manager, scope_id, instructions, context),
        }
    }
}

impl GenerateCode for IfExpr {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let else_label = Label::gen();
        let end_label = Label::gen();

        let _ = self
            .condition
            .gencode(scope_manager, scope_id, instructions, context)?;

        instructions.push(Asm::If(BranchIf { else_label }));
        let _ = self
            .then_branch
            .gencode(scope_manager, scope_id, instructions, context)?;
        instructions.push(Asm::Goto(Goto {
            label: Some(end_label),
        }));

        instructions.push_label_id(else_label, "else".to_string().into());
        let _ = self
            .else_branch
            .gencode(scope_manager, scope_id, instructions, context)?;
        instructions.push(Asm::Goto(Goto {
            label: Some(end_label),
        }));

        instructions.push_label_id(end_label, "end_if".to_string().into());

        Ok(())
    }
}

impl<B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq> GenerateCode
    for PrimitiveCase<B>
{
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let block_label = Label::gen();
        let mut else_label = Label::gen();

        for (i, value) in self.patterns.iter().enumerate() {
            instructions.push_label_id(else_label, format!("case_{}", i));
            instructions.push(Asm::Mem(Mem::Dup(value.size_of())));

            value.gencode(scope_manager, scope_id, instructions, context)?;

            instructions.push(Asm::Operation(Operation {
                kind: crate::vm::asm::operation::OperationKind::Equal(Equal {
                    left: value.size_of(),
                    right: value.size_of(),
                }),
            }));

            else_label = Label::gen();
            instructions.push(Asm::If(BranchIf { else_label }));
            instructions.push(Asm::Pop(value.size_of()));
            instructions.push(Asm::Goto(Goto {
                label: Some(block_label),
            }));
        }

        instructions.push_label_id(block_label, "match_block".to_string());
        self.block
            .gencode(scope_manager, scope_id, instructions, context)?;
        instructions.push(Asm::Goto(Goto {
            label: context.break_label,
        }));
        instructions.push_label_id(else_label, "fallthrough".to_string());

        Ok(())
    }
}

impl<B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq> GenerateCode
    for StringCase<B>
{
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let block_label = Label::gen();
        let mut else_label = Label::gen();

        for (i, value) in self.patterns.iter().enumerate() {
            instructions.push_label_id(else_label, format!("case_{}", i));
            instructions.push(Asm::Mem(Mem::Dup(POINTER_SIZE)));

            value.gencode(scope_manager, scope_id, instructions, context)?;

            instructions.push(Asm::Operation(Operation {
                kind: crate::vm::asm::operation::OperationKind::StrEqual(StrEqual),
            }));

            else_label = Label::gen();
            instructions.push(Asm::If(BranchIf { else_label }));
            instructions.push(Asm::Pop(POINTER_SIZE));
            instructions.push(Asm::Goto(Goto {
                label: Some(block_label),
            }));
        }

        instructions.push_label_id(block_label, "match_block".to_string());
        self.block
            .gencode(scope_manager, scope_id, instructions, context)?;
        instructions.push(Asm::Goto(Goto {
            label: context.break_label,
        }));
        instructions.push_label_id(else_label, "fallthrough".to_string());

        Ok(())
    }
}

impl<B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq> GenerateCode
    for EnumCase<B>
{
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let block_label = Label::gen();
        let mut else_label = Label::gen();

        for (i, (_, _, value)) in self.patterns.iter().enumerate() {
            instructions.push_label_id(else_label, format!("case_{}", i));
            instructions.push(Asm::Mem(Mem::Dup(POINTER_SIZE)));

            let Some(value) = value else {
                return Err(CodeGenerationError::UnresolvedError);
            };
            instructions.push(Asm::Data(Data::Serialized {
                data: (*value).to_le_bytes().into(),
            }));

            instructions.push(Asm::Operation(Operation {
                kind: crate::vm::asm::operation::OperationKind::Equal(Equal { left: 8, right: 8 }),
            }));

            else_label = Label::gen();
            instructions.push(Asm::If(BranchIf { else_label }));
            instructions.push(Asm::Pop(POINTER_SIZE));
            instructions.push(Asm::Goto(Goto {
                label: Some(block_label),
            }));
        }

        instructions.push_label_id(block_label, "match_block".to_string());
        self.block
            .gencode(scope_manager, scope_id, instructions, context)?;
        instructions.push(Asm::Goto(Goto {
            label: context.break_label,
        }));
        instructions.push_label_id(else_label, "fallthrough".to_string());

        Ok(())
    }
}

impl<B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq> GenerateCode
    for UnionCase<B>
{
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let block_label = Label::gen();
        let else_label = Label::gen();

        let Some(value) = self.pattern.variant_value else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        instructions.push(Asm::Mem(Mem::Dup(POINTER_SIZE)));

        instructions.push(Asm::Data(Data::Serialized {
            data: value.to_le_bytes().into(),
        }));

        instructions.push(Asm::Operation(Operation {
            kind: crate::vm::asm::operation::OperationKind::Equal(Equal { left: 8, right: 8 }),
        }));

        instructions.push(Asm::If(BranchIf { else_label }));
        instructions.push(Asm::Pop(POINTER_SIZE));

        instructions.push_label_id(block_label, "match_block".to_string());

        let Some(ids) = &self.pattern.vars_id else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let Some(inner_scope) = self.block.scope() else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        if ScopeState::IIFE
            == *scope_manager
                .scope_states
                .get(&inner_scope)
                .unwrap_or(&ScopeState::Inline)
        {
            // IIFE : the created vars are left on the stack to be picked up by the IIFE
        } else {
            for id in ids.iter().rev() {
                let Some(VariableInfo { address, ctype, .. }) =
                    scope_manager.find_var_by_id(*id).ok()
                else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                instructions.push(Asm::Mem(Mem::Store {
                    size: ctype.size_of(),
                    address: (*address)
                        .try_into()
                        .map_err(|_| CodeGenerationError::UnresolvedError)?,
                }));
            }
        }

        self.block
            .gencode(scope_manager, scope_id, instructions, context)?;
        instructions.push(Asm::Goto(Goto {
            label: context.break_label,
        }));
        instructions.push_label_id(else_label, "fallthrough".to_string());
        Ok(())
    }
}

impl GenerateCode for MatchExpr {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
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

impl GenerateCode for TryExpr {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(return_size) = self.metadata.signature().map(|t| t.size_of()) else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let else_label = Label::gen();
        let recover_else_label = Label::gen();
        let end_label = Label::gen();

        let _ = instructions.push_label("try".to_string().into());

        instructions.push(Asm::Try(BranchTry::StartTry {
            else_label: recover_else_label,
        }));

        let _ = self.try_branch.gencode(
            scope_manager,
            scope_id,
            instructions,
            &CodeGenerationContext::default(),
        )?;

        // (1)
        if self.pop_last_err {
            let next = Label::gen();
            /* Pop the error */
            instructions.push(Asm::If(BranchIf { else_label: next }));
            instructions.push(Asm::Pop(return_size)); // discard error value
            instructions.push(Asm::Goto(Goto {
                label: Some(else_label),
            }));
            instructions.push_label_id(next, "else".to_string().into());
        }

        instructions.push(Asm::Try(BranchTry::EndTry));

        instructions.push(Asm::Goto(Goto {
            label: Some(end_label),
        }));
        instructions.push_label_id(recover_else_label, "recover_else".to_string().into());

        if self.pop_last_err {
            // Push dummy data that will be returned
            let mut dummy_data = vec![0; return_size];
            dummy_data.push(ERROR_VALUE);

            instructions.push(Asm::Data(Data::Serialized {
                data: vec![0; return_size].into(),
            }));
            instructions.push(Asm::Return(Return { size: return_size })); // Once return the cursor will go back to (1)
        } else {
            instructions.push(Asm::CloseFrame(CloseFrame));
        }

        instructions.push_label_id(else_label, "else".to_string().into());
        instructions.push(Asm::Try(BranchTry::EndTry));

        if let Some(block) = &self.else_branch {
            block.gencode(scope_manager, scope_id, instructions, context)?;
        }

        instructions.push_label_id(end_label, "end_try".to_string().into());
        Ok(())
    }
}

impl GenerateCode for FCall {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        todo!();

        // for item in &self.value {
        //     match item {
        //         super::FormatItem::Str(string) => {
        //             let str_bytes: Box<[u8]> = string.as_bytes().into();
        //             let size = (&str_bytes).len() as u64;
        //             instructions.push(Asm::Data(data::Data::Serialized { data: str_bytes }));
        //             instructions.push(Asm::Data(data::Data::Serialized {
        //                 data: size.to_le_bytes().into(),
        //             }));
        //             instructions.push(Asm::Core(CoreCasm::Std(StdCasm::Strings(
        //                 StringsCasm::ToStr(ToStrCasm::ToStrStrSlice),
        //             ))));
        //         }
        //         super::FormatItem::Expr(expr) => {
        //             let _ = expr.gencode(scope_manager, scope_id, instructions, context)?;
        //         }
        //     }
        // }
        // instructions.push(Asm::Core(CoreCasm::Std(StdCasm::Strings(
        //     StringsCasm::Join(JoinCasm::NoSepFromSlice(Some(self.value.len()))),
        // ))));
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive, Struct},
                Atomic, Expression,
            },
            statements::Statement,
            TryParse,
        },
        clear_stack, p_num,
        semantic::{
            scope::{
                scope::ScopeManager,
                static_types::{NumberType, PrimitiveType},
                user_types::{self, UserType},
            },
            Resolve,
        },
        test_extract_variable, test_extract_variable_with, test_statements, v_num,
        vm::{
            asm::operation::OpPrimitive,
            vm::{Executable, Runtime},
        },
    };

    use super::*;

    #[test]
    fn valid_if() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1);
            let res = test_extract_variable::<i64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            true
        }

        test_statements(
            r##"

        let res1 = if true then {
            1
        } else {
            2
        };

        let res2 = if false then {
            1
        } else {
            2
        };
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_if_with_inner_var() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 6);
            let res = test_extract_variable::<i64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 7);
            true
        }

        test_statements(
            r##"

        let res1 = if true then {
            let x = 5;
            x + 1
        } else {
            2
        };

        let res2 = if false then {
            1
        } else {
            let x = 5;
            x + 2
        };
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_if_with_inner_var_in_local_scope() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 6);
            let res = test_extract_variable::<i64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            true
        }

        test_statements(
            r##"

        let res1 = {
            let y = if true then {
                let x = 5;
                x + 1
            } else {
                2
            };
            y
        };

        let res2 = {
            let y = if false then {
                1
            } else {
                2
            };
            y
        };
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_match() {
        let mut engine = crate::vm::vm::DbgGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            let res = test_extract_variable::<i64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            let res = test_extract_variable::<i64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            let res = test_extract_variable::<u32>("res4", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1);
            true
        }

        test_statements(
            r##"

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

        let res1 = match 1 {
            case 1 | 2 => { 5 },
            else => { 10 }
        };


        let var2 = Test::Point { x : 1, y : 5 };
        let res2 = match var2 {
            case Test::Point { x, y } => { y },
            else => { 10 }
        };

        let res3 = {  
            let var3 = Test::Point { x : 1, y : 5 };
            let res3 = match var3 {
                case Test::Point { x, y } => { y },
                else => { 10 }
            };
            res3
        };

        let res4 = {  
            let var4 = Test::Point2 { x : 1, y : 5 };
            let res4 = match var4 {
                case Test::Point { x, y } => { y },
                case Test::Point2 { x, y } => { x },
                else => { 10 }
            };
            res4
        };
        "##,
            &mut engine,
            assert_fn,
        );
    }

    // #[test]
    // fn valid_if_basic() {
    //     let mut statement_then = IfExpr::parse(
    //         r##"
    //        if true then 420 else 69
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = statement_then
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");
    //     // Code generation.
    //     let mut instructions_then = Program::default();
    //     statement_then
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions_then,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions_then.len() > 0);
    //     // Execute the instructions.

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions_then);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 420));
    // }

    // #[test]
    // fn valid_if_basic_else() {
    //     let mut statement_else = IfExpr::parse(
    //         r##"
    //        if false then 420 else 69
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = statement_else
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions_else = Program::default();
    //     statement_else
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions_else,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions_else.len() > 0);
    //     // Execute the instructions.
    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions_else);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 69));
    // }

    // #[test]
    // fn valid_if_basic_scope() {
    //     let mut statement_then = IfExpr::parse(
    //         r##"
    //        if true then {
    //            let x = 420;
    //            return x;
    //        } else 69
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = statement_then
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions_then = Program::default();
    //     statement_then
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions_then,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions_then.len() > 0);
    //     // Execute the instructions.

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions_then);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 420));
    // }

    // #[test]
    // fn valid_if_basic_scope_else() {
    //     let mut statement_else = IfExpr::parse(
    //         r##"
    //        if false then 420 else {
    //         let x = 69;
    //         return x;
    //         }
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();

    //     let _ = statement_else
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.

    //     let mut instructions_else = Program::default();
    //     statement_else
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions_else,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions_else.len() > 0);
    //     // Execute the instructions.
    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions_else);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 69));
    // }

    // #[test]
    // fn valid_if_complex() {
    //     let user_type = user_types::Struct {
    //         id: "Point".to_string().into(),
    //         fields: {
    //             let mut res = Vec::new();
    //             res.push(("x".to_string().into(), p_num!(I64)));
    //             res.push(("y".to_string().into(), p_num!(I64)));
    //             res
    //         },
    //     };
    //     let mut statement_then = IfExpr::parse(
    //         r##"
    //     if true then {
    //         let point:Point;
    //         point.x = 420;
    //         point.y = 420;
    //         return point;
    //     } else Point {
    //         x : 69,
    //         y : 69
    //     }
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager
    //         .register_type("Point", UserType::Struct(user_type.clone()), None)
    //         .expect("Registering of user type should have succeeded");
    //     let _ = statement_then
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions_then = Program::default();
    //     statement_then
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions_then,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");
    //     assert!(instructions_then.len() > 0);
    //     // Execute the instructions.

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions_then);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let result: Struct = user_type
    //         .deserialize_from(&data)
    //         .expect("Deserialization should have succeeded");

    //     for (r_id, res) in &result.fields {
    //         match res {
    //             Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
    //                 match x {
    //                     Number::I64(res) => {
    //                         if *r_id == "x" {
    //                             assert_eq!(420, *res);
    //                         } else if *r_id == "y" {
    //                             assert_eq!(420, *res);
    //                         }
    //                     }
    //                     _ => assert!(false, "Expected i64"),
    //                 }
    //             }
    //             _ => assert!(false, "Expected i64"),
    //         }
    //     }
    // }

    // #[test]
    // fn valid_if_complex_else() {
    //     let user_type = user_types::Struct {
    //         id: "Point".to_string().into(),
    //         fields: {
    //             let mut res = Vec::new();
    //             res.push(("x".to_string().into(), p_num!(I64)));
    //             res.push(("y".to_string().into(), p_num!(I64)));
    //             res
    //         },
    //     };
    //     let mut statement_else = IfExpr::parse(
    //         r##"
    //     if false then {
    //         let point:Point;
    //         point.x = 420;
    //         point.y = 420;
    //         return point;
    //     } else Point {
    //         x : 69,
    //         y : 69
    //     }
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager
    //         .register_type("Point", UserType::Struct(user_type.clone()), None)
    //         .expect("Registering of user type should have succeeded");
    //     let _ = statement_else
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions_else = Program::default();
    //     statement_else
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions_else,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");
    //     assert!(instructions_else.len() > 0);
    //     // Execute the instructions.

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions_else);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let result: Struct = user_type
    //         .deserialize_from(&data)
    //         .expect("Deserialization should have succeeded");

    //     for (r_id, res) in &result.fields {
    //         match res {
    //             Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(x)))) => {
    //                 match x {
    //                     Number::I64(res) => {
    //                         if *r_id == "x" {
    //                             assert_eq!(69, *res);
    //                         } else if *r_id == "y" {
    //                             assert_eq!(69, *res);
    //                         }
    //                     }
    //                     _ => assert!(false, "Expected i64"),
    //                 }
    //             }
    //             _ => assert!(false, "Expected i64"),
    //         }
    //     }
    // }

    // #[test]
    // fn valid_if_complex_outvar() {
    //     let mut statement_then = Statement::parse(
    //         r##"
    //     let x = {
    //         let y = true;
    //         return if y then 420 else 69;
    //     };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = statement_then
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions_then = Program::default();
    //     statement_then
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions_then,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions_then.len() > 0);
    //     // Execute the instructions.

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions_then);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};
    //     dbg!(&program.main);
    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 420));
    // }

    // #[test]
    // fn valid_match_union() {
    //     let user_type = user_types::Union {
    //         id: "Geo".to_string().into(),
    //         variants: {
    //             let mut res = Vec::new();
    //             res.push((
    //                 "Point".to_string().into(),
    //                 user_types::Struct {
    //                     id: "Point".to_string().into(),
    //                     fields: vec![
    //                         ("x".to_string().into(), p_num!(I64)),
    //                         ("y".to_string().into(), p_num!(I64)),
    //                     ],
    //                 },
    //             ));
    //             res.push((
    //                 "Axe".to_string().into(),
    //                 user_types::Struct {
    //                     id: "Axe".to_string().into(),
    //                     fields: {
    //                         let mut res = Vec::new();
    //                         res.push(("x".to_string().into(), p_num!(I64)));
    //                         res
    //                     },
    //                 },
    //             ));
    //             res
    //         },
    //     };
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let geo = Geo::Point {
    //                 x : 420,
    //                 y: 69,
    //             };
    //             let z = 27;
    //             return match geo {
    //                 case Geo::Point {x,y} => x,
    //                 case Geo::Axe {x} => z,
    //             };
    //         };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager
    //         .register_type("Geo", UserType::Union(user_type), None)
    //         .expect("Registering of user type should have succeeded");
    //     let _ = statement
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = Program::default();
    //     statement
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 420));
    // }

    // #[test]
    // fn valid_match_enum() {
    //     let user_type = user_types::Enum {
    //         id: "Geo".to_string().into(),
    //         values: vec![
    //             "Point".to_string().into(),
    //             "Axe".to_string().into(),
    //             "Other".to_string().into(),
    //         ],
    //     };
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let geo = Geo::Point;
    //             let z = 27;
    //             return match geo {
    //                 case Geo::Point => 420,
    //                 case Geo::Axe => 69,
    //                 case Geo::Other => 69,
    //             };
    //         };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager
    //         .register_type("Geo", UserType::Enum(user_type), None)
    //         .expect("Registering of user type should have succeeded");
    //     let _ = statement
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = Program::default();
    //     statement
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 420));
    // }

    // #[test]
    // fn valid_match_enum_else() {
    //     let user_type = user_types::Enum {
    //         id: "Geo".to_string().into(),
    //         values: vec![
    //             "Point".to_string().into(),
    //             "Axe".to_string().into(),
    //             "Other".to_string().into(),
    //         ],
    //     };
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let geo = Geo::Other;
    //             let z = 27;
    //             return match geo {
    //                 case Geo::Point => 420,
    //                 case Geo::Axe => 420,
    //                 else => 69,
    //             };
    //         };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager
    //         .register_type("Geo", UserType::Enum(user_type), None)
    //         .expect("Registering of user type should have succeeded");
    //     let _ = statement
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = Program::default();
    //     statement
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 69));
    // }

    // #[test]
    // fn valid_match_union_else() {
    //     let user_type = user_types::Union {
    //         id: "Geo".to_string().into(),
    //         variants: {
    //             let mut res = Vec::new();
    //             res.push((
    //                 "Point".to_string().into(),
    //                 user_types::Struct {
    //                     id: "Point".to_string().into(),
    //                     fields: vec![
    //                         ("x".to_string().into(), p_num!(I64)),
    //                         ("y".to_string().into(), p_num!(I64)),
    //                     ],
    //                 },
    //             ));
    //             res.push((
    //                 "Axe".to_string().into(),
    //                 user_types::Struct {
    //                     id: "Axe".to_string().into(),
    //                     fields: {
    //                         let mut res = Vec::new();
    //                         res.push(("x".to_string().into(), p_num!(I64)));
    //                         res
    //                     },
    //                 },
    //             ));
    //             res
    //         },
    //     };
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let geo = Geo::Point {
    //                 x : 420,
    //                 y: 69,
    //             };
    //             let z = 27;
    //             return match geo {
    //                 case Geo::Axe {x} => x,
    //                 else => z,
    //             };
    //         };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager
    //         .register_type("Geo", UserType::Union(user_type), None)
    //         .expect("Registering of user type should have succeeded");
    //     let _ = statement
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = Program::default();
    //     statement
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 27));
    // }

    // #[test]
    // fn valid_match_number() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = match 69 {
    //             case 69 => 420,
    //             else => 69
    //         };
    //     "##
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
    //     assert_eq!(result, v_num!(I64, 420));
    // }

    // #[test]
    // fn valid_match_number_else() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = match 420 {
    //             case 69 => 420,
    //             else => 69
    //         };
    //     "##
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
    //     assert_eq!(result, v_num!(I64, 69));
    // }

    // #[test]
    // fn valid_match_string() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = match "Hello world" {
    //             case "Hello world" => 420,
    //             else => 69
    //         };
    //     "##
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
    //     assert_eq!(result, v_num!(I64, 420));
    // }

    // #[test]
    // fn valid_match_string_else() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = match "CipherPool" {
    //             case "Hello world" => 420,
    //             else => 69
    //         };
    //     "##
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
    //     assert_eq!(result, v_num!(I64, 69));
    // }

    // #[test]
    // fn valid_match_multiple_case_strslice() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = match "CipherPool" {
    //             case "Hello world" | "CipherPool" => 420,
    //             else => 69
    //         };
    //     "##
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
    //     assert_eq!(result, v_num!(I64, 420));
    // }
    // #[test]
    // fn valid_match_multiple_case_num() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = match 500 {
    //             case 86 | 500 => 420,
    //             else => 69
    //         };
    //     "##
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
    //     assert_eq!(result, v_num!(I64, 420));
    // }

    // #[test]
    // fn valid_match_union_mult() {
    //     let user_type = user_types::Union {
    //         id: "Geo".to_string().into(),
    //         variants: {
    //             let mut res = Vec::new();
    //             res.push((
    //                 "Point".to_string().into(),
    //                 user_types::Struct {
    //                     id: "Point".to_string().into(),
    //                     fields: vec![("x".to_string().into(), p_num!(I64))],
    //                 },
    //             ));
    //             res.push((
    //                 "Axe".to_string().into(),
    //                 user_types::Struct {
    //                     id: "Axe".to_string().into(),
    //                     fields: vec![("x".to_string().into(), p_num!(I64))],
    //                 },
    //             ));
    //             res.push((
    //                 "Other".to_string().into(),
    //                 user_types::Struct {
    //                     id: "Axe".to_string().into(),
    //                     fields: vec![("x".to_string().into(), p_num!(I64))],
    //                 },
    //             ));
    //             res
    //         },
    //     };
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let geo = Geo::Point {
    //                 x : 420,
    //             };
    //             let z = 27;
    //             return match geo {
    //                 case Geo::Axe {x} | Geo::Point {x} => x,
    //                 else => z,
    //             };
    //         };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager
    //         .register_type("Geo", UserType::Union(user_type), None)
    //         .expect("Registering of user type should have succeeded");
    //     let _ = statement
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = Program::default();
    //     statement
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 420));
    // }

    // #[test]
    // fn valid_match_enum_mult() {
    //     let user_type = user_types::Enum {
    //         id: "Geo".to_string().into(),
    //         values: vec![
    //             "Point".to_string().into(),
    //             "Axe".to_string().into(),
    //             "Other".to_string().into(),
    //         ],
    //     };
    //     let mut statement = Statement::parse(
    //         r##"
    //         let x = {
    //             let geo = Geo::Axe;
    //             let z = 27;
    //             return match geo {
    //                 case Geo::Point | Geo::Axe => 420,
    //                 else => 69,
    //             };
    //         };
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager
    //         .register_type("Geo", UserType::Enum(user_type), None)
    //         .expect("Registering of user type should have succeeded");
    //     let _ = statement
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = Program::default();
    //     statement
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::I64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(I64, 420));
    // }

    // #[test]
    // fn valid_try_tuple() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let res = try (10,Ok()) else 20;
    //         return res;
    //     };

    //     "##
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
    // fn valid_try_tuple_else() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let res = try (10,Err()) else 20;
    //         return res;
    //     };

    //     "##
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
    //     assert_eq!(result, v_num!(U64, 20));
    // }

    // #[test]
    // fn valid_try_tuple_catch_err_string_access() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let buf = string("aaaabbbbcc");
    //         let res = try buf[16] else 'a';
    //         return res;
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(result, Primitive::Char('a'));
    // }

    // #[test]
    // fn valid_try_tuple_catch_err_strslice_access() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let buf = "aaaabbbbcc";
    //         let res = try buf[16] else 'a';
    //         return res;
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Char, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(result, Primitive::Char('a'));
    // }

    // #[test]
    // fn valid_try_tuple_catch_err_slice_access() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let buf = [1,5,3,8];
    //         let res = try buf[16] else 10;
    //         return res;
    //     };

    //     "##
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
    // fn valid_try_tuple_catch_err_vec_access() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let buf = vec[1,5,3,8];
    //         let res = try buf[16] else 10;
    //         return res;
    //     };

    //     "##
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
