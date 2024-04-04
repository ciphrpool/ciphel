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
    let break_label = Label::gen();
    let continue_label = Label::gen();
    let no_return_label = Label::gen();
    // let end_no_return_label = Label::gen();

    let scope_is_in_loop = value
        .scope()
        .map_err(|_| CodeGenerationError::UnresolvedError)?
        .as_ref()
        .borrow()
        .state()
        .is_loop;
    if scope_is_in_loop {
        instructions.push(Casm::MemCopy(MemCopy::Dup(1)));

        instructions.push(Casm::Serialize(Serialized { data: vec![1u8] }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Equal(Equal { left: 1, right: 1 }),
        }));
        instructions.push(Casm::If(BranchIf {
            else_label: break_label,
        }));
    } else {
        instructions.push(Casm::If(BranchIf {
            else_label: no_return_label,
        }));
    }
    if scope_is_in_loop {
        instructions.push(Casm::Pop(1)); /* Pop the unused dup flag*/
    }
    instructions.push(Casm::StackFrame(StackFrame::Return { return_size: None }));
    // instructions.push(Casm::Goto(Goto {
    //     label: Some(end_no_return_label),
    // }));
    if scope_is_in_loop {
        instructions.push_label_id(break_label, "break_label".into());

        instructions.push(Casm::MemCopy(MemCopy::Dup(1)));

        instructions.push(Casm::Serialize(Serialized { data: vec![2u8] }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Equal(Equal { left: 1, right: 1 }),
        }));
        instructions.push(Casm::If(BranchIf {
            else_label: continue_label,
        }));
        if !is_direct_loop {
            instructions.push(Casm::StackFrame(StackFrame::Break));
        } else {
            instructions.push(Casm::Pop(9)); /* Pop the unused return size*/
            instructions.push(Casm::MemCopy(MemCopy::GetReg(UReg::R4)));
            instructions.push(Casm::Goto(Goto { label: None }));
        }

        instructions.push_label_id(continue_label, "continue_label".into());

        instructions.push(Casm::Serialize(Serialized { data: vec![3u8] }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Equal(Equal { left: 1, right: 1 }),
        }));
        instructions.push(Casm::If(BranchIf {
            else_label: no_return_label,
        }));
        if !is_direct_loop {
            instructions.push(Casm::StackFrame(StackFrame::Continue));
        } else {
            instructions.push(Casm::Pop(8)); /* Pop the unused return size*/
            instructions.push(Casm::MemCopy(MemCopy::GetReg(UReg::R3)));
            instructions.push(Casm::Goto(Goto { label: None }));
        }
    }
    instructions.push_label_id(no_return_label, "no_return_label".into());
    instructions.push(Casm::Pop(8)); /* Pop the unused return size*/
    // instructions.push_label_id(end_no_return_label, "end_no_return_label".into());

    Ok(())
}

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
