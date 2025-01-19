use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
};

use ulid::Ulid;

use crate::{
    ast::modules::Module,
    semantic::{EType, SemanticError, SizeOf},
    vm::{allocator::MemoryAddress, CodeGenerationError},
};

use super::{static_types::POINTER_SIZE, user_types::UserType};

#[derive(Debug, Clone, Default, PartialEq, Copy)]
pub enum VariableAddress {
    Global(usize),
    Local(usize),
    #[default]
    Unallocated,
}

impl TryInto<MemoryAddress> for VariableAddress {
    type Error = ();
    fn try_into(self) -> Result<MemoryAddress, Self::Error> {
        match self {
            VariableAddress::Global(address) => Ok(MemoryAddress::Global { offset: address }),
            VariableAddress::Local(address) => Ok(MemoryAddress::Frame { offset: address }),
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
    Function,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub id: u64,
    pub ctype: EType,
    pub scope: Option<u128>,
}

#[derive(Debug, Clone)]
pub enum ClosedMarker {
    Open,
    Close {
        closed_scope: u128,
        env_address: MemoryAddress,
        offset: usize,
    },
}

impl ClosedMarker {
    pub fn get(
        &self,
        scope_id: Option<u128>,
        scope_manager: &ScopeManager,
    ) -> Option<(MemoryAddress, usize)> {
        let Some(scope_id) = scope_id else {
            return None;
        };
        let ClosedMarker::Close {
            closed_scope,
            env_address,
            offset,
        } = self
        else {
            return None;
        };
        let Some(branch) = scope_manager.scope_branches.get(&scope_id) else {
            return None;
        };
        if branch.iter().find(|sid| **sid == *closed_scope).is_some() {
            return Some((env_address.clone(), *offset));
        } else {
            return None;
        }
    }
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub id: u64,
    pub name: String,
    pub count: usize,
    pub ctype: EType,
    pub scope: Option<u128>,
    pub is_global: bool,
    pub marked_as_closed_var: ClosedMarker,
    pub address: VariableAddress,
    pub state: VariableState,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub id: u64,
    pub def: UserType,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub id: u64,
    pub name: String,
    pub count: usize,
    pub def: UserType,
    pub scope: Option<u128>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ScopeState {
    Function,
    Closure,
    Lambda,
    Inline,
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
        // EMPTY STRING : must set the first 8 bytes to 0
        Self { top: 8 }
    }
}

impl GlobalMapping {
    pub const EMPTY_STRING_ADDRESS: MemoryAddress = MemoryAddress::Global { offset: 0 };
    pub fn alloc(&mut self, size: usize) -> VariableAddress {
        let variable = VariableAddress::Global(self.top);
        self.top += size;
        variable
    }
}

#[derive(Debug, Clone, Default)]
pub struct FrameMapping {
    pub param_size: usize,
    pub local_size: usize,
    pub top: usize,
    pub vars: Vec<(u64, usize)>, // var id and offset
}

#[derive(Debug, Clone, Default)]
pub struct TransactionStore {
    pub created_scopes: HashSet<u128>,
    pub created_vars: HashSet<u64>,
    pub created_types: HashSet<u64>,
    pub previous_global_top: usize,
    pub is_open: bool,
}

impl TransactionStore {
    pub fn open(&mut self, previous_global_top: usize) {
        self.created_scopes.clear();
        self.created_vars.clear();
        self.created_types.clear();
        self.previous_global_top = previous_global_top;
        self.is_open = true;
    }
}

#[derive(Debug, Clone)]
pub struct ScopeManager {
    pub modules: Vec<Module>,

    vars: HashMap<u64, VariableInfo>,
    types: Vec<TypeInfo>,
    scope_branches: HashMap<u128, Vec<u128>>, // parent scope of a given scope (key)

    pub allocating_scope: HashMap<u128, FrameMapping>,

    pub scope_types: HashMap<u128, Vec<EType>>,
    pub scope_lookup: HashMap<u128, HashSet<u64>>,
    pub scope_states: HashMap<u128, ScopeState>,
    pub global_mapping: GlobalMapping,

    pub transaction_store: TransactionStore,
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self {
            scope_branches: HashMap::default(),

            types: Vec::default(),
            vars: HashMap::default(),
            allocating_scope: HashMap::default(),

            scope_types: HashMap::default(),
            scope_lookup: HashMap::default(),
            scope_states: HashMap::default(),
            global_mapping: GlobalMapping::default(),
            modules: Vec::default(),
            transaction_store: TransactionStore::default(),
        }
    }
}

impl ScopeManager {
    pub fn open_transaction(&mut self) {
        self.transaction_store.open(self.global_mapping.top);
    }
    pub fn reject_transaction(&mut self) {
        self.global_mapping.top = self.transaction_store.previous_global_top;

        // Remove created variables
        for var_id in self.transaction_store.created_vars.iter() {
            self.vars.remove(var_id);
        }

        // Remove created types
        let types_to_keep: Vec<TypeInfo> = self
            .types
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.transaction_store.created_types.contains(&(*i as u64)))
            .map(|(_, t)| t.clone())
            .collect();
        self.types = types_to_keep;

        // Remove created scopes and clean up related mappings
        for scope_id in self.transaction_store.created_scopes.iter() {
            // Remove from scope branches
            self.scope_branches.remove(scope_id);

            // Remove from allocating scope
            self.allocating_scope.remove(scope_id);

            // Remove from scope types
            self.scope_types.remove(scope_id);

            // Remove from scope lookup
            self.scope_lookup.remove(scope_id);

            // Remove from scope states
            self.scope_states.remove(scope_id);
        }
        self.transaction_store.is_open = false;
    }

    pub fn accept_transaction(&mut self) {
        self.transaction_store.is_open = false;
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

        if self.transaction_store.is_open {
            self.transaction_store.created_scopes.insert(scope_id);
        }

        self.scope_lookup.insert(scope_id, HashSet::new());

        Ok(scope_id)
    }

    pub fn spawn_allocating(
        &mut self,
        parent: Option<u128>,
        with_caller: bool,
    ) -> Result<u128, SemanticError> {
        let scope_id = self.spawn(parent)?;
        self.allocating_scope.insert(
            scope_id,
            FrameMapping {
                top: if with_caller { POINTER_SIZE } else { 0 },
                local_size: 0,
                param_size: 0,
                vars: Vec::default(),
            },
        );
        if self.transaction_store.is_open {
            self.transaction_store.created_scopes.insert(scope_id);
        }
        Ok(scope_id)
    }

    pub fn hash_id(name: &str, count: usize, scope: Option<u128>) -> u64 {
        let mut hasher = DefaultHasher::default();
        name.hash(&mut hasher);
        count.hash(&mut hasher);
        scope.hash(&mut hasher);
        let hash = hasher.finish();
        hash
    }

    pub fn register_data(
        &mut self,
        data_size: usize,
        scope_id: Option<u128>,
    ) -> Result<Option<MemoryAddress>, SemanticError> {
        let Some(scope_id) = scope_id else {
            return Ok(None);
        };

        let Some(branch) = self.scope_branches.get(&scope_id) else {
            return Ok(None);
        };

        for id in branch.iter().rev() {
            let Some(scope_state) = self.scope_states.get(&id) else {
                continue;
            };

            if (*scope_state == ScopeState::IIFE)
                || (*scope_state == ScopeState::Closure)
                || (*scope_state == ScopeState::Lambda)
                || (*scope_state == ScopeState::Function)
            {
                let address: MemoryAddress;
                if let Some(mapping) = self.allocating_scope.get_mut(id) {
                    address = MemoryAddress::Frame {
                        offset: mapping.top,
                    };
                    mapping.top += data_size;
                    mapping.local_size += data_size;
                } else {
                    return Err(SemanticError::NotResolvedYet);
                }
                return Ok(Some(address));
            }
        }
        return Ok(None);
    }

    pub fn register_var(
        &mut self,
        name: &str,
        ctype: EType,
        scope: Option<u128>,
    ) -> Result<u64, SemanticError> {
        let count = self
            .vars
            .values()
            .filter(|v| (v.name == name) && (v.ctype != ctype))
            .map(|v| v.count)
            .max()
            .unwrap_or(0)
            + 1;
        let var_id = ScopeManager::hash_id(name, count, scope);
        let is_global = self.signal_variable_registation(var_id, ctype.size_of(), scope, false);
        if !self.vars.contains_key(&var_id) {
            self.vars.insert(
                var_id,
                VariableInfo {
                    id: var_id,
                    name: name.to_string(),
                    count,
                    is_global,
                    marked_as_closed_var: ClosedMarker::Open,
                    ctype,
                    scope,
                    address: VariableAddress::default(),
                    state: scope.map_or(VariableState::Global, |_| VariableState::Local),
                },
            );
        }
        if self.transaction_store.is_open {
            self.transaction_store.created_vars.insert(var_id);
        }
        Ok(var_id)
    }

    pub fn register_parameter(
        &mut self,
        name: &str,
        ctype: EType,
        scope: Option<u128>,
    ) -> Result<u64, SemanticError> {
        let count = self
            .vars
            .values()
            .filter(|v| v.name == name)
            .map(|v| v.count)
            .max()
            .unwrap_or(0)
            + 1;
        let var_id = ScopeManager::hash_id(name, count, scope);

        let _ = self.signal_variable_registation(var_id, ctype.size_of(), scope, true);
        self.vars.insert(
            var_id,
            VariableInfo {
                id: var_id,
                name: name.to_string(),
                count,
                is_global: false,
                marked_as_closed_var: ClosedMarker::Open,
                ctype,
                scope,
                address: VariableAddress::default(),
                state: VariableState::Parameter,
            },
        );
        if self.transaction_store.is_open {
            self.transaction_store.created_vars.insert(var_id);
        }
        Ok(var_id)
    }

    pub fn register_caller(
        &mut self,
        name: &str,
        ctype: EType,
        scope: u128,
    ) -> Result<u64, SemanticError> {
        let count = self
            .vars
            .values()
            .filter(|v| v.name == name)
            .map(|v| v.count)
            .max()
            .unwrap_or(0)
            + 1;
        let var_id = ScopeManager::hash_id(name, count, Some(scope));

        self.vars.insert(
            var_id,
            VariableInfo {
                id: var_id,
                name: name.to_string(),
                count,
                is_global: false,
                marked_as_closed_var: ClosedMarker::Open,
                ctype,
                scope: Some(scope),
                address: VariableAddress::Local(0),
                state: VariableState::Function,
            },
        );

        if self.transaction_store.is_open {
            self.transaction_store.created_vars.insert(var_id);
        }
        Ok(var_id)
    }

    pub fn register_type(
        &mut self,
        name: &str,
        ctype: UserType,
        scope: Option<u128>,
    ) -> Result<u64, SemanticError> {
        let count = self
            .types
            .iter()
            .filter(|v| v.name == name)
            .map(|v| v.count)
            .max()
            .unwrap_or(0)
            + 1;
        let type_id = ScopeManager::hash_id(name, count, scope);
        self.types.push(TypeInfo {
            id: type_id,
            name: name.to_string(),
            count,
            def: ctype,
            scope,
        });

        if self.transaction_store.is_open {
            self.transaction_store.created_types.insert(type_id);
        }
        Ok(type_id)
    }

    pub fn find_var_by_name(
        &self,
        name: &str,
        path: Option<&[String]>,
        scope: Option<u128>,
    ) -> Result<Variable, SemanticError> {
        if let Some(path) = path {
            return self
                .modules
                .iter()
                .find_map(|module| module.find_var(path, name))
                .ok_or(SemanticError::UnknownVar(name.to_string()));
        }

        match scope {
            Some(scope) => {
                let Some(branch) = self.scope_branches.get(&scope) else {
                    return Err(SemanticError::UnknownVar(name.to_string()));
                };
                let mut buffer: Vec<_> = self
                    .vars
                    .values()
                    .filter(|v| v.name == name)
                    .filter(|v| {
                        v.scope.is_none()
                            || branch.iter().find(|id| **id == v.scope.unwrap()).is_some()
                    })
                    .collect();

                buffer.sort_by(|v1, v2| {
                    v1.count
                        .partial_cmp(&v2.count)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let Some(variable) = buffer.last() else {
                    return Err(SemanticError::UnknownVar(name.to_string()));
                };
                Ok(Variable {
                    ctype: variable.ctype.clone(),
                    id: variable.id,
                    scope: variable.scope,
                })
            }
            None => {
                let mut buffer: Vec<_> = self
                    .vars
                    .values()
                    .filter(|v| v.scope.is_none() && v.name == name)
                    .collect();

                buffer.sort_by(|v1, v2| {
                    v1.count
                        .partial_cmp(&v2.count)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let Some(variable) = buffer.last() else {
                    return Err(SemanticError::UnknownVar(name.to_string()));
                };
                Ok(Variable {
                    ctype: variable.ctype.clone(),
                    id: variable.id,
                    scope: variable.scope,
                })
            }
        }
    }

    pub fn signal_variable_access(&mut self, variable: &Variable, scope_id: u128) {
        if variable.scope.is_none() {
            return;
        }
        let var_scope = variable.scope.unwrap();
        let Some(branch) = self.scope_branches.get(&scope_id) else {
            return;
        };

        for id in branch.iter().rev() {
            let Some(scope_state) = self.scope_states.get(&id) else {
                continue;
            };
            if (*scope_state == ScopeState::IIFE
                || *scope_state == ScopeState::Closure
                || *scope_state == ScopeState::Lambda)
                && (var_scope != *id)
            {
                let Some(outside) = self.scope_branches.get(&scope_id) else {
                    continue;
                };

                if outside.iter().rev().find(|s| var_scope == **s).is_some() {
                    // The variable is outside a closed scope
                    self.scope_lookup
                        .get_mut(&id)
                        .map(|set| set.insert(variable.id));
                }
            }
        }
    }

    pub fn mark_as_closed_var(
        &mut self,
        scope_id: u128,
        id: u64,
        address: MemoryAddress,
        offset: usize,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let Some(VariableInfo {
            marked_as_closed_var,
            ..
        }) = self.vars.get_mut(&id)
        else {
            dbg!("Here");
            return Err(CodeGenerationError::Unlocatable);
        };
        *marked_as_closed_var = ClosedMarker::Close {
            closed_scope: scope_id,
            env_address: address,
            offset,
        };
        Ok(())
    }

    pub fn is_scope_global(&self, scope_id: Option<u128>) -> bool {
        let Some(scope_id) = scope_id else {
            return true;
        };

        let Some(branch) = self.scope_branches.get(&scope_id) else {
            return true;
        };

        let mut is_global = true;
        for id in branch.iter().rev() {
            let Some(scope_state) = self.scope_states.get(&id) else {
                continue;
            };
            if (*scope_state == ScopeState::IIFE)
                || (*scope_state == ScopeState::Closure)
                || (*scope_state == ScopeState::Lambda)
                || (*scope_state == ScopeState::Function)
            {
                is_global = false;
                break;
            }
        }
        return is_global;
    }

    pub fn signal_variable_registation(
        &mut self,
        var_id: u64,
        var_size: usize,
        scope_id: Option<u128>,
        is_parameter: bool,
    ) -> bool {
        let Some(scope_id) = scope_id else {
            return true;
        };

        let Some(branch) = self.scope_branches.get(&scope_id) else {
            return true;
        };

        let mut is_global = true;
        for id in branch.iter().rev() {
            let Some(scope_state) = self.scope_states.get(&id) else {
                continue;
            };

            if (*scope_state == ScopeState::IIFE)
                || (*scope_state == ScopeState::Closure)
                || (*scope_state == ScopeState::Lambda)
                || (*scope_state == ScopeState::Function)
            {
                is_global = false;
                if let Some(mapping) = self.allocating_scope.get_mut(id) {
                    mapping.vars.push((var_id, mapping.top));
                    mapping.top += var_size;
                    if is_parameter {
                        mapping.param_size += var_size;
                    } else {
                        mapping.local_size += var_size;
                    }
                }
                break;
            }
        }
        return is_global;
    }

    pub fn find_var_by_id(&self, id: u64) -> Result<&VariableInfo, CodeGenerationError> {
        self.vars.get(&id).ok_or(CodeGenerationError::Unlocatable)
    }
    pub fn alloc_global_var_by_id(
        &mut self,
        id: u64,
    ) -> Result<VariableAddress, CodeGenerationError> {
        let Some(var) = self.vars.get_mut(&id) else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        var.address = self.global_mapping.alloc(var.ctype.size_of());
        Ok(var.address.clone())
    }

    pub fn find_type_by_name(
        &self,
        path: Option<&[String]>,
        name: &str,
        scope: Option<u128>,
    ) -> Result<Type, SemanticError> {
        if let Some(path) = path {
            return self
                .modules
                .iter()
                .find_map(|module| module.find_type(path, name))
                .ok_or(SemanticError::UnknownType(name.to_string()));
        }

        match scope {
            Some(scope) => {
                let Some(branch) = self.scope_branches.get(&scope) else {
                    return Err(SemanticError::UnknownType(name.to_string()));
                };

                let Some(ctype) = self
                    .types
                    .iter()
                    .rev()
                    .filter(|v| v.name == name)
                    .filter(|v| {
                        v.scope.is_none()
                            || branch.iter().find(|id| **id == v.scope.unwrap()).is_some()
                    })
                    .next()
                else {
                    return Err(SemanticError::UnknownType(name.to_string()));
                };
                Ok(Type {
                    id: ctype.id,
                    def: ctype.def.clone(),
                })
            }
            None => {
                let Some(ctype) = self
                    .types
                    .iter()
                    .rev()
                    .filter(|v| v.name == name && v.scope.is_none())
                    .next()
                else {
                    return Err(SemanticError::UnknownType(name.to_string()));
                };
                Ok(Type {
                    id: ctype.id,
                    def: ctype.def.clone(),
                })
            }
        }
    }

    pub fn find_type_by_id(&self, id: u64, scope: Option<u128>) -> Result<UserType, SemanticError> {
        let ctype = self.types.iter().find(|var| var.id == id);
        match ctype {
            Some(ctype) => Ok(ctype.def.clone()),
            None => return Err(SemanticError::CantInferType("unknown".to_string())),
        }
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

    pub fn can_return(&self, scope_id: u128) -> Option<u128> {
        let Some(branch) = self.scope_branches.get(&scope_id) else {
            return None;
        };

        for id in branch.iter().rev() {
            if let Some(scope_state) = self.scope_states.get(&id) {
                if *scope_state == ScopeState::Closure
                    || *scope_state == ScopeState::Function
                    || *scope_state == ScopeState::Lambda
                {
                    return Some(*id);
                }
            }
        }
        return None;
    }

    pub fn is_var_global(&self, var_id: u64) -> bool {
        self.vars.get(&var_id).filter(|v| v.is_global).is_some()
    }

    pub fn iter_on_global_variable(&self) -> impl Iterator<Item = &VariableInfo> {
        self.vars.values().filter(move |var| var.scope.is_none())
    }

    pub fn iter_mut_on_local_variable<'a>(
        &'a mut self,
        scope_id: u128,
    ) -> impl Iterator<Item = (&mut VariableInfo, usize)> + 'a {
        let vars = self.allocating_scope.get(&scope_id);

        self.vars.values_mut().filter_map(move |var| {
            let opt =
                vars.and_then(|mapping| mapping.vars.iter().find(|(id, offset)| *id == var.id));

            match opt {
                Some((_, offset)) if var.state == VariableState::Local => Some((var, *offset)),
                _ => None,
            }
        })
    }

    pub fn iter_mut_on_parameters<'a>(
        &'a mut self,
        scope_id: u128,
    ) -> impl Iterator<Item = (&mut VariableInfo, usize)> + 'a {
        let vars = self.allocating_scope.get(&scope_id);

        self.vars.values_mut().filter_map(move |var| {
            let opt =
                vars.and_then(|mapping| mapping.vars.iter().find(|(id, offset)| *id == var.id));

            match opt {
                Some((_, offset)) if var.state == VariableState::Parameter => Some((var, *offset)),
                _ => None,
            }
        })
    }
}
