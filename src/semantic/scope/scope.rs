use crate::{
    arw_new, arw_read, arw_write,
    ast::utils::strings::ID,
    p_num,
    semantic::{AccessLevel, ArcMutex, ArcRwLock, Either, SemanticError, SizeOf},
    vm::{allocator::stack::Offset, vm::CodeGenerationError},
};
use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Cell, RefCell},
    collections::{BTreeSet, HashMap},
    slice::Iter,
};
use std::{
    slice::IterMut,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, RwLock, Weak,
    },
};

use super::{
    static_types::{NumberType, PrimitiveType, StaticType},
    user_type_impl::UserType,
    var_impl::{Var, VarState},
    ClosureState, ScopeState,
};

#[derive(Debug, Clone)]
pub struct ScopeData {
    vars: Vec<(ArcRwLock<Var>, ArcRwLock<Offset>)>,
    env_vars: ArcRwLock<BTreeSet<Arc<Var>>>,
    env_vars_address: HashMap<ID, ArcMutex<AccessLevel>>,
    types: HashMap<ID, Arc<UserType>>,
    state: ArcRwLock<ScopeState>,
}

#[derive(Debug, Clone)]
pub enum Scope {
    Inner {
        parent: Option<Weak<RwLock<Scope>>>,
        general: ArcRwLock<Scope>,
        data: ScopeData,
    },
    General {
        data: ScopeData,
        stack_top: Arc<AtomicUsize>,
    },
}

impl ScopeData {
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            types: HashMap::new(),
            state: Default::default(),
            env_vars: Default::default(),
            env_vars_address: HashMap::default(),
        }
    }

    pub fn spawn(&self) -> Self {
        Self {
            vars: Vec::default(),
            types: HashMap::new(),
            state: self.state.clone(),
            env_vars: Default::default(),
            env_vars_address: HashMap::default(),
        }
    }
}

impl Scope {
    pub fn new() -> ArcRwLock<Self> {
        Arc::new(RwLock::new(Self::General {
            data: ScopeData::new(),
            stack_top: Default::default(),
        }))
    }
}

impl Scope {
    pub fn spawn(
        parent: &ArcRwLock<Self>,
        vars: Vec<Var>,
    ) -> Result<ArcRwLock<Self>, SemanticError> {
        let borrowed_parent = arw_read!(parent, SemanticError::ConcurrencyError)?;
        match &*borrowed_parent {
            Scope::Inner { general, data, .. } => {
                let mut child = Self::Inner {
                    parent: Some(Arc::downgrade(parent)),
                    general: general.clone(),
                    data: data.spawn(),
                };
                for variable in vars {
                    let _ = child.register_var(variable);
                }
                let child = arw_new!(child);
                Ok(child)
            }
            Scope::General { .. } => {
                let mut child = Self::Inner {
                    parent: None,
                    general: parent.clone(),
                    data: ScopeData::new(),
                };
                for variable in vars {
                    let _ = child.register_var(variable);
                }
                Ok(arw_new!(child))
            }
        }
    }

    pub fn register_type(&mut self, id: &ID, reg: UserType) -> Result<(), SemanticError> {
        match self {
            Scope::Inner { data, .. } => {
                data.types.insert(id.clone(), reg.into());
                Ok(())
            }
            Scope::General { data, .. } => {
                data.types.insert(id.clone(), reg.into());
                Ok(())
            }
        }
    }

    pub fn register_var(&mut self, mut reg: Var) -> Result<(), SemanticError> {
        match self {
            Scope::Inner { data, .. } => {
                data.vars
                    .push((Arc::new(RwLock::new(reg)), Default::default()));
                Ok(())
            }
            Scope::General { data, .. } => {
                reg.state = VarState::Global;
                data.vars
                    .push((Arc::new(RwLock::new(reg)), Default::default()));
                Ok(())
            }
        }
    }

    pub fn find_var(&self, id: &ID) -> Result<ArcRwLock<Var>, SemanticError> {
        match self {
            Scope::Inner {
                data,
                parent,
                general,
                ..
            } => data
                .vars
                .iter()
                .rev()
                .find(|(var, _)| {
                    let borrowed_var = &arw_read!(var, CodeGenerationError::ConcurrencyError);
                    if let Ok(v) = borrowed_var {
                        v.id == *id
                    } else {
                        false
                    }
                })
                .map(|(v, _)| v)
                .cloned()
                .or_else(|| match parent {
                    Some(parent) => parent.upgrade().and_then(|p| {
                        let borrowed_scope = arw_read!(p, SemanticError::ConcurrencyError).ok()?;
                        let var = {
                            let var = borrowed_scope.find_var(id);
                            var.ok()
                        };
                        if let Some(var) = &var {
                            self.capture(var.clone());
                        }
                        var
                    }),
                    None => {
                        let borrowed_scope =
                            arw_read!(general, SemanticError::ConcurrencyError).ok()?;
                        let borrowed_scope = borrowed_scope.borrow();

                        match borrowed_scope.find_var(id) {
                            Ok(var) => Some(var),
                            Err(_) => None,
                        }
                    }
                })
                .ok_or(SemanticError::UnknownVar(id.clone())),
            Scope::General { data, .. } => data
                .vars
                .iter()
                .find(|(var, _)| {
                    let borrowed_var = &arw_read!(var, CodeGenerationError::ConcurrencyError);
                    if let Ok(v) = borrowed_var {
                        v.id == *id
                    } else {
                        false
                    }
                })
                .map(|(v, _)| v)
                .cloned()
                .ok_or(SemanticError::UnknownVar(id.clone())),
        }
    }

    pub fn access_var(
        &self,
        id: &ID,
    ) -> Result<(ArcRwLock<Var>, Offset, AccessLevel), CodeGenerationError> {
        let is_closure = self
            .state()
            .map_err(|_| CodeGenerationError::ConcurrencyError)?
            .is_closure;
        match self {
            Scope::Inner {
                data,
                parent,
                general,
                ..
            } => {
                for (var, offset) in data.vars.iter().rev() {
                    let borrowed_var = arw_read!(var, CodeGenerationError::ConcurrencyError)?;
                    if &borrowed_var.id == id && borrowed_var.is_declared {
                        return Ok((
                            var.clone(),
                            arw_read!(offset, CodeGenerationError::ConcurrencyError)?.clone(),
                            AccessLevel::Direct,
                        ));
                    } else {
                        continue;
                    }
                }
                // The variable is in a parent scope
                match parent {
                    Some(parent) => {
                        let Some(p) = parent.upgrade() else {
                            return Err(CodeGenerationError::UnresolvedError);
                        };

                        let borrowed_scope = arw_read!(p, CodeGenerationError::ConcurrencyError)?;

                        match borrowed_scope.access_var(id) {
                            Ok((var, offset, level)) => {
                                let borrowed_var =
                                    arw_read!(var, CodeGenerationError::ConcurrencyError)?;
                                let level = match level {
                                    AccessLevel::General => AccessLevel::General,
                                    AccessLevel::Direct => AccessLevel::Backward(1),
                                    AccessLevel::Backward(l) => AccessLevel::Backward(l + 1),
                                };
                                if is_closure == ClosureState::CAPTURING {
                                    let level = match level {
                                        AccessLevel::Backward(1) => AccessLevel::Direct,
                                        AccessLevel::Backward(l) => AccessLevel::Backward(l - 1),
                                        _ => level,
                                    };
                                    let offset = {
                                        let mut idx = 0;
                                        for env_var in borrowed_scope
                                            .env_vars()
                                            .map_err(|_| CodeGenerationError::ConcurrencyError)?
                                        {
                                            if env_var.id == borrowed_var.id {
                                                break;
                                            }
                                            idx += env_var.as_ref().type_sig.size_of();
                                        }
                                        idx
                                    };
                                    let env_offset =
                                        match self.access_var(&"$ENV".to_string().into()).ok() {
                                            Some((_, Offset::FP(o), _)) => o,
                                            _ => return Err(CodeGenerationError::UnresolvedError),
                                        };
                                    drop(borrowed_var);
                                    return Ok((var, Offset::FE(env_offset, offset), level));
                                } else {
                                    drop(borrowed_var);
                                    return Ok((var, offset, level));
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    None => {
                        let borrowed_scope =
                            arw_read!(general, CodeGenerationError::ConcurrencyError)?;
                        let borrowed_scope = borrowed_scope.borrow();

                        match borrowed_scope.access_var(id) {
                            Ok(var) => return Ok(var),
                            Err(_) => {}
                        };
                    }
                }
                return Err(CodeGenerationError::UnresolvedError);
            }
            Scope::General { data, .. } => {
                for (var, offset) in data.vars.iter().rev() {
                    let borrowed_var = arw_read!(var, CodeGenerationError::ConcurrencyError)?;
                    if &borrowed_var.id == id {
                        return Ok((
                            var.clone(),
                            arw_read!(offset, CodeGenerationError::ConcurrencyError)?.clone(),
                            AccessLevel::General,
                        ));
                    }
                }
                return Err(CodeGenerationError::UnresolvedError);
            }
        }
    }

    pub fn access_var_in_parent(
        &self,
        id: &ID,
    ) -> Result<(ArcRwLock<Var>, Offset, AccessLevel), CodeGenerationError> {
        let _is_closure = self
            .state()
            .map_err(|_| CodeGenerationError::ConcurrencyError)?
            .is_closure;
        match self {
            Scope::Inner {
                parent, general, ..
            } => match parent {
                Some(parent) => parent.upgrade().and_then(|p| {
                    let borrowed_scope = arw_read!(p, SemanticError::ConcurrencyError).ok()?;
                    match borrowed_scope.access_var(id) {
                        Ok((var, offset, level)) => {
                            let level = match level {
                                AccessLevel::General => AccessLevel::General,
                                AccessLevel::Direct => AccessLevel::Backward(1),
                                AccessLevel::Backward(l) => AccessLevel::Backward(l + 1),
                            };
                            Some((var, offset, level))
                        }
                        Err(_) => None,
                    }
                }),
                None => {
                    if let Some(borrowed_scope) =
                        arw_read!(general, SemanticError::ConcurrencyError).ok()
                    {
                        match borrowed_scope.access_var(id) {
                            Ok(var) => Some(var),
                            Err(_) => None,
                        }
                    } else {
                        None
                    }
                }
            }
            .ok_or(CodeGenerationError::UnresolvedError),
            Scope::General { .. } => Err(CodeGenerationError::UnresolvedError),
        }
    }
    pub fn capture(&self, var: ArcRwLock<Var>) -> Result<bool, SemanticError> {
        match self {
            Scope::Inner { data, .. } => {
                let state = arw_read!(data.state, SemanticError::ConcurrencyError)?;
                if state.is_closure != ClosureState::DEFAULT {
                    let Some(mut env_vars) =
                        arw_write!(data.env_vars, SemanticError::ConcurrencyError).ok()
                    else {
                        return Ok(false);
                    };
                    let var = arw_read!(var, SemanticError::ConcurrencyError)?.clone();
                    env_vars.insert(Arc::new(var));
                    return Ok(true);
                }
                return Ok(false);
            }
            _ => Ok(false),
        }
    }

    pub fn find_type(&self, id: &ID) -> Result<Arc<UserType>, SemanticError> {
        match self {
            Scope::Inner {
                data,
                parent,
                general,
                ..
            } => data
                .types
                .get(id)
                .cloned()
                .or_else(|| match parent {
                    Some(parent) => parent.upgrade().and_then(|p| {
                        let borrowed_scope = arw_read!(p, SemanticError::ConcurrencyError).ok()?;
                        let borrowed_scope = borrowed_scope.borrow();
                        borrowed_scope.find_type(id).ok()
                    }),
                    None => {
                        let borrowed_scope =
                            arw_read!(general, SemanticError::ConcurrencyError).ok()?;
                        let borrowed_scope = borrowed_scope.borrow();

                        borrowed_scope.find_type(id).ok()
                    }
                })
                .ok_or(SemanticError::UnknownType(id.clone())),
            Scope::General { data, .. } => data
                .types
                .get(id)
                .cloned()
                .ok_or(SemanticError::UnknownType(id.clone())),
        }
    }

    pub fn state(&self) -> Result<ScopeState, SemanticError> {
        match self {
            Scope::Inner { data, .. } => {
                let state = arw_read!(data.state, SemanticError::ConcurrencyError)?;
                Ok(state.clone())
            }
            Scope::General { data, .. } => {
                let state = arw_read!(data.state, SemanticError::ConcurrencyError)?;
                Ok(state.clone())
            }
        }
    }

    pub fn to_closure(&mut self, state: ClosureState) -> Result<(), SemanticError> {
        match self {
            Scope::Inner { data, .. } => {
                let mut data_state = arw_write!(data.state, SemanticError::ConcurrencyError)?;
                data_state.is_closure = state;
                if state == ClosureState::CAPTURING {
                    let mut offset = 0;
                    for (var, _o) in &data.vars {
                        let borrowed_var = arw_read!(var, SemanticError::ConcurrencyError)?;
                        if borrowed_var.state == VarState::Parameter {
                            offset += borrowed_var.type_sig.size_of();
                        }
                    }
                    data.vars.push((
                        Arc::new(RwLock::new(Var {
                            id: "$ENV".to_string().into(),
                            type_sig: p_num!(U64),
                            state: VarState::Parameter.into(),
                            is_declared: true,
                        })),
                        Arc::new(RwLock::new(Offset::FP(offset))),
                    ));
                    Ok(())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    pub fn to_loop(&mut self) -> Result<(), SemanticError> {
        match self {
            Scope::Inner { data, .. } => {
                let mut data_state = arw_write!(data.state, SemanticError::ConcurrencyError)?;
                data_state.is_loop = true;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn env_vars(&self) -> Result<Vec<Arc<Var>>, SemanticError> {
        match self {
            Scope::Inner { data, .. } => {
                Ok(arw_read!(data.env_vars, SemanticError::ConcurrencyError)?
                    .clone()
                    .into_iter()
                    .collect())
            }
            Scope::General { data, .. } => {
                Ok(arw_read!(data.env_vars, SemanticError::ConcurrencyError)?
                    .clone()
                    .into_iter()
                    .collect())
            }
        }
    }

    pub fn vars(&self) -> Iter<(ArcRwLock<Var>, ArcRwLock<Offset>)> {
        match self {
            Scope::Inner {
                parent: _,
                general: _,
                data,
            } => data.vars.iter(),
            Scope::General { data, .. } => data.vars.iter(),
        }
    }

    pub fn update_var_offset(
        &self,
        id: &ID,
        offset: Offset,
    ) -> Result<ArcRwLock<Var>, CodeGenerationError> {
        match self {
            Scope::Inner { data, .. } => {
                if let Some((var, var_offset)) = data.vars.iter().rev().find(|(v, _)| {
                    let borrowed_var = &arw_read!(v, CodeGenerationError::ConcurrencyError);
                    if let Ok(v) = borrowed_var {
                        v.id == *id
                    } else {
                        false
                    }
                }) {
                    let mut var_offset =
                        arw_write!(var_offset, CodeGenerationError::ConcurrencyError)?;
                    *var_offset = offset;
                    Ok(var.clone())
                } else {
                    Err(CodeGenerationError::UnresolvedError)
                }
            }
            Scope::General { data, .. } => {
                if let Some((var, var_offset)) = data.vars.iter().rev().find(|(v, _)| {
                    let borrowed_var = &arw_read!(v, CodeGenerationError::ConcurrencyError);
                    if let Ok(v) = borrowed_var {
                        v.id == *id
                    } else {
                        false
                    }
                }) {
                    let mut var_offset =
                        arw_write!(var_offset, CodeGenerationError::ConcurrencyError)?;
                    *var_offset = offset;
                    Ok(var.clone())
                } else {
                    Err(CodeGenerationError::UnresolvedError)
                }
            }
        }
    }
    pub fn stack_top(&self) -> Option<usize> {
        match self {
            Scope::Inner { .. } => None,
            Scope::General { stack_top, .. } => Some(stack_top.load(Ordering::Acquire)),
        }
    }

    pub fn update_stack_top(&self, top: usize) -> Result<(), CodeGenerationError> {
        match self {
            Scope::Inner { .. } => Err(CodeGenerationError::UnresolvedError),
            Scope::General { stack_top, .. } => {
                stack_top.store(top, Ordering::Release);
                Ok(())
            }
        }
    }
}
