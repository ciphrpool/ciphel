use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::statements,
    semantic::{scope::ScopeApi, MutRc, SizeOf},
    vm::{
        allocator::stack::Offset,
        casm::{
            alloc::{Alloc, StackFrame},
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode, RuntimeError},
    },
};

use super::Scope;

impl<OuterScope: ScopeApi> GenerateCode<OuterScope> for Scope<OuterScope> {
    fn gencode(
        &self,
        scope: &MutRc<OuterScope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        let borrowed = self.inner_scope.borrow();
        let Some(borrowed_scope) = borrowed.as_ref() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let borrowed = borrowed_scope.as_ref().borrow();

        let vars = borrowed.inner_vars();
        let mut offset_idx = 0;
        let mut borrowed_instructions = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;

        let return_size = self.metadata.signature().map_or(0, |t| t.size_of());
        // Start Stack Frame
        // borrowed_instructions.push(Casm::StackFrame(StackFrame::Set { return_size }));

        for var in vars {
            let var = var.as_ref();
            let var_size = var.type_sig.size_of();
            var.address.set(Some(Offset::FZ(offset_idx)));
            // Alloc and push heap address on stack
            borrowed_instructions.push(Casm::Alloc(Alloc::Stack { size: var_size }));
            offset_idx += (var_size as isize);
        }
        drop(borrowed_instructions);

        let inner_scope = self.inner_scope.borrow();
        let Some(inner_scope) = inner_scope.as_ref() else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        for statement in &self.instructions {
            let _ = statement.gencode(inner_scope, instructions)?;
        }

        let mut borrowed_instructions = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;
        // End Stack Frame
        if return_size == 0 {
            borrowed_instructions.push(Casm::StackFrame(StackFrame::Clean));
        }
        Ok(())
    }
}
