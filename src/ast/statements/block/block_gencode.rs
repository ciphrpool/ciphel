use crate::semantic::scope::scope_impl::Scope;


use crate::{
    semantic::{
        scope::{
            var_impl::{VarState},
        },
        MutRc, SizeOf,
    },
    vm::{
        allocator::{
            stack::{Offset},
        },
        casm::{
            alloc::{Alloc, StackFrame},
            branch::{Call, Goto, Label},
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::Block;

pub fn inner_block_gencode(
    scope: &MutRc<Scope>,
    block: &Block,
    param_size: Option<usize>,
    is_direct_loop: bool,
    instructions: &CasmProgram,
) -> Result<(), CodeGenerationError> {
    let scope_label = Label::gen();
    let end_scope_label = Label::gen();

    instructions.push(Casm::Goto(Goto {
        label: Some(end_scope_label),
    }));

    instructions.push_label_id(scope_label, "block".into());

    let _ = block.gencode(scope, &instructions)?;

    instructions.push_label_id(end_scope_label, "end_scope".into());
    instructions.push(Casm::Call(Call::From {
        label: scope_label,
        param_size: param_size.unwrap_or(0),
    }));
    instructions.push(Casm::StackFrame(StackFrame::Transfer { is_direct_loop }));

    Ok(())
}

impl GenerateCode for Block {
    fn gencode(
        &self,
        _scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let borrowed = self.inner_scope.borrow();
        let Some(borrowed_scope) = borrowed.as_ref() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let borrowed = borrowed_scope.as_ref().borrow();

        let _return_size = self.metadata.signature().map_or(0, |t| t.size_of());

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
                if var_size == 0 {
                    continue;
                }
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
        //             /* Assign the value of the caller to the caller reference inside the current block */
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
        instructions.push(Casm::StackFrame(StackFrame::Clean));
        Ok(())
    }
}
