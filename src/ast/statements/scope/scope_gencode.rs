use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::statements,
    semantic::{
        scope::{var_impl::Var, ScopeApi},
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

        let vars = borrowed.inner_vars();

        let return_size = self.metadata.signature().map_or(0, |t| t.size_of());

        // Parameter allocation if any
        let mut env_vars = vars
            .iter()
            .filter(|v| v.is_captured.get().1)
            .collect::<Vec<&Rc<Var>>>();
        env_vars.sort_by_key(|v| v.is_captured.get().0);

        let mut parameters = vars
            .iter()
            .filter(|v| v.is_parameter.get().1)
            .collect::<Vec<&Rc<Var>>>();
        parameters.sort_by_key(|v| v.is_parameter.get().0);

        let mut offset_idx = 0;
        for var in env_vars {
            let var = var.as_ref();
            let var_size = var.type_sig.size_of();
            // Already allocated
            var.address.set(Some(Offset::FP(offset_idx)));
            offset_idx += var_size;
        }
        for var in parameters {
            let var = var.as_ref();
            let var_size = var.type_sig.size_of();
            // Already allocated
            var.address.set(Some(Offset::FP(offset_idx)));
            offset_idx += var_size;
        }

        let mut offset_idx = 0;
        for var in vars {
            let var = var.as_ref();
            let var_size = var.type_sig.size_of();
            if !var.is_parameter.get().1 && !var.is_captured.get().1 {
                var.address.set(Some(Offset::FZ(offset_idx as isize)));
                // Alloc and push heap address on stack
                instructions.push(Casm::Alloc(Alloc::Stack { size: var_size }));
            }
            offset_idx += var_size;
        }

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
