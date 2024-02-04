use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::statements,
    semantic::{scope::ScopeApi, SizeOf},
    vm::{
        strips::{alloc::Alloc, Strip},
        vm::{CodeGenerationError, GenerateCode, RuntimeError},
    },
};

use super::Scope;

impl<OuterScope: ScopeApi> GenerateCode<OuterScope> for Scope<OuterScope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<OuterScope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        let borrowed = self.inner_scope.borrow();
        let Some(borrowed_scope) = borrowed.as_ref() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let borrowed = borrowed_scope.as_ref().borrow();
        let vars = borrowed.inner_vars();
        let mut offset = offset;
        let mut borrowed_instructions = instructions.as_ref().borrow_mut();
        for var in vars {
            let var = var.as_ref();
            let var_size = var.type_sig.size_of();
            var.address.set(Some(offset));
            // Alloc and push heap address on stack
            borrowed_instructions.push(Strip::Alloc(Alloc::Stack { size: var_size }));
            offset += var_size;
        }
        drop(borrowed_instructions);

        let inner_scope = self.inner_scope.borrow();
        let Some(inner_scope) = inner_scope.as_ref() else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        for statement in &self.instructions {
            let _ = statement.gencode(inner_scope, instructions, offset)?;
        }

        Ok(())
    }
}
