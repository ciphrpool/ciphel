use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{
    ast::statements,
    semantic::{
        scope::{
            var_impl::{Var, VarState},
            ClosureState, ScopeApi,
        },
        MutRc, SizeOf,
    },
    vm::{
        allocator::{
            stack::{Offset, UReg},
            MemoryAddress,
        },
        casm::{
            alloc::{Access, Alloc, StackFrame},
            branch::{BranchIf, Call, Goto, Label},
            locate::Locate,
            memcopy::MemCopy,
            operation::{Equal, Operation, OperationKind},
            serialize::Serialized,
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode, RuntimeError},
    },
};

use super::Scope;

pub fn inner_scope_gencode<S: ScopeApi>(
    scope: &MutRc<S>,
    value: &Scope<S>,
    param_size: Option<usize>,
    is_direct_loop: bool,
    instructions: &CasmProgram,
) -> Result<(), CodeGenerationError> {
    let scope_label = Label::gen();
    let end_scope_label = Label::gen();

    instructions.push(Casm::Goto(Goto {
        label: Some(end_scope_label),
    }));

    instructions.push_label_id(scope_label, "scope".into());

    let _ = value.gencode(scope, &instructions)?;

    instructions.push_label_id(end_scope_label, "end_scope".into());
    instructions.push(Casm::Call(Call::From {
        label: scope_label,
        param_size: param_size.unwrap_or(0),
    }));
    instructions.push(Casm::StackFrame(StackFrame::Transfer { is_direct_loop }));

    Ok(())
}

impl<OuterScope: ScopeApi> GenerateCode<OuterScope> for Scope<OuterScope> {
    fn gencode(
        &self,
        _scope: &MutRc<OuterScope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let borrowed = self.inner_scope.borrow();
        let Some(borrowed_scope) = borrowed.as_ref() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let borrowed = borrowed_scope.as_ref().borrow();

        let return_size = self.metadata.signature().map_or(0, |t| t.size_of());

        // Parameter allocation if any
        let parameters = borrowed
            .vars()
            .filter(|(v, _)| v.state.get() == VarState::Parameter);

        let mut offset_idx = 0;

        for (var, offset) in parameters {
            let var = var.as_ref();
            let var_size = var.type_sig.size_of();
            // Already allocated
            offset.set(Offset::FP(offset_idx));
            offset_idx += var_size;
        }

        let mut offset_idx = 0;
        for (var, offset) in borrowed.vars() {
            let var = var.as_ref();
            let var_size = var.type_sig.size_of();

            if var.state.get() != VarState::Parameter {
                offset.set(Offset::FZ(offset_idx as isize));
                // Alloc and push heap address on stack
                instructions.push(Casm::Alloc(Alloc::Stack { size: var_size }));
                offset_idx += var_size;
            }
        }

        // if let Some(caller) = self.caller.as_ref().borrow().as_ref() {
        //     for (var, _) in borrowed.vars() {
        //         if var.id == caller.id {
        //             let (_, offset, level) = borrowed.access_var(&var.id)?;
        //             let (_, offset_parent, level_parent) =
        //                 borrowed.access_var_in_parent(&var.id)?;
        //             /* Assign the value of the caller to the caller reference inside the current scope */
        //             instructions.push(Casm::Access(Access::Static {
        //                 address: MemoryAddress::Stack {
        //                     offset: offset_parent,
        //                     level: level_parent,
        //                 },
        //                 size: 8,
        //             }));
        //             instructions.push(Casm::Locate(Locate {
        //                 address: MemoryAddress::Stack { offset, level },
        //             }));
        //             instructions.push(Casm::MemCopy(MemCopy::Take { size: 8 }));
        //             break;
        //         }
        //     }
        // }
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
