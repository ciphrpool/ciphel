use crate::semantic::scope::scope::{
    ScopeManager, ScopeState, Variable, VariableAddress, VariableInfo,
};
use crate::semantic::scope::static_types::POINTER_SIZE;
use crate::vm::asm::branch::{BranchTry, Return};
use crate::vm::vm::CodeGenerationContext;
use crate::{
    semantic::SizeOf,
    vm::{
        asm::{
            alloc::Alloc,
            branch::{Call, Goto, Label},
            Asm, Program,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{Block, ClosureBlock, ExprBlock, FunctionBlock, LambdaBlock};

impl GenerateCode for Block {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        for statement in &self.statements {
            let _ = statement.gencode(scope_manager, self.scope, instructions, context)?;
        }
        Ok(())
    }
}

impl GenerateCode for FunctionBlock {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
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

        // Allocate all parameter variable
        for (VariableInfo { address, .. }, offset) in
            scope_manager.iter_mut_on_parameters(inner_scope)
        {
            *address = VariableAddress::Local(offset);
        }

        // Allocate all local variable
        if let Some(allocating_size) = scope_manager
            .allocating_scope
            .get(&inner_scope)
            .map(|m| m.local_size)
        {
            instructions.push(Asm::Alloc(Alloc::Stack {
                size: allocating_size,
            }));
        }
        for (VariableInfo { address, .. }, offset) in
            scope_manager.iter_mut_on_local_variable(inner_scope)
        {
            *address = VariableAddress::Local(offset);
        }

        // generate code for all statements
        for statement in &self.statements {
            let _ = statement.gencode(
                scope_manager,
                self.scope,
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
        instructions.push(Asm::Return(Return { size: return_size }));

        Ok(())
    }
}

impl GenerateCode for ClosureBlock {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
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

        // Allocate all parameter variable
        for (VariableInfo { address, .. }, offset) in
            scope_manager.iter_mut_on_parameters(inner_scope)
        {
            *address = VariableAddress::Local(offset);
        }

        // Allocate all local variable
        if let Some(allocating_size) = scope_manager
            .allocating_scope
            .get(&inner_scope)
            .map(|m| m.local_size)
        {
            instructions.push(Asm::Alloc(Alloc::Stack {
                size: allocating_size,
            }));
        }
        for (VariableInfo { address, .. }, offset) in
            scope_manager.iter_mut_on_local_variable(inner_scope)
        {
            *address = VariableAddress::Local(offset);
        }

        // generate code for all statements
        for statement in &self.statements {
            let _ = statement.gencode(
                scope_manager,
                self.scope,
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
        instructions.push(Asm::Return(Return { size: return_size }));

        Ok(())
    }
}

impl GenerateCode for LambdaBlock {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
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

        // Allocate all parameter variable
        for (VariableInfo { address, .. }, offset) in
            scope_manager.iter_mut_on_parameters(inner_scope)
        {
            *address = VariableAddress::Local(offset);
        }

        // Allocate all local variable
        if let Some(allocating_size) = scope_manager
            .allocating_scope
            .get(&inner_scope)
            .map(|m| m.local_size)
        {
            instructions.push(Asm::Alloc(Alloc::Stack {
                size: allocating_size,
            }));
        }
        for (VariableInfo { address, .. }, offset) in
            scope_manager.iter_mut_on_local_variable(inner_scope)
        {
            *address = VariableAddress::Local(offset);
        }

        // generate code for all statements
        for statement in &self.statements {
            let _ = statement.gencode(
                scope_manager,
                self.scope,
                instructions,
                &CodeGenerationContext {
                    return_label: Some(epilog_label),
                    break_label: None,
                    continue_label: None,
                },
            )?;
        }

        // lambda epilog
        instructions.push_label_id(epilog_label, "epilog_lambda".to_string());
        instructions.push(Asm::Return(Return { size: return_size }));

        Ok(())
    }
}

impl GenerateCode for ExprBlock {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(inner_scope) = self.scope else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(return_type) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let scope_state = scope_manager
            .scope_states
            .get(&inner_scope)
            .cloned()
            .unwrap_or(ScopeState::Default);

        let return_size = return_type.size_of();

        let call_label = Label::gen();
        let epilog_label = Label::gen();
        let start_iife = Label::gen();
        let end_iife = Label::gen();

        if ScopeState::IIFE == scope_state {
            instructions.push(Asm::Goto(Goto {
                label: Some(call_label),
            }));
            instructions.push_label_id(start_iife, "start_IIFE".to_string());
        }
        let mut param_size = None;
        if ScopeState::IIFE == scope_state {
            // Allocate all local variable

            if let Some(allocating_size) = scope_manager
                .allocating_scope
                .get(&inner_scope)
                .map(|m| m.local_size)
            {
                instructions.push(Asm::Alloc(Alloc::Stack {
                    size: allocating_size,
                }));
            }
            if let Some(allocating_size) = scope_manager
                .allocating_scope
                .get(&inner_scope)
                .map(|m| m.param_size)
            {
                param_size = Some(allocating_size);
            }
            for (VariableInfo { address, .. }, offset) in
                scope_manager.iter_mut_on_parameters(inner_scope)
            {
                *address = VariableAddress::Local(offset);
            }

            for (
                VariableInfo {
                    ref ctype, address, ..
                },
                offset,
            ) in scope_manager.iter_mut_on_local_variable(inner_scope)
            {
                *address = VariableAddress::Local(offset);
            }
        }

        // generate code for all statements
        for statement in &self.statements {
            let _ = statement.gencode(
                scope_manager,
                self.scope,
                instructions,
                &CodeGenerationContext {
                    return_label: Some(epilog_label),
                    break_label: None,
                    continue_label: None,
                },
            )?;
        }

        if ScopeState::IIFE == scope_state {
            // IIFE epilog
            instructions.push_label_id(epilog_label, "epilog_IIFE".to_string());
            instructions.push(Asm::Return(Return { size: return_size }));
            instructions.push(Asm::Goto(Goto {
                label: Some(end_iife),
            }));
            instructions.push_label_id(call_label, "call_IIFE".to_string());
            instructions.push(Asm::Call(Call::From {
                label: start_iife,
                param_size: param_size.unwrap_or(0),
            }));
            instructions.push_label_id(end_iife, "end_IIFE".to_string());
        } else {
            instructions.push_label_id(epilog_label, "epilog_block".to_string());
        }
        Ok(())
    }
}
