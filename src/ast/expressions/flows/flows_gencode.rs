use num_traits::ToBytes;

use std::fmt::Debug;

use crate::ast::statements::block::BlockCommonApi;
use crate::ast::TryParse;
use crate::semantic::scope::scope::{ScopeState, VariableInfo};
use crate::semantic::scope::static_types::POINTER_SIZE;
use crate::semantic::Resolve;
use crate::vm::asm::branch::{BranchTry, CloseFrame, Return};
use crate::vm::asm::mem::Mem;
use crate::vm::asm::operation::{Equal, Operation, StrEqual};
use crate::vm::core::ERROR_VALUE;
use crate::vm::{CodeGenerationContext, CodeGenerationError, GenerateCode};
use crate::{
    semantic::{SizeOf, TypeOf},
    vm::asm::{
        branch::{BranchIf, Goto, Label},
        data::Data,
        Asm,
    },
};

use super::{EnumCase, ExprFlow, IfExpr, MatchExpr, PrimitiveCase, StringCase, TryExpr, UnionCase};

impl GenerateCode for ExprFlow {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            ExprFlow::If(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            ExprFlow::Match(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            ExprFlow::Try(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            ExprFlow::SizeOf(value, _metadata) => {
                let value = value
                    .type_of(scope_manager, scope_id)
                    .map_err(|_| CodeGenerationError::UnresolvedError)?;

                instructions.push(Asm::Data(Data::Serialized {
                    data: (value.size_of() as u64).to_le_bytes().into(),
                }));
                Ok(())
            }
        }
    }
}

impl GenerateCode for IfExpr {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let else_label = Label::gen();
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

        instructions.push_label_by_id(else_label, "else".to_string().into());
        let _ = self
            .else_branch
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;
        instructions.push(Asm::Goto(Goto {
            label: Some(end_label),
        }));

        instructions.push_label_by_id(end_label, "end_if".to_string().into());

        Ok(())
    }
}

impl<B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq> GenerateCode
    for PrimitiveCase<B>
{
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let block_label = Label::gen();
        let mut else_label = Label::gen();

        for (i, value) in self.patterns.iter().enumerate() {
            instructions.push_label_by_id(else_label, format!("case_{}", i));
            instructions.push(Asm::Mem(Mem::Dup(value.size_of())));

            value.gencode::<E>(scope_manager, scope_id, instructions, context)?;

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

        instructions.push_label_by_id(block_label, "match_block".to_string());
        self.block
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;
        instructions.push(Asm::Goto(Goto {
            label: context.break_label,
        }));
        instructions.push_label_by_id(else_label, "fallthrough".to_string());

        Ok(())
    }
}

impl<B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq> GenerateCode
    for StringCase<B>
{
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let block_label = Label::gen();
        let mut else_label = Label::gen();

        for (i, value) in self.patterns.iter().enumerate() {
            instructions.push_label_by_id(else_label, format!("case_{}", i));
            instructions.push(Asm::Mem(Mem::Dup(POINTER_SIZE)));

            value.gencode::<E>(scope_manager, scope_id, instructions, context)?;

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

        instructions.push_label_by_id(block_label, "match_block".to_string());
        self.block
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;
        instructions.push(Asm::Goto(Goto {
            label: context.break_label,
        }));
        instructions.push_label_by_id(else_label, "fallthrough".to_string());

        Ok(())
    }
}

impl<B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq> GenerateCode
    for EnumCase<B>
{
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let block_label = Label::gen();
        let mut else_label = Label::gen();

        for (i, (_, value)) in self.patterns.iter().enumerate() {
            instructions.push_label_by_id(else_label, format!("case_{}", i));
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

        instructions.push_label_by_id(block_label, "match_block".to_string());
        self.block
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;
        instructions.push(Asm::Goto(Goto {
            label: context.break_label,
        }));
        instructions.push_label_by_id(else_label, "fallthrough".to_string());

        Ok(())
    }
}

impl<B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq> GenerateCode
    for UnionCase<B>
{
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
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
        // clean up padding + variant_value
        instructions.push(Asm::Pop(
            POINTER_SIZE + self.pattern.variant_padding.unwrap_or(0),
        ));

        instructions.push_label_by_id(block_label, "match_block".to_string());

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
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;
        instructions.push(Asm::Goto(Goto {
            label: context.break_label,
        }));
        instructions.push_label_by_id(else_label, "fallthrough".to_string());
        Ok(())
    }
}

impl GenerateCode for MatchExpr {
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

impl GenerateCode for TryExpr {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
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

        let _ = self.try_branch.gencode::<E>(
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
            instructions.push_label_by_id(next, "else".to_string().into());
        } else {
        }

        instructions.push(Asm::Try(BranchTry::EndTry));

        instructions.push(Asm::Goto(Goto {
            label: Some(end_label),
        }));
        instructions.push_label_by_id(recover_else_label, "recover_else".to_string().into());

        if self.pop_last_err {
            if let Some(inner_scope) = self.try_branch.scope {
                match scope_manager.scope_states.get(&inner_scope) {
                    Some(ScopeState::IIFE) => {
                        // Push dummy data that will be returned
                        let mut dummy_data = vec![0; return_size];
                        dummy_data.push(ERROR_VALUE);

                        instructions.push(Asm::Data(Data::Serialized {
                            data: dummy_data.into(),
                        }));
                        // Once return the cursor will go back to (1)
                        instructions.push(Asm::Return(Return { size: return_size }));
                    }
                    Some(ScopeState::Inline) => {
                        instructions.push(Asm::Goto(Goto {
                            label: Some(else_label),
                        }));
                    }
                    _ => return Err(CodeGenerationError::UnresolvedError),
                }
            } else {
                return Err(CodeGenerationError::UnresolvedError);
            }
        } else {
            if let Some(inner_scope) = self.try_branch.scope {
                match scope_manager.scope_states.get(&inner_scope) {
                    Some(ScopeState::IIFE) => {
                        // Once return the cursor will go back to (1)
                        instructions.push(Asm::CloseFrame(CloseFrame));
                    }
                    Some(ScopeState::Inline) => {
                        instructions.push(Asm::Goto(Goto {
                            label: Some(else_label),
                        }));
                    }
                    _ => return Err(CodeGenerationError::UnresolvedError),
                }
            } else {
                return Err(CodeGenerationError::UnresolvedError);
            }
        }

        instructions.push_label_by_id(else_label, "else".to_string().into());
        instructions.push(Asm::Try(BranchTry::EndTry));

        if let Some(block) = &self.else_branch {
            block.gencode::<E>(scope_manager, scope_id, instructions, context)?;
        }

        instructions.push_label_by_id(end_label, "end_try".to_string().into());
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
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
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
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
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
                case Test::Point { x, y } => { y as u32 },
                case Test::Point2 { x, y } => { x },
                else => { 10u32 }
            };
            res4
        };
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_try() {
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 0);
            let res = test_extract_variable::<i64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3);
            let res = test_extract_variable::<i64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 0);
            let res = test_extract_variable::<i64>("res4", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3);
            let res = test_extract_variable::<i64>("res5", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 0);
            let res = test_extract_variable::<i64>("res6", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            true
        }

        test_statements(
            r##"

        let arr = [1,2,3];
        let res1 = try { arr[4] } else { 0 };
        let res2 = try { arr[2] } else { 0 };

        let res3 = try { 
            let x = 1;
            arr[4] 
        } else { 0 };

        let res4 = try { 
            let x = 2;
            arr[2] 
        } else { 0 };

         
        let res5 = try { (5,Err()) } else { 0 };
        let res6 = try { (5,Ok()) } else { 0 };

        "##,
            &mut engine,
            assert_fn,
        );
    }
}
