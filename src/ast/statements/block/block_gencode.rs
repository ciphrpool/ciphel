use crate::semantic::scope::scope::{ScopeManager, Variable, VariableAddress};
use crate::vm::casm::branch::{BranchTry, Return};
use crate::vm::vm::CodeGenerationContext;
use crate::{
    semantic::SizeOf,
    vm::{
        casm::{
            alloc::Alloc,
            branch::{Call, Goto, Label},
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{Block, ClosureBlock, ExprBlock, FunctionBlock};

impl GenerateCode for Block {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        for statement in &self.statements {
            let _ = statement.gencode(scope_manager, scope_id, instructions, context)?;
        }
        Ok(())
    }
}

impl GenerateCode for FunctionBlock {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(inner_scope) = self.scope else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(return_type) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let return_size = return_type.size_of();

        let epilog_label = Label::gen();

        let mut local_offset = 0;
        // Allocate all parameter variable
        for Variable {
            ref ctype, address, ..
        } in scope_manager.iter_mut_on_parameters(inner_scope)
        {
            *address = VariableAddress::Local(local_offset);
            let size = ctype.size_of();
            local_offset += size;
            instructions.push(Casm::Alloc(Alloc::Stack { size }));
        }

        // Allocate all local variable
        for Variable {
            ref ctype, address, ..
        } in scope_manager.iter_mut_on_local_variable(inner_scope)
        {
            *address = VariableAddress::Local(local_offset);
            let size = ctype.size_of();
            local_offset += size;
            instructions.push(Casm::Alloc(Alloc::Stack { size }));
        }

        // generate code for all statements
        for statement in &self.statements {
            let _ = statement.gencode(
                scope_manager,
                scope_id,
                instructions,
                &CodeGenerationContext {
                    return_label: Some(epilog_label),
                    break_label: None,
                    continue_label: None,
                },
            )?;
        }

        // Function epilog
        instructions.push_label_id(epilog_label, "epilog_Function".to_string());
        instructions.push(Casm::Return(Return { size: return_size }));

        Ok(())
    }
}

impl GenerateCode for ClosureBlock {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(inner_scope) = self.scope else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(return_type) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let return_size = return_type.size_of();

        let epilog_label = Label::gen();

        let mut local_offset = 0;
        // Allocate all parameter variable
        for Variable {
            ref ctype, address, ..
        } in scope_manager.iter_mut_on_parameters(inner_scope)
        {
            *address = VariableAddress::Local(local_offset);
            let size = ctype.size_of();
            local_offset += size;
            instructions.push(Casm::Alloc(Alloc::Stack { size }));
        }

        // Allocate all local variable
        for Variable {
            ref ctype, address, ..
        } in scope_manager.iter_mut_on_local_variable(inner_scope)
        {
            *address = VariableAddress::Local(local_offset);
            let size = ctype.size_of();
            local_offset += size;
            instructions.push(Casm::Alloc(Alloc::Stack { size }));
        }

        // generate code for all statements
        for statement in &self.statements {
            let _ = statement.gencode(
                scope_manager,
                scope_id,
                instructions,
                &CodeGenerationContext {
                    return_label: Some(epilog_label),
                    break_label: None,
                    continue_label: None,
                },
            )?;
        }

        // Function epilog
        instructions.push_label_id(epilog_label, "epilog_Function".to_string());
        instructions.push(Casm::Return(Return { size: return_size }));

        Ok(())
    }
}

impl GenerateCode for ExprBlock {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(inner_scope) = self.scope else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(return_type) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let return_size = return_type.size_of();

        let call_label = Label::gen();
        let epilog_label = Label::gen();
        let start_iife = Label::gen();
        let end_iife = Label::gen();

        instructions.push(Casm::Goto(Goto {
            label: Some(call_label),
        }));
        instructions.push_label_id(start_iife, "start_IIFE".to_string());

        // Allocate all local variable
        let mut local_offset = 0;
        for Variable {
            ref ctype, address, ..
        } in scope_manager.iter_mut_on_local_variable(inner_scope)
        {
            *address = VariableAddress::Local(local_offset);
            let size = ctype.size_of();
            local_offset += size;
            instructions.push(Casm::Alloc(Alloc::Stack { size }));
        }

        // generate code for all statements
        for statement in &self.statements {
            let _ = statement.gencode(
                scope_manager,
                scope_id,
                instructions,
                &CodeGenerationContext {
                    return_label: Some(epilog_label),
                    break_label: None,
                    continue_label: None,
                },
            )?;
        }

        // IIFE epilog
        instructions.push_label_id(epilog_label, "epilog_IIFE".to_string());
        instructions.push(Casm::Return(Return { size: return_size }));
        instructions.push(Casm::Goto(Goto {
            label: Some(end_iife),
        }));
        instructions.push_label_id(start_iife, "call_IIFE".to_string());
        instructions.push(Casm::Call(Call::From {
            label: start_iife,
            param_size: 0,
        }));
        instructions.push_label_id(end_iife, "end_IIFE".to_string());

        Ok(())
    }
}
