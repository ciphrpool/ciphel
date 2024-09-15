use std::{
    any::Any,
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use nom::branch;
use ulid::Ulid;

use crate::{
    semantic::{EType, SemanticError, SizeOf},
    vm::{
        allocator::MemoryAddress,
        vm::{CodeGenerationContext, CodeGenerationError},
    },
    CompilationError,
};

use super::user_type_impl::UserType;

#[derive(Debug, Clone, Default, PartialEq, Copy)]
pub enum VariableAddress {
    Global(usize),
    Local(usize),
    #[default]
    Unallocated,
}

impl TryInto<usize> for VariableAddress {
    type Error = ();
    fn try_into(self) -> Result<usize, Self::Error> {
        match self {
            VariableAddress::Global(_) => todo!(),
            VariableAddress::Local(_) => todo!(),
            VariableAddress::Unallocated => Err(()),
        }
    }
}

impl TryInto<MemoryAddress> for VariableAddress {
    type Error = ();
    fn try_into(self) -> Result<MemoryAddress, Self::Error> {
        match self {
            VariableAddress::Global(_) => todo!(),
            VariableAddress::Local(_) => todo!(),
            VariableAddress::Unallocated => Err(()),
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub enum VariableState {
    #[default]
    Global,
    Local,
    Parameter,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub id: u64,
    pub ctype: EType,
    pub scope: Option<u128>,
    pub address: VariableAddress,
    pub state: VariableState,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub id: u64,
    pub def: UserType,
    pub scope: Option<u128>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ScopeState {
    Function,
    Inligned,
    IIFE,
    Loop,
    #[default]
    Default,
}

#[derive(Debug, Clone)]
pub struct GlobalMapping {
    pub top: usize,
}

impl Default for GlobalMapping {
    fn default() -> Self {
        Self { top: 0 }
    }
}

impl GlobalMapping {
    pub fn alloc(&mut self, size: usize) -> VariableAddress {
        let variable = VariableAddress::Global(self.top);
        self.top += size;
        variable
    }
}

#[derive(Debug, Clone)]
pub struct ScopeManager {
    vars: Vec<Variable>,
    types: Vec<Type>,
    transaction_vars_idx: Option<usize>, // Index of the last pushed variable
    transaction_types_idx: Option<usize>, // Index of the last pushed types
    scope_branches: HashMap<u128, Vec<u128>>,
    pub scope_types: HashMap<u128, Vec<EType>>,
    pub scope_states: HashMap<u128, ScopeState>,
    pub global_mapping: GlobalMapping,
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self {
            scope_branches: HashMap::default(),
            transaction_types_idx: None,
            transaction_vars_idx: None,
            types: Vec::default(),
            vars: Vec::default(),
            scope_types: HashMap::default(),
            scope_states: HashMap::default(),
            global_mapping: GlobalMapping::default(),
        }
    }
}

impl ScopeManager {
    pub fn open_transaction(&mut self) -> Result<(), CompilationError> {
        self.transaction_vars_idx = Some(self.vars.len());
        self.transaction_types_idx = Some(self.types.len());
        Ok(())
    }
    pub fn commit_transaction(&mut self) -> Result<(), CompilationError> {
        self.transaction_vars_idx = None;
        self.transaction_types_idx = None;
        Ok(())
    }
    pub fn reject_transaction(&mut self) -> Result<(), CompilationError> {
        self.vars
            .truncate(self.transaction_vars_idx.unwrap_or(self.vars.len()));
        self.types
            .truncate(self.transaction_types_idx.unwrap_or(self.types.len()));
        self.transaction_vars_idx = None;
        self.transaction_types_idx = None;
        Ok(())
    }

    pub fn spawn(&mut self, parent: Option<u128>) -> Result<u128, SemanticError> {
        let scope_id = Ulid::new().0;
        if let Some(parent) = parent {
            let Some(parent_branch) = self.scope_branches.get(&parent) else {
                return Err(SemanticError::Default);
            };
            let mut branch = parent_branch.clone();
            branch.push(scope_id);
            self.scope_branches.insert(scope_id, branch);
        } else {
            self.scope_branches.insert(scope_id, vec![scope_id]);
        }
        Ok(scope_id)
    }

    pub fn hash_id(name: &str, scope: Option<u128>) -> u64 {
        let mut hasher = DefaultHasher::default();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        hash
    }

    pub fn register_var(
        &mut self,
        name: &str,
        ctype: EType,
        scope: Option<u128>,
    ) -> Result<u64, SemanticError> {
        let var_id = ScopeManager::hash_id(name, scope);
        self.vars.push(Variable {
            id: var_id,
            ctype,
            scope,
            address: VariableAddress::default(),
            state: scope.map_or(VariableState::Global, |_| VariableState::Local),
        });
        Ok(var_id)
    }

    pub fn register_parameter(
        &mut self,
        name: &str,
        ctype: EType,
        scope: Option<u128>,
    ) -> Result<u64, SemanticError> {
        let var_id = ScopeManager::hash_id(name, scope);
        self.vars.push(Variable {
            id: var_id,
            ctype,
            scope,
            address: VariableAddress::default(),
            state: VariableState::Parameter,
        });
        Ok(var_id)
    }

    pub fn register_type(
        &mut self,
        name: &str,
        ctype: UserType,
        scope: Option<u128>,
    ) -> Result<u64, SemanticError> {
        let type_id = ScopeManager::hash_id(name, scope);
        self.types.push(Type {
            id: type_id,
            def: ctype,
            scope,
        });
        Ok(type_id)
    }

    pub fn find_var_by_name(
        &self,
        id: &str,
        scope: Option<u128>,
    ) -> Result<&Variable, SemanticError> {
        let var_id = ScopeManager::hash_id(id, scope);
        let var = self.vars.iter().find(|var| var.id == var_id);
        match var {
            Some(var) => Ok(var),
            None => return Err(SemanticError::UnknownVar(id.to_string())),
        }
    }

    pub fn find_var_by_id(&self, id: u64, scope: Option<u128>) -> Result<&Variable, SemanticError> {
        let var = self.vars.iter().find(|var| var.id == id);
        match var {
            Some(var) => Ok(var),
            None => return Err(SemanticError::UnknownVar(id.to_string())),
        }
    }
    pub fn alloc_global_var_by_id(
        &mut self,
        id: u64,
    ) -> Result<VariableAddress, CodeGenerationError> {
        let Some(var) = self.vars.iter_mut().find(|var| var.id == id) else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        var.address = self.global_mapping.alloc(var.ctype.size_of());
        Ok(var.address.clone())
    }

    pub fn find_type_by_name(
        &self,
        name: &str,
        scope: Option<u128>,
    ) -> Result<&UserType, SemanticError> {
        let type_id = ScopeManager::hash_id(name, scope);
        match scope {
            Some(scope_id) => {}
            None => {
                let found = self
                    .types
                    .iter()
                    .filter(|ut| ut.id == type_id && ut.scope.is_none())
                    .rev()
                    .collect::<Vec<_>>();
                let Some(user_type) = found.first() else {
                    return Err(SemanticError::CantInferType(name.to_string()));
                };
                return Ok(&user_type.def);
            }
        }
        todo!()
    }

    pub fn find_type_by_id(
        &self,
        id: u64,
        scope: Option<u128>,
    ) -> Result<&UserType, SemanticError> {
        let type_id = id;
        match scope {
            Some(scope_id) => {}
            None => {
                let found = self
                    .types
                    .iter()
                    .filter(|ut| ut.id == type_id && ut.scope.is_none())
                    .rev()
                    .collect::<Vec<_>>();
                let Some(user_type) = found.first() else {
                    return Err(SemanticError::CantInferType(id.to_string()));
                };
                return Ok(&user_type.def);
            }
        }
        todo!()
    }

    pub fn is_scope_in(&self, scope_id: u128, state: ScopeState) -> Option<u128> {
        let Some(branch) = self.scope_branches.get(&scope_id) else {
            return None;
        };

        for id in branch.iter().rev() {
            if let Some(scope_state) = self.scope_states.get(&id) {
                if *scope_state == state {
                    return Some(*id);
                }
            }
        }
        return None;
    }

    pub fn iter_on_local_variable<'a, T: 'a>(
        &'a self,
        scope_id: u128,
        map: fn(&Variable) -> T,
    ) -> impl Iterator<Item = T> + 'a {
        self.vars.iter().filter_map(move |var| {
            var.scope.and_then(|var_scope| {
                self.scope_branches.get(&var_scope).and_then(|branch| {
                    if branch.iter().rev().any(|&id| id == scope_id) {
                        Some(map(var))
                    } else {
                        None
                    }
                })
            })
        })
    }

    pub fn iter_mut_on_local_variable<'a>(
        &'a mut self,
        scope_id: u128,
    ) -> impl Iterator<Item = &mut Variable> + 'a {
        // First, collect the relevant scope branches
        let relevant_scopes: Vec<_> = self
            .scope_branches
            .iter()
            .filter(|(_, branch)| branch.iter().rev().any(|&id| id == scope_id))
            .map(|(scope, _)| *scope)
            .collect();

        // Then, filter and map the variables
        self.vars.iter_mut().filter_map(move |var| {
            if let Some(var_scope) = var.scope {
                if var.state == VariableState::Local && relevant_scopes.contains(&var_scope) {
                    return Some(var);
                }
            }
            None
        })
    }

    pub fn iter_mut_on_parameters<'a>(
        &'a mut self,
        scope_id: u128,
    ) -> impl Iterator<Item = &mut Variable> + 'a {
        // First, collect the relevant scope branches
        let relevant_scopes: Vec<_> = self
            .scope_branches
            .iter()
            .filter(|(_, branch)| branch.iter().rev().any(|&id| id == scope_id))
            .map(|(scope, _)| *scope)
            .collect();

        // Then, filter and map the variables
        self.vars.iter_mut().filter_map(move |var| {
            if let Some(var_scope) = var.scope {
                if var.state == VariableState::Parameter && relevant_scopes.contains(&var_scope) {
                    return Some(var);
                }
            }
            None
        })
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{DefaultHasher, Hash, Hasher};

    use super::*;

    #[test]
    fn valid_hash() {
        let x: String = "var".into();
        let mut hasher = DefaultHasher::new();
        x.hash(&mut hasher);
        let hash = hasher.finish();
    }
}
