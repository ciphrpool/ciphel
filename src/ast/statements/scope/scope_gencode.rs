use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{
    ast::statements,
    semantic::{
        scope::{
            var_impl::{Var, VarState},
            ScopeApi,
        },
        MutRc, SizeOf,
    },
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
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let borrowed = self.inner_scope.borrow();
        let Some(borrowed_scope) = borrowed.as_ref() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let borrowed = borrowed_scope.as_ref().borrow();

        let return_size = self.metadata.signature().map_or(0, |t| t.size_of());

        // Parameter allocation if any

        let binding = borrowed.vars();
        let parameters = binding
            .iter()
            .filter(|(v, _)| v.state.get() == VarState::Parameter);

        let mut offset_idx = 0;

        if borrowed.state().is_closure {
            offset_idx = 8;
        }

        for (var, offset) in parameters {
            let var = var.as_ref();
            let var_size = var.type_sig.size_of();
            // Already allocated
            offset.set(Offset::FP(offset_idx));
            // let _ = scope
            //     .borrow()
            //     .update_var_offset(&var.id, Offset::FP(offset_idx))
            //     .map_err(|_| CodeGenerationError::UnresolvedError)?;
            offset.set(Offset::FP(offset_idx));
            offset_idx += var_size;
        }

        let mut offset_idx = 0;
        for (var, offset) in borrowed.vars() {
            let var = var.as_ref();
            let var_size = var.type_sig.size_of();

            if var.state.get() != VarState::Parameter {
                dbg!("here");
                // let _ = scope
                //     .borrow()
                //     .update_var_offset(&var.id, Offset::FZ(offset_idx as isize))
                //     .map_err(|_| CodeGenerationError::UnresolvedError)?;
                offset.set(Offset::FZ(offset_idx as isize));
                // Alloc and push heap address on stack
                instructions.push(Casm::Alloc(Alloc::Stack { size: var_size }));
            }
            offset_idx += var_size;
        }

        drop(borrowed);

        let inner_scope = self.inner_scope.borrow();
        let Some(inner_scope) = inner_scope.as_ref() else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        for statement in &self.instructions {
            let _ = statement.gencode(inner_scope, instructions)?;
        }

        // End Stack Frame
        if return_size == 0 {
            instructions.push(Casm::StackFrame(StackFrame::Clean));
        }
        Ok(())
    }
}
