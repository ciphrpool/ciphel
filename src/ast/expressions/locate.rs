use crate::{
    semantic::{scope::static_types::StaticType, EType, SizeOf},
    vm::{
        allocator::MemoryAddress,
        casm::{
            alloc::Access,
            locate::{LocateIndex, LocateOffset},
            Casm,
        },
        vm::{CodeGenerationContext, CodeGenerationError, GenerateCode},
    },
};

use super::{
    data::{Address, Data, PtrAccess, Variable},
    operation::{FieldAccess, FnCall, ListAccess, TupleAccess},
    Atomic, Expression,
};

pub trait Locatable {
    fn locate(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError>;

    fn locate_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: Option<MemoryAddress>,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError>;

    fn access_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: MemoryAddress,
    ) -> Result<(), CodeGenerationError>;

    fn runtime_access(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<(), CodeGenerationError>;

    fn is_assignable(&self) -> bool;
}

impl Locatable for Variable {
    fn is_assignable(&self) -> bool {
        true
    }
    fn locate(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        let Some(super::data::VariableState::Variable { id }) = &self.state else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Ok(crate::semantic::scope::scope::Variable { address, .. }) =
            scope_manager.find_var_by_id(*id, scope_id)
        else {
            return Err(CodeGenerationError::Unlocatable);
        };
        let Ok(address) = (address.clone()).try_into() else {
            return Err(CodeGenerationError::Unlocatable);
        };
        return Ok(Some(address));
    }

    fn locate_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: Option<MemoryAddress>,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        let Some(super::data::VariableState::Field { offset, size }) = &self.state else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        match address {
            Some(address) => {
                // static address
                Ok(Some(address.add(*offset)))
            }
            None => {
                // the address was pushed on the stack
                instructions.push(Casm::Offset(LocateOffset { offset: *offset }));
                Ok(None)
            }
        }
    }

    fn access_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: MemoryAddress,
    ) -> Result<(), CodeGenerationError> {
        let Some(super::data::VariableState::Field { offset, size }) = &self.state else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        instructions.push(Casm::Access(Access::Static {
            address: address.add(*offset),
            size: *size,
        }));
        Ok(())
    }

    fn runtime_access(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(super::data::VariableState::Field { offset, size }) = &self.state else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        instructions.push(Casm::Offset(LocateOffset { offset: *offset }));
        instructions.push(Casm::Access(Access::Runtime { size: Some(*size) }));
        Ok(())
    }
}

impl Locatable for Address {
    fn is_assignable(&self) -> bool {
        false
    }
    fn locate(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        Err(CodeGenerationError::Unlocatable)
    }

    fn locate_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: Option<MemoryAddress>,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        Err(CodeGenerationError::Unlocatable)
    }

    fn access_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: MemoryAddress,
    ) -> Result<(), CodeGenerationError> {
        Err(CodeGenerationError::Unaccessible)
    }

    fn runtime_access(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        Err(CodeGenerationError::Unaccessible)
    }
}
impl Locatable for PtrAccess {
    fn is_assignable(&self) -> bool {
        true
    }
    fn locate(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        let _ = self.value.gencode(
            scope_manager,
            scope_id,
            instructions,
            &CodeGenerationContext::default(),
        )?;
        Ok(None)
    }

    fn locate_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: Option<MemoryAddress>,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        Err(CodeGenerationError::Unlocatable)
    }

    fn access_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: MemoryAddress,
    ) -> Result<(), CodeGenerationError> {
        Err(CodeGenerationError::Unaccessible)
    }

    fn runtime_access(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        Err(CodeGenerationError::Unaccessible)
    }
}
impl Locatable for FieldAccess {
    fn is_assignable(&self) -> bool {
        true
    }
    fn locate(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        match self.var.locate(scope_manager, scope_id, instructions)? {
            Some(address) => {
                // the address is static
                self.field
                    .locate_from(scope_manager, scope_id, instructions, Some(address))
            }
            None => {
                // the address was pushed on the stack
                self.field
                    .locate_from(scope_manager, scope_id, instructions, None)
            }
        }
    }

    fn locate_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: Option<MemoryAddress>,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        match self
            .var
            .locate_from(scope_manager, scope_id, instructions, address)?
        {
            Some(address) => {
                // the address is static
                self.field
                    .locate_from(scope_manager, scope_id, instructions, Some(address))
            }
            None => {
                // the address was pushed on the stack
                self.field
                    .locate_from(scope_manager, scope_id, instructions, None)
            }
        }
    }

    fn access_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: MemoryAddress,
    ) -> Result<(), CodeGenerationError> {
        match self
            .var
            .locate_from(scope_manager, scope_id, instructions, Some(address))?
        {
            Some(address) => {
                // the address is static
                self.field
                    .access_from(scope_manager, scope_id, instructions, address)?;
            }
            None => {
                // the address was pushed on the stack
                self.field
                    .runtime_access(scope_manager, scope_id, instructions)?;
            }
        }
        Ok(())
    }

    fn runtime_access(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self
            .var
            .locate_from(scope_manager, scope_id, instructions, None)?
        {
            Some(address) => {
                // the address is static
                self.field
                    .access_from(scope_manager, scope_id, instructions, address)?;
            }
            None => {
                // the address was pushed on the stack
                self.field
                    .runtime_access(scope_manager, scope_id, instructions)?;
            }
        }
        Ok(())
    }
}
impl Locatable for TupleAccess {
    fn is_assignable(&self) -> bool {
        true
    }
    fn locate(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        let Some(offset) = self.offset else {
            return Err(CodeGenerationError::Unlocatable);
        };

        match self.var.locate(scope_manager, scope_id, instructions)? {
            Some(address) => {
                // the address is static
                Ok(Some(address.add(offset)))
            }
            None => {
                // the address was pushed on the stack
                instructions.push(Casm::Offset(LocateOffset { offset }));
                Ok(None)
            }
        }
    }

    fn locate_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: Option<MemoryAddress>,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        let Some(offset) = self.offset else {
            return Err(CodeGenerationError::Unlocatable);
        };

        match self
            .var
            .locate_from(scope_manager, scope_id, instructions, address)?
        {
            Some(address) => {
                // the address is static
                Ok(Some(address.add(offset)))
            }
            None => {
                // the address was pushed on the stack
                instructions.push(Casm::Offset(LocateOffset { offset }));
                Ok(None)
            }
        }
    }

    fn access_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: MemoryAddress,
    ) -> Result<(), CodeGenerationError> {
        let Some(item_type) = self.metadata.signature() else {
            return Err(CodeGenerationError::Unlocatable);
        };

        let Some(offset) = self.offset else {
            return Err(CodeGenerationError::Unlocatable);
        };

        let size = item_type.size_of();

        match self
            .var
            .locate_from(scope_manager, scope_id, instructions, Some(address))?
        {
            Some(address) => {
                // the address is static
                instructions.push(Casm::Access(Access::Static {
                    address: address.add(offset),
                    size,
                }))
            }
            None => {
                // the address was pushed on the stack
                instructions.push(Casm::Offset(LocateOffset { offset }));
                instructions.push(Casm::Access(Access::Runtime { size: Some(size) }));
            }
        }
        Ok(())
    }

    fn runtime_access(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(item_type) = self.metadata.signature() else {
            return Err(CodeGenerationError::Unlocatable);
        };

        let Some(offset) = self.offset else {
            return Err(CodeGenerationError::Unlocatable);
        };

        let size = item_type.size_of();

        match self
            .var
            .locate_from(scope_manager, scope_id, instructions, None)?
        {
            Some(address) => {
                // the address is static
                instructions.push(Casm::Access(Access::Static {
                    address: address.add(offset),
                    size,
                }))
            }
            None => {
                // the address was pushed on the stack
                instructions.push(Casm::Offset(LocateOffset { offset }));
                instructions.push(Casm::Access(Access::Runtime { size: Some(size) }));
            }
        }
        Ok(())
    }
}

impl Locatable for ListAccess {
    fn is_assignable(&self) -> bool {
        true
    }
    fn locate(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        let Some(item_type) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let size = item_type.size_of();

        let Some(array_type) = self.var.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let offset = match array_type {
            EType::Static(StaticType::Vec(_)) => crate::vm::platform::core::core_vector::VEC_HEADER,
            EType::Static(StaticType::Slice(_)) => 0,
            _ => return Err(CodeGenerationError::UnresolvedError),
        };

        match self.var.locate(scope_manager, scope_id, instructions)? {
            Some(address) => {
                // the address is static
                let _ = self.index.gencode(
                    scope_manager,
                    scope_id,
                    instructions,
                    &CodeGenerationContext::default(),
                )?;

                instructions.push(Casm::OffsetIdx(LocateIndex {
                    size,
                    base_address: Some(address),
                    offset: Some(offset),
                }));
            }
            None => {
                // the address was pushed on the stack

                let _ = self.index.gencode(
                    scope_manager,
                    scope_id,
                    instructions,
                    &CodeGenerationContext::default(),
                )?;

                instructions.push(Casm::OffsetIdx(LocateIndex {
                    size,
                    base_address: None,
                    offset: Some(offset),
                }));
            }
        }
        Ok(None)
    }

    fn locate_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: Option<MemoryAddress>,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        let Some(item_type) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let size = item_type.size_of();

        let Some(array_type) = self.var.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let offset = match array_type {
            EType::Static(StaticType::Vec(_)) => crate::vm::platform::core::core_vector::VEC_HEADER,
            EType::Static(StaticType::Slice(_)) => 0,
            _ => return Err(CodeGenerationError::UnresolvedError),
        };

        match self
            .var
            .locate_from(scope_manager, scope_id, instructions, address)?
        {
            Some(address) => {
                // the address is static
                let _ = self.index.gencode(
                    scope_manager,
                    scope_id,
                    instructions,
                    &CodeGenerationContext::default(),
                )?;

                instructions.push(Casm::OffsetIdx(LocateIndex {
                    size,
                    base_address: Some(address),
                    offset: Some(offset),
                }));
            }
            None => {
                // the address was pushed on the stack

                let _ = self.index.gencode(
                    scope_manager,
                    scope_id,
                    instructions,
                    &CodeGenerationContext::default(),
                )?;

                instructions.push(Casm::OffsetIdx(LocateIndex {
                    size,
                    base_address: None,
                    offset: Some(offset),
                }));
            }
        }
        Ok(None)
    }

    fn access_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: MemoryAddress,
    ) -> Result<(), CodeGenerationError> {
        let Some(item_type) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let size = item_type.size_of();

        let Some(array_type) = self.var.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let offset = match array_type {
            EType::Static(StaticType::Vec(_)) => crate::vm::platform::core::core_vector::VEC_HEADER,
            EType::Static(StaticType::Slice(_)) => 0,
            _ => return Err(CodeGenerationError::UnresolvedError),
        };

        match self
            .var
            .locate_from(scope_manager, scope_id, instructions, Some(address))?
        {
            Some(address) => {
                // the address is static
                let _ = self.index.gencode(
                    scope_manager,
                    scope_id,
                    instructions,
                    &CodeGenerationContext::default(),
                )?;

                instructions.push(Casm::OffsetIdx(LocateIndex {
                    size,
                    base_address: Some(address),
                    offset: Some(offset),
                }));
                instructions.push(Casm::Access(Access::Runtime { size: Some(size) }));
            }
            None => {
                // the address was pushed on the stack

                let _ = self.index.gencode(
                    scope_manager,
                    scope_id,
                    instructions,
                    &CodeGenerationContext::default(),
                )?;

                instructions.push(Casm::OffsetIdx(LocateIndex {
                    size,
                    base_address: None,
                    offset: Some(offset),
                }));
                instructions.push(Casm::Access(Access::Runtime { size: Some(size) }));
            }
        }
        Ok(())
    }

    fn runtime_access(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let Some(item_type) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let size = item_type.size_of();

        let Some(array_type) = self.var.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let offset = match array_type {
            EType::Static(StaticType::Vec(_)) => crate::vm::platform::core::core_vector::VEC_HEADER,
            EType::Static(StaticType::Slice(_)) => 0,
            _ => return Err(CodeGenerationError::UnresolvedError),
        };

        match self
            .var
            .locate_from(scope_manager, scope_id, instructions, None)?
        {
            Some(address) => {
                // the address is static
                let _ = self.index.gencode(
                    scope_manager,
                    scope_id,
                    instructions,
                    &CodeGenerationContext::default(),
                )?;

                instructions.push(Casm::OffsetIdx(LocateIndex {
                    size,
                    base_address: Some(address),
                    offset: Some(offset),
                }));
                instructions.push(Casm::Access(Access::Runtime { size: Some(size) }));
            }
            None => {
                // the address was pushed on the stack

                let _ = self.index.gencode(
                    scope_manager,
                    scope_id,
                    instructions,
                    &CodeGenerationContext::default(),
                )?;

                instructions.push(Casm::OffsetIdx(LocateIndex {
                    size,
                    base_address: None,
                    offset: Some(offset),
                }));
                instructions.push(Casm::Access(Access::Runtime { size: Some(size) }));
            }
        }
        Ok(())
    }
}

impl Locatable for FnCall {
    fn is_assignable(&self) -> bool {
        false
    }
    fn locate(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        todo!()
    }

    fn locate_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: Option<MemoryAddress>,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        todo!()
    }

    fn access_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: MemoryAddress,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }

    fn runtime_access(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl Locatable for Expression {
    fn is_assignable(&self) -> bool {
        match self {
            Expression::FieldAccess(value) => value.is_assignable(),
            Expression::FnCall(value) => value.is_assignable(),
            Expression::ListAccess(value) => value.is_assignable(),
            Expression::TupleAccess(value) => value.is_assignable(),
            Expression::Atomic(value) => value.is_assignable(),
            _ => false,
        }
    }
    fn locate(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        match self {
            Expression::FieldAccess(value) => value.locate(scope_manager, scope_id, instructions),
            Expression::FnCall(value) => value.locate(scope_manager, scope_id, instructions),
            Expression::ListAccess(value) => value.locate(scope_manager, scope_id, instructions),
            Expression::TupleAccess(value) => value.locate(scope_manager, scope_id, instructions),
            Expression::Atomic(value) => value.locate(scope_manager, scope_id, instructions),
            _ => Err(CodeGenerationError::Unlocatable),
        }
    }

    fn locate_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: Option<MemoryAddress>,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        match self {
            Expression::FieldAccess(value) => {
                value.locate_from(scope_manager, scope_id, instructions, address)
            }
            Expression::FnCall(value) => {
                value.locate_from(scope_manager, scope_id, instructions, address)
            }
            Expression::ListAccess(value) => {
                value.locate_from(scope_manager, scope_id, instructions, address)
            }
            Expression::TupleAccess(value) => {
                value.locate_from(scope_manager, scope_id, instructions, address)
            }
            Expression::Atomic(value) => {
                value.locate_from(scope_manager, scope_id, instructions, address)
            }
            _ => Err(CodeGenerationError::Unlocatable),
        }
    }

    fn access_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: MemoryAddress,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Expression::FieldAccess(value) => {
                value.access_from(scope_manager, scope_id, instructions, address)
            }
            Expression::FnCall(value) => {
                value.access_from(scope_manager, scope_id, instructions, address)
            }
            Expression::ListAccess(value) => {
                value.access_from(scope_manager, scope_id, instructions, address)
            }
            Expression::TupleAccess(value) => {
                value.access_from(scope_manager, scope_id, instructions, address)
            }
            Expression::Atomic(value) => {
                value.access_from(scope_manager, scope_id, instructions, address)
            }
            _ => Err(CodeGenerationError::Unlocatable),
        }
    }

    fn runtime_access(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Expression::FieldAccess(value) => {
                value.runtime_access(scope_manager, scope_id, instructions)
            }
            Expression::FnCall(value) => {
                value.runtime_access(scope_manager, scope_id, instructions)
            }
            Expression::ListAccess(value) => {
                value.runtime_access(scope_manager, scope_id, instructions)
            }
            Expression::TupleAccess(value) => {
                value.runtime_access(scope_manager, scope_id, instructions)
            }
            Expression::Atomic(value) => {
                value.runtime_access(scope_manager, scope_id, instructions)
            }
            _ => Err(CodeGenerationError::Unlocatable),
        }
    }
}
impl Locatable for Atomic {
    fn is_assignable(&self) -> bool {
        match self {
            Atomic::Data(value) => value.is_assignable(),
            Atomic::Paren(value) => value.is_assignable(),
            _ => false,
        }
    }
    fn locate(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        match self {
            Atomic::Data(value) => value.locate(scope_manager, scope_id, instructions),
            Atomic::Paren(value) => value.locate(scope_manager, scope_id, instructions),
            _ => Err(CodeGenerationError::Unlocatable),
        }
    }

    fn locate_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: Option<MemoryAddress>,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        match self {
            Atomic::Data(value) => {
                value.locate_from(scope_manager, scope_id, instructions, address)
            }
            Atomic::Paren(value) => {
                value.locate_from(scope_manager, scope_id, instructions, address)
            }
            _ => Err(CodeGenerationError::Unlocatable),
        }
    }

    fn access_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: MemoryAddress,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Atomic::Data(value) => {
                value.access_from(scope_manager, scope_id, instructions, address)
            }
            Atomic::Paren(value) => {
                value.access_from(scope_manager, scope_id, instructions, address)
            }
            _ => Err(CodeGenerationError::Unaccessible),
        }
    }

    fn runtime_access(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Atomic::Data(value) => value.runtime_access(scope_manager, scope_id, instructions),
            Atomic::Paren(value) => value.runtime_access(scope_manager, scope_id, instructions),
            _ => Err(CodeGenerationError::Unaccessible),
        }
    }
}
impl Locatable for Data {
    fn is_assignable(&self) -> bool {
        match self {
            Data::Address(value) => value.is_assignable(),
            Data::PtrAccess(value) => value.is_assignable(),
            Data::Variable(value) => value.is_assignable(),
            _ => false,
        }
    }
    fn locate(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        match self {
            Data::Address(value) => value.locate(scope_manager, scope_id, instructions),
            Data::PtrAccess(value) => value.locate(scope_manager, scope_id, instructions),
            Data::Variable(value) => value.locate(scope_manager, scope_id, instructions),
            _ => Err(CodeGenerationError::Unlocatable),
        }
    }

    fn locate_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: Option<MemoryAddress>,
    ) -> Result<Option<MemoryAddress>, CodeGenerationError> {
        match self {
            Data::Address(value) => value.locate(scope_manager, scope_id, instructions),
            Data::PtrAccess(value) => value.locate(scope_manager, scope_id, instructions),
            Data::Variable(value) => value.locate(scope_manager, scope_id, instructions),
            _ => Err(CodeGenerationError::Unlocatable),
        }
    }

    fn access_from(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
        address: MemoryAddress,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Data::Address(value) => {
                value.access_from(scope_manager, scope_id, instructions, address)
            }
            Data::PtrAccess(value) => {
                value.access_from(scope_manager, scope_id, instructions, address)
            }
            Data::Variable(value) => {
                value.access_from(scope_manager, scope_id, instructions, address)
            }
            _ => Err(CodeGenerationError::Unaccessible),
        }
    }

    fn runtime_access(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::casm::CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Data::Address(value) => value.runtime_access(scope_manager, scope_id, instructions),
            Data::PtrAccess(value) => value.runtime_access(scope_manager, scope_id, instructions),
            Data::Variable(value) => value.runtime_access(scope_manager, scope_id, instructions),
            _ => Err(CodeGenerationError::Unaccessible),
        }
    }
}
