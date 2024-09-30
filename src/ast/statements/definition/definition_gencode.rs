use super::{Definition, FnDef, TypeDef};
use crate::semantic::scope::scope::VariableInfo;
use crate::vm::{CodeGenerationError, GenerateCode};

use crate::vm::asm::{
    branch::{Goto, Label},
    mem::Mem,
    Asm,
};

impl GenerateCode for Definition {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            Definition::Type(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Definition::Fn(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
        }
    }
}

impl GenerateCode for TypeDef {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        _scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        _instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        Ok(())
    }
}

impl GenerateCode for FnDef {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let function_label = Label::gen();
        let store_label = Label::gen();

        instructions.push(Asm::Goto(Goto {
            label: Some(store_label),
        }));
        instructions.push_label_by_id(function_label, format!("fn_{0}", self.name));
        self.scope
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;

        instructions.push_label_by_id(store_label, format!("store_fn_{0}", self.name));

        instructions.push(Asm::Mem(Mem::Label(function_label)));

        if let Some(scope_id) = scope_id {
            // LOCAL FUNCTION
            // store the function label as it is considered a variable
            let Some((id_external, id_internal, _)) = self.id else {
                return Err(CodeGenerationError::UnresolvedError);
            };
            let Ok(VariableInfo { address, .. }) = scope_manager.find_var_by_id(id_external) else {
                return Err(CodeGenerationError::UnresolvedError);
            };
            instructions.push(Asm::Mem(Mem::Store {
                size: 8,
                address: (*address)
                    .try_into()
                    .map_err(|_| CodeGenerationError::UnresolvedError)?,
            }));
        } else {
            // GLOBAL FUNCTION
            // allocate the function
            let Some((id_external, id_internal, _)) = self.id else {
                return Err(CodeGenerationError::UnresolvedError);
            };
            // store the function label as it is considered a variable
            let address = scope_manager.alloc_global_var_by_id(id_external)?;
            instructions.push(Asm::Mem(Mem::Store {
                size: 8,
                address: (address)
                    .try_into()
                    .map_err(|_| CodeGenerationError::UnresolvedError)?,
            }));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{test_extract_variable, test_statements};

    #[test]
    fn valid_fn() {
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            let res = test_extract_variable::<i64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            let res = test_extract_variable::<i64>("res5", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            true
        }

        test_statements(
            r##"
        fn test1() {
            let x = 5;
        }

        test1();

        fn test2() -> i64 {
            let x = 5;
            return x;
        }

        let res2 = test2();

        fn test3(x:i64) -> i64 {
            return x + 1;
        }
        let res3 = test3(4);

        fn test4(x:i64) {
            let y = x;
        }
        test4(5);

        fn test5(x:i64,y:i64) -> i64 {
            let z = x + y;
            return z;
        }
        let res5 = test5(2,3);

        "##,
            &mut engine,
            assert_fn,
        );
    }
}
