use crate::{
    arw_new, arw_read, arw_write,
    ast::utils::strings::ID,
    p_num,
    semantic::{AccessLevel, ArcMutex,  SemanticError, SizeOf},
    vm::{allocator::stack::Offset, vm::CodeGenerationError},
    CompilationError,
};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, RwLock, Weak,
};
use std::{
    borrow::Borrow,
    collections::{BTreeSet, HashMap},
    slice::Iter,
};

use super::{
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
        general: Scope,
        data: ScopeData,
        refers_to_parent_vars: Arc<AtomicBool>,
    },
    General {
        data: ScopeData,
        transaction_data: ScopeData,
        stack_top: Arc<AtomicUsize>,
        in_transaction: bool,
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

    pub fn clear(&mut self) {
        self.vars.clear();
        self.types.clear();
        self.state = Default::default();
        self.env_vars = Default::default();
        self.env_vars_address = Default::default();
    }

    pub fn spawn(&self) -> Result<Self, SemanticError> {
        Ok(Self {
            vars: Vec::default(),
            types: HashMap::new(),
            state: Arc::new(RwLock::new(
                self.state
                    .read()
                    .map_err(|_| SemanticError::ConcurrencyError)?
                    .clone(),
            )),
            env_vars: Default::default(),
            env_vars_address: HashMap::default(),
        })
    }
}

impl Scope {
    pub fn new() -> ArcRwLock<Self> {
        Arc::new(RwLock::new(Self::General {
            data: ScopeData::new(),
            transaction_data: ScopeData::new(),
            stack_top: Default::default(),
            in_transaction: false,
        }))
    }
}

impl Scope {
    pub fn open_transaction(&mut self) -> Result<(), CompilationError> {
        match self {
            Scope::Inner {
                parent,
                general,
                data,
                ..
            } => {
                return Err(CompilationError::TransactionError(
                    "Cannot open a transaction in an inner scope",
                ))
            }
            Scope::General {
                data,
                stack_top,
                in_transaction,
                transaction_data,
            } => {
                *in_transaction = true;
                transaction_data.clear();
                Ok(())
            }
        }
    }

    pub fn commit_transaction(&mut self) -> Result<(), CompilationError> {
        match self {
            Scope::Inner {
                parent,
                general,
                data,
                ..
            } => {
                return Err(CompilationError::TransactionError(
                    "Cannot commit a transaction in an inner scope",
                ))
            }
            Scope::General {
                data,
                stack_top,
                in_transaction,
                transaction_data,
            } => {
                if *in_transaction {
                    data.vars.extend(transaction_data.vars.drain(..));
                    data.types.extend(transaction_data.types.drain());
                }
                Ok(())
            }
        }
    }

    pub fn reject_transaction(&mut self) -> Result<(), CompilationError> {
        match self {
            Scope::Inner {
                parent,
                general,
                data,
                ..
            } => {
                return Err(CompilationError::TransactionError(
                    "Cannot reject a transaction in an inner scope",
                ))
            }
            Scope::General {
                data,
                stack_top,
                in_transaction,
                transaction_data,
            } => {
                if *in_transaction {
                    transaction_data.vars.clear();
                    transaction_data.types.clear();
                }
                Ok(())
            }
        }
    }

    pub fn close_transaction(&mut self) -> Result<(), CompilationError> {
        match self {
            Scope::Inner {
                parent,
                general,
                data,
                ..
            } => {
                return Err(CompilationError::TransactionError(
                    "Cannot close a transaction in an inner scope",
                ))
            }
            Scope::General {
                data,
                stack_top,
                in_transaction,
                transaction_data,
            } => {
                *in_transaction = true;
                transaction_data.clear();
                Ok(())
            }
        }
    }

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
                    data: data.spawn()?,
                    refers_to_parent_vars: AtomicBool::new(false).into(),
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
                    refers_to_parent_vars: AtomicBool::new(false).into(),
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
            Scope::General {
                data,
                in_transaction,
                transaction_data,
                ..
            } => {
                if *in_transaction {
                    transaction_data.types.insert(id.clone(), reg.into());
                } else {
                    data.types.insert(id.clone(), reg.into());
                }
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
            Scope::General {
                data,
                in_transaction,
                transaction_data,
                ..
            } => {
                if *in_transaction {
                    reg.state = VarState::Global;
                    transaction_data
                        .vars
                        .push((Arc::new(RwLock::new(reg)), Default::default()));
                } else {
                    reg.state = VarState::Global;
                    data.vars
                        .push((Arc::new(RwLock::new(reg)), Default::default()));
                }
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
            } => self
                .find_var_in_data(data, id)
                .or_else(|| self.find_var_in_parent(parent, id))
                .or_else(|| self.find_var_in_general(general, id))
                .ok_or_else(|| SemanticError::UnknownVar(id.clone())),
            Scope::General {
                data,
                in_transaction,
                transaction_data,
                ..
            } => self.find_var_in_general_scope(data, *in_transaction, transaction_data, id),
        }
    }

    fn find_var_in_data(&self, data: &ScopeData, id: &ID) -> Option<ArcRwLock<Var>> {
        data.vars.iter().rev().find_map(|(var, _)| {
            arw_read!(var, SemanticError::ConcurrencyError)
                .ok()
                .filter(|v| v.id == *id)
                .map(|_| var.clone())
        })
    }

    fn find_var_in_parent(
        &self,
        parent: &Option<Weak<RwLock<Scope>>>,
        id: &ID,
    ) -> Option<ArcRwLock<Var>> {
        parent.as_ref().and_then(|p| {
            p.upgrade().and_then(|p| {
                let borrowed_scope = arw_read!(p, SemanticError::ConcurrencyError).ok()?;
                let var = borrowed_scope.find_var(id).ok()?;
                let _ = self.capture(var.clone());
                Some(var)
            })
        })
    }

    fn find_var_in_general(&self, general: &Scope, id: &ID) -> Option<ArcRwLock<Var>> {
        arw_read!(general, SemanticError::ConcurrencyError)
            .ok()
            .and_then(|borrowed_scope| borrowed_scope.find_var(id).ok())
    }

    fn find_var_in_general_scope(
        &self,
        data: &ScopeData,
        in_transaction: bool,
        transaction_data: &ScopeData,
        id: &ID,
    ) -> Result<ArcRwLock<Var>, SemanticError> {
        let find_in_data = |data: &ScopeData| {
            data.vars.iter().find_map(|(var, _)| {
                match arw_read!(var, SemanticError::ConcurrencyError) {
                    Ok(borrowed_var) if borrowed_var.id == *id => Some(var.clone()),
                    _ => None,
                }
            })
        };

        if in_transaction {
            find_in_data(transaction_data)
                .or_else(|| find_in_data(data))
                .ok_or_else(|| SemanticError::UnknownVar(id.clone()))
        } else {
            find_in_data(data).ok_or_else(|| SemanticError::UnknownVar(id.clone()))
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
            } => self.access_var_in_inner(id, is_closure, data, parent, general),
            Scope::General { data, .. } => self.access_var_in_general(id, data),
        }
    }

    fn access_var_in_inner(
        &self,
        id: &ID,
        is_closure: ClosureState,
        data: &ScopeData,
        parent: &Option<Weak<RwLock<Scope>>>,
        general: &Scope,
    ) -> Result<(ArcRwLock<Var>, Offset, AccessLevel), CodeGenerationError> {
        if let Some(result) = self.find_var_in_data_for_access(id, data)? {
            return Ok(result);
        }
        let from_inlined_scope = match self {
            Scope::Inner {
                data,
                parent,
                general,
                refers_to_parent_vars,
            } => data.vars.len() == 0,
            Scope::General { data, .. } => false,
        };
        match parent {
            Some(parent) => self.access_var_in_parent(id, is_closure, parent, from_inlined_scope),
            None => self.access_var_in_general_scope(id, general),
        }
    }

    fn find_var_in_data_for_access(
        &self,
        id: &ID,
        data: &ScopeData,
    ) -> Result<Option<(ArcRwLock<Var>, Offset, AccessLevel)>, CodeGenerationError> {
        for (var, offset) in data.vars.iter().rev() {
            let borrowed_var = arw_read!(var, CodeGenerationError::ConcurrencyError)?;
            if &borrowed_var.id == id && borrowed_var.is_declared {
                return Ok(Some((
                    var.clone(),
                    arw_read!(offset, CodeGenerationError::ConcurrencyError)?.clone(),
                    AccessLevel::Direct,
                )));
            }
        }
        Ok(None)
    }

    fn access_var_in_parent(
        &self,
        id: &ID,
        is_closure: ClosureState,
        parent: &Weak<RwLock<Scope>>,
        from_inlined_scope: bool,
    ) -> Result<(ArcRwLock<Var>, Offset, AccessLevel), CodeGenerationError> {
        let p = parent
            .upgrade()
            .ok_or(CodeGenerationError::UnresolvedError)?;
        let borrowed_scope = arw_read!(p, CodeGenerationError::ConcurrencyError)?;

        let (var, offset, level) = borrowed_scope.access_var(id)?;
        let borrowed_var = arw_read!(var, CodeGenerationError::ConcurrencyError)?;

        let level = self.adjust_access_level(level, from_inlined_scope);

        if is_closure == ClosureState::CAPTURING {
            self.handle_closure_capturing(borrowed_scope, borrowed_var, var.clone(), level)
        } else {
            Ok((var.clone(), offset, level))
        }
    }

    fn adjust_access_level(&self, level: AccessLevel, from_inlined_scope: bool) -> AccessLevel {
        match level {
            AccessLevel::General => AccessLevel::General,
            AccessLevel::Direct => {
                if from_inlined_scope {
                    AccessLevel::Direct
                } else {
                    AccessLevel::Backward(1)
                }
            }
            AccessLevel::Backward(l) => {
                AccessLevel::Backward(l + if from_inlined_scope { 0 } else { 1 })
            }
        }
    }

    fn handle_closure_capturing(
        &self,
        borrowed_scope: impl std::ops::Deref<Target = Scope>,
        borrowed_var: impl std::ops::Deref<Target = Var>,
        var: ArcRwLock<Var>,
        level: AccessLevel,
    ) -> Result<(ArcRwLock<Var>, Offset, AccessLevel), CodeGenerationError> {
        let level = match level {
            AccessLevel::Backward(1) => AccessLevel::Direct,
            AccessLevel::Backward(l) => AccessLevel::Backward(l - 1),
            _ => level,
        };

        let offset = self.calculate_env_var_offset(&borrowed_scope, &borrowed_var)?;
        let env_offset = self.get_env_offset()?;

        Ok((var, Offset::FE(env_offset, offset), level))
    }

    fn calculate_env_var_offset(
        &self,
        borrowed_scope: &Scope,
        borrowed_var: &Var,
    ) -> Result<usize, CodeGenerationError> {
        let mut offset = 0;
        for env_var in borrowed_scope
            .env_vars()
            .map_err(|_| CodeGenerationError::ConcurrencyError)?
        {
            if env_var.id == borrowed_var.id {
                break;
            }
            offset += env_var.as_ref().type_sig.size_of();
        }
        Ok(offset)
    }

    fn get_env_offset(&self) -> Result<usize, CodeGenerationError> {
        match self.access_var(&"$ENV".to_string().into()) {
            Ok((_, Offset::FP(o), _)) => Ok(o),
            _ => Err(CodeGenerationError::UnresolvedError),
        }
    }

    fn access_var_in_general_scope(
        &self,
        id: &ID,
        general: &Scope,
    ) -> Result<(ArcRwLock<Var>, Offset, AccessLevel), CodeGenerationError> {
        let borrowed_scope = arw_read!(general, CodeGenerationError::ConcurrencyError)?;
        borrowed_scope.access_var(id)
    }

    fn access_var_in_general(
        &self,
        id: &ID,
        data: &ScopeData,
    ) -> Result<(ArcRwLock<Var>, Offset, AccessLevel), CodeGenerationError> {
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
        Err(CodeGenerationError::UnresolvedError)
    }

    pub fn capture(&self, var: ArcRwLock<Var>) -> Result<bool, SemanticError> {
        match self {
            Scope::Inner {
                data,
                refers_to_parent_vars,
                ..
            } => {
                let state = arw_read!(data.state, SemanticError::ConcurrencyError)?;
                refers_to_parent_vars.store(true, Ordering::Release);

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
            Scope::General {
                data,
                in_transaction,
                transaction_data,
                ..
            } => {
                let found = data.types.get(id);
                if let Some(typ) = found {
                    return Ok(typ.clone());
                } else if *in_transaction {
                    let found = transaction_data.types.get(id);
                    if let Some(typ) = found {
                        return Ok(typ.clone());
                    } else {
                        return Err(SemanticError::UnknownType(id.clone()));
                    }
                } else {
                    return Err(SemanticError::UnknownType(id.clone()));
                }
            }
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

    pub fn refers_to_parent_vars(&self) -> bool {
        match self {
            Scope::Inner {
                data,
                refers_to_parent_vars,
                ..
            } => refers_to_parent_vars.load(Ordering::Acquire),
            Scope::General { data, .. } => false,
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
                ..
            } => data.vars.iter(),
            Scope::General { data, .. } => data.vars.iter(),
        }
    }

    pub fn for_closure_vars(
        &self,
    ) -> Result<Iter<(ArcRwLock<Var>, ArcRwLock<Offset>)>, SemanticError> {
        match self {
            Scope::Inner {
                parent: _,
                general: _,
                data,
                ..
            } => Ok(data.vars.iter()),
            Scope::General { .. } => Err(SemanticError::ExpectedClosure),
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
