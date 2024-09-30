use crate::vm::asm::branch::BranchIf;

use crate::vm::asm::{
    branch::{Goto, Label},
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

    use crate::{test_extract_variable, test_statements};

    #[test]
    fn valid_for() {
        let mut engine = crate::vm::external::test::NoopEngine {};

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
        let mut engine = crate::vm::external::test::NoopEngine {};

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
        let mut engine = crate::vm::external::test::NoopEngine {};

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
}
