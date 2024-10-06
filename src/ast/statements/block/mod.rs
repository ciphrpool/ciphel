use crate::semantic::{Metadata, SemanticError};

use super::Statement;

pub mod block_gencode;
pub mod block_parse;
pub mod block_resolve;
pub mod block_typeof;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub scope: Option<u128>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionBlock {
    pub statements: Vec<Statement>,
    pub scope: Option<u128>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosureBlock {
    pub statements: Vec<Statement>,
    pub scope: Option<u128>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LambdaBlock {
    pub statements: Vec<Statement>,
    pub scope: Option<u128>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprBlock {
    pub statements: Vec<Statement>,
    pub scope: Option<u128>,
    pub metadata: Metadata,
}

pub trait BlockCommonApi {
    fn init_from_parent(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        parent_scope_id: Option<u128>,
    ) -> Result<u128, SemanticError>;

    fn scope(&self) -> Option<u128>;
}

impl FunctionBlock {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self {
            metadata: Metadata::default(),
            statements,
            scope: None,
        }
    }
}
impl BlockCommonApi for FunctionBlock {
    fn init_from_parent(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        parent_scope_id: Option<u128>,
    ) -> Result<u128, SemanticError> {
        let inner_scope = scope_manager.spawn_allocating(parent_scope_id, true)?;
        self.scope = Some(inner_scope);
        Ok(inner_scope)
    }
    fn scope(&self) -> Option<u128> {
        self.scope
    }
}

impl ClosureBlock {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self {
            metadata: Metadata::default(),
            statements,
            scope: None,
        }
    }
}
impl BlockCommonApi for ClosureBlock {
    fn init_from_parent(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        parent_scope_id: Option<u128>,
    ) -> Result<u128, SemanticError> {
        let inner_scope = scope_manager.spawn_allocating(parent_scope_id, true)?;
        self.scope = Some(inner_scope);
        Ok(inner_scope)
    }
    fn scope(&self) -> Option<u128> {
        self.scope
    }
}

impl LambdaBlock {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self {
            metadata: Metadata::default(),
            statements,
            scope: None,
        }
    }
}
impl BlockCommonApi for LambdaBlock {
    fn init_from_parent(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        parent_scope_id: Option<u128>,
    ) -> Result<u128, SemanticError> {
        let inner_scope = scope_manager.spawn_allocating(parent_scope_id, true)?;
        self.scope = Some(inner_scope);
        Ok(inner_scope)
    }
    fn scope(&self) -> Option<u128> {
        self.scope
    }
}
impl ExprBlock {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self {
            metadata: Metadata::default(),
            statements,
            scope: None,
        }
    }
}
impl BlockCommonApi for ExprBlock {
    fn init_from_parent(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        parent_scope_id: Option<u128>,
    ) -> Result<u128, SemanticError> {
        let inner_scope = scope_manager.spawn_allocating(parent_scope_id, false)?;
        self.scope = Some(inner_scope);
        Ok(inner_scope)
    }
    fn scope(&self) -> Option<u128> {
        self.scope
    }
}

impl Block {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self {
            metadata: Metadata::default(),
            statements,
            scope: None,
        }
    }
}
impl BlockCommonApi for Block {
    fn init_from_parent(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        parent_scope_id: Option<u128>,
    ) -> Result<u128, SemanticError> {
        let inner_scope = scope_manager.spawn(parent_scope_id)?;
        self.scope = Some(inner_scope);
        Ok(inner_scope)
    }
    fn scope(&self) -> Option<u128> {
        self.scope
    }
}
