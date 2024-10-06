use crate::semantic::scope::scope::Scope;
use crate::{arw_read, arw_write};

use crate::vm::casm::branch::BranchTry;
use crate::{
    semantic::{scope::var_impl::VarState, SizeOf},
    vm::{
        allocator::stack::Offset,
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
    scope: &crate::semantic::ArcRwLock<Scope>,
    block: &Block,
    param_size: Option<usize>,
    is_direct_loop: bool,
    is_try_block: bool,
    instructions: &mut CasmProgram,
) -> Result<(), CodeGenerationError> {
    let scope_label = Label::gen();
    let end_scope_label = Label::gen();

    instructions.push(Casm::Goto(Goto {
        label: Some(end_scope_label),
    }));

    instructions.push_label_id(scope_label, "block".to_string().into());

    let _ = block.gencode(scope, instructions)?;

    instructions.push_label_id(end_scope_label, "end_scope".to_string().into());
    instructions.push(Casm::Call(Call::From {
        label: scope_label,
        param_size: param_size.unwrap_or(0),
    }));
    if is_try_block {
        instructions.push(Casm::Try(BranchTry::EndTry));
    }
    instructions.push(Casm::StackFrame(StackFrame::Transfer { is_direct_loop }));

    Ok(())
}

impl GenerateCode for Block {
    fn gencode(
        &self,
        _scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let borrowed = &self.inner_scope;
        let Some(borrowed_scope) = borrowed else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let borrowed = crate::arw_read!(borrowed_scope, CodeGenerationError::ConcurrencyError)?;

        let _return_size = self.metadata.signature().map_or(0, |t| t.size_of());

        // Parameter allocation if any
        let parameters = borrowed.vars().filter(|(v, _)| {
            arw_read!(v, CodeGenerationError::ConcurrencyError)
                .ok()
                .map(|v| v.state == VarState::Parameter)
                .unwrap_or(false)
        });

        let mut offset_idx = 0;

        for (var, offset) in parameters {
            let var = var.as_ref();
            let var_size = arw_read!(var, CodeGenerationError::ConcurrencyError)?
                .type_sig
                .size_of();
            // Already allocated
            let mut borrowed_offset = arw_write!(offset, CodeGenerationError::ConcurrencyError)?;
            *borrowed_offset = Offset::FP(offset_idx);
            offset_idx += var_size;
        }

        let mut offset_idx = 0;
        for (var, offset) in borrowed.vars() {
            let var = var.as_ref();
            let var_size = arw_read!(var, CodeGenerationError::ConcurrencyError)?
                .type_sig
                .size_of();

            if arw_read!(var, CodeGenerationError::ConcurrencyError)?.state != VarState::Parameter {
                let mut borrowed_offset =
                    arw_write!(offset, CodeGenerationError::ConcurrencyError)?;
                *borrowed_offset = Offset::FZ(offset_idx as isize);
                // Alloc and push heap address on stack
                if var_size == 0 {
                    continue;
                }
                instructions.push(Casm::Alloc(Alloc::Stack { size: var_size }));
                offset_idx += var_size;
            }
        }
        drop(borrowed);

        let inner_scope = &self.inner_scope;
        let Some(inner_scope) = inner_scope else {
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
