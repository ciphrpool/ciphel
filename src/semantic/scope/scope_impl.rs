use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, HashMap},
    rc::{Rc, Weak},
    slice::Iter,
};

use crate::{
    ast::utils::strings::ID,
    p_num,
    semantic::{AccessLevel, Either, MutRc, SemanticError, SizeOf},
    vm::{allocator::stack::Offset, vm::CodeGenerationError},
};

use super::{
    chan_impl::Chan,
    event_impl::Event,
    static_types::{NumberType, PrimitiveType, StaticType},
    user_type_impl::UserType,
    var_impl::{Var, VarState},
    ClosureState, ScopeApi, ScopeState,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeData {
    vars: Vec<(Rc<Var>, Cell<Offset>)>,
    env_vars: MutRc<BTreeSet<Rc<Var>>>,
    env_vars_address: HashMap<ID, Cell<AccessLevel>>,
    types: HashMap<ID, Rc<UserType>>,
    state: Cell<ScopeState>,
}

#[derive(Debug, Clone)]
pub enum Scope {
    Inner {
        parent: Option<Weak<RefCell<Scope>>>,
        general: MutRc<Scope>,
        data: ScopeData,
    },
    General {
        data: ScopeData,
        events: HashMap<ID, Event>,
        channels: HashMap<ID, Chan>,
        stack_top: Cell<usize>,
    },
}

impl ScopeData {
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            types: HashMap::new(),
            state: Cell::default(),
            env_vars: Rc::new(RefCell::new(BTreeSet::default())),
            env_vars_address: HashMap::default(),
        }
    }

    pub fn spawn(&self) -> Self {
        Self {
            vars: Vec::default(),
            types: HashMap::new(),
            state: self.state.clone(),
            env_vars: Rc::new(RefCell::new(BTreeSet::default())),
            env_vars_address: HashMap::default(),
        }
    }
}

impl Scope {
    pub fn new() -> MutRc<Self> {
        Rc::new(RefCell::new(Self::General {
            data: ScopeData::new(),
            events: HashMap::new(),
            channels: HashMap::new(),
            stack_top: Cell::new(0),
        }))
    }
}

impl ScopeApi for Scope {
    // type UserType = UserType;

    // type StaticType = StaticType;

    // type Var = Var;

    // type Chan = Chan;

    // type Event = Event;

    fn spawn(parent: &MutRc<Self>, vars: Vec<Var>) -> Result<MutRc<Self>, SemanticError> {
        let borrowed_parent = parent.as_ref().borrow();
        match &*borrowed_parent {
            Scope::Inner { general, data, .. } => {
                let mut child = Self::Inner {
                    parent: Some(Rc::downgrade(parent)),
                    general: general.clone(),
                    data: data.spawn(),
                };
                for variable in vars {
                    let _ = child.register_var(variable);
                }
                let child = Rc::new(RefCell::new(child));
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
                Ok(Rc::new(RefCell::new(child)))
            }
        }
    }

    fn register_type(&mut self, id: &ID, reg: UserType) -> Result<(), SemanticError> {
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

    fn register_chan(&mut self, _reg: &ID) -> Result<(), SemanticError> {
        todo!()
    }

    fn register_var(&mut self, reg: Var) -> Result<(), SemanticError> {
        match self {
            Scope::Inner { data, .. } => {
                data.vars.push((reg.into(), Cell::default()));
                Ok(())
            }
            Scope::General { data, .. } => {
                reg.state.set(VarState::Global);
                data.vars.push((reg.into(), Cell::default()));
                Ok(())
            }
        }
    }

    fn register_event(&mut self, _reg: Event) -> Result<(), SemanticError> {
        todo!()
    }

    fn find_var(&self, id: &ID) -> Result<Rc<Var>, SemanticError> {
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
                .find(|(var, _)| &var.as_ref().id == id)
                .map(|(v, _)| v)
                .cloned()
                .or_else(|| match parent {
                    Some(parent) => parent.upgrade().and_then(|p| {
                        let borrowed_scope = p.as_ref().borrow();
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
                            <MutRc<Scope> as Borrow<RefCell<Scope>>>::borrow(&general);
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
                .find(|(var, _)| &var.as_ref().id == id)
                .map(|(v, _)| v)
                .cloned()
                .ok_or(SemanticError::UnknownVar(id.clone())),
        }
    }

    fn access_var(&self, id: &ID) -> Result<(Rc<Var>, Offset, AccessLevel), CodeGenerationError> {
        let is_closure = self.state().is_closure;
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
                .find(|(var, _)| &var.as_ref().id == id && var.is_declared.get())
                .map(|(var, offset)| (var.clone(), offset.get(), AccessLevel::Direct))
                .or_else(|| match parent {
                    Some(parent) => parent.upgrade().and_then(|p| {
                        let borrowed_scope = p.as_ref().borrow();
                        match borrowed_scope.access_var(id) {
                            Ok((var, offset, level)) => {
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
                                        for env_var in borrowed_scope.env_vars() {
                                            if env_var.id == var.id {
                                                break;
                                            }
                                            idx += env_var.as_ref().type_sig.size_of();
                                        }
                                        idx
                                    };
                                    let env_offset = match self.access_var(&"$ENV".into()).ok() {
                                        Some((_, Offset::FP(o), _)) => o,
                                        _ => return None,
                                    };
                                    Some((var, Offset::FE(env_offset, offset), level))
                                } else {
                                    Some((var, offset, level))
                                }
                            }
                            Err(_) => None,
                        }
                    }),
                    None => {
                        let borrowed_scope =
                            <MutRc<Scope> as Borrow<RefCell<Scope>>>::borrow(&general);
                        let borrowed_scope = borrowed_scope.borrow();

                        match borrowed_scope.access_var(id) {
                            Ok(var) => Some(var),
                            Err(_) => None,
                        }
                    }
                })
                .ok_or(CodeGenerationError::UnresolvedError),
            Scope::General { data, .. } => data
                .vars
                .iter()
                .find(|(var, _)| &var.as_ref().id == id)
                .map(|(var, offset)| (var.clone(), offset.get(), AccessLevel::General))
                .ok_or(CodeGenerationError::UnresolvedError),
        }
    }

    fn access_var_in_parent(
        &self,
        id: &ID,
    ) -> Result<(Rc<Var>, Offset, AccessLevel), CodeGenerationError> {
        let is_closure = self.state().is_closure;
        match self {
            Scope::Inner {
                data,
                parent,
                general,
                ..
            } => match parent {
                Some(parent) => parent.upgrade().and_then(|p| {
                    let borrowed_scope = p.as_ref().borrow();
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
                    let borrowed_scope = <MutRc<Scope> as Borrow<RefCell<Scope>>>::borrow(&general);
                    let borrowed_scope = borrowed_scope.borrow();

                    match borrowed_scope.access_var(id) {
                        Ok(var) => Some(var),
                        Err(_) => None,
                    }
                }
            }
            .ok_or(CodeGenerationError::UnresolvedError),
            Scope::General { data, .. } => Err(CodeGenerationError::UnresolvedError),
        }
    }
    fn capture(&self, var: Rc<Var>) -> bool {
        match self {
            Scope::Inner { data, .. } => {
                if data.state.get().is_closure != ClosureState::DEFAULT {
                    data.env_vars.as_ref().borrow_mut().insert(var);
                    return true;
                }
                return false;
            }
            _ => false,
        }
    }

    fn find_chan(&self) -> Result<&Chan, SemanticError> {
        todo!()
    }

    fn find_type(&self, id: &ID) -> Result<Rc<UserType>, SemanticError> {
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
                        let borrowed_scope = <MutRc<Scope> as Borrow<RefCell<Scope>>>::borrow(&p);
                        let borrowed_scope = borrowed_scope.borrow();
                        borrowed_scope.find_type(id).ok()
                    }),
                    None => {
                        let borrowed_scope =
                            <MutRc<Scope> as Borrow<RefCell<Scope>>>::borrow(&general);
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
    fn find_event(&self) -> Result<&Event, SemanticError> {
        todo!()
    }

    fn state(&self) -> ScopeState {
        match self {
            Scope::Inner { data, .. } => data.state.get(),
            Scope::General { data, .. } => data.state.get(),
        }
    }

    fn to_closure(&mut self, state: ClosureState) {
        match self {
            Scope::Inner { data, .. } => {
                data.state.get_mut().is_closure = state;
                if state == ClosureState::CAPTURING {
                    let mut offset = 0;
                    for (var, o) in &data.vars {
                        if var.state.get() == VarState::Parameter {
                            offset += var.type_sig.size_of();
                        }
                    }
                    data.vars.push((
                        Rc::new(Var {
                            id: "$ENV".into(),
                            type_sig: p_num!(U64),
                            state: VarState::Parameter.into(),
                            is_declared: Cell::new(true),
                        }),
                        Cell::new(Offset::FP(offset)),
                    ));
                }
            }
            _ => {}
        }
    }

    fn to_generator(&mut self) {
        match self {
            Scope::Inner { data, .. } => data.state.get_mut().is_generator = true,
            _ => {}
        }
    }

    fn to_loop(&mut self) {
        match self {
            Scope::Inner { data, .. } => data.state.get_mut().is_loop = true,
            _ => {}
        }
    }

    fn env_vars(&self) -> Vec<Rc<Var>> {
        match self {
            Scope::Inner { data, .. } => (&data.env_vars)
                .as_ref()
                .borrow()
                .clone()
                .into_iter()
                .collect(),
            Scope::General { data, .. } => (&data.env_vars)
                .as_ref()
                .borrow()
                .clone()
                .into_iter()
                .collect(),
        }
    }

    // fn access_level_of(&self, id: &ID) -> Result<AccessLevel, crate::vm::vm::CodeGenerationError> {
    //     todo!()
    // }

    fn vars(&self) -> Iter<(Rc<Var>, Cell<Offset>)> {
        match self {
            Scope::Inner {
                parent,
                general,
                data,
            } => data.vars.iter(),
            Scope::General { data, .. } => data.vars.iter(),
        }
    }

    fn update_var_offset(&self, id: &ID, offset: Offset) -> Result<Rc<Var>, CodeGenerationError> {
        match self {
            Scope::Inner { data, .. } => {
                if let Some((var, var_offset)) = data.vars.iter().rev().find(|(v, _)| &v.id == id) {
                    var_offset.set(offset);
                    Ok(var.clone())
                } else {
                    Err(CodeGenerationError::UnresolvedError)
                }
            }
            Scope::General { data, .. } => {
                if let Some((var, var_offset)) = data.vars.iter().rev().find(|(v, _)| &v.id == id) {
                    var_offset.set(offset);
                    Ok(var.clone())
                } else {
                    Err(CodeGenerationError::UnresolvedError)
                }
            }
        }
    }
    fn stack_top(&self) -> Option<usize> {
        match self {
            Scope::Inner { .. } => None,
            Scope::General { stack_top, .. } => Some(stack_top.get()),
        }
    }

    fn update_stack_top(&self, top: usize) -> Result<(), CodeGenerationError> {
        match self {
            Scope::Inner { .. } => Err(CodeGenerationError::UnresolvedError),
            Scope::General { stack_top, .. } => {
                stack_top.set(top);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MockScope {}

impl ScopeApi for MockScope {
    // type UserType = UserType;

    // type StaticType = StaticType;

    // type Var = Var;

    // type Chan = Chan;

    // type Event = Event;

    fn spawn(_parent: &MutRc<Self>, _vars: Vec<Var>) -> Result<MutRc<Self>, SemanticError> {
        unimplemented!("Mock function call")
    }

    fn register_type(&mut self, _id: &ID, _reg: UserType) -> Result<(), SemanticError> {
        unimplemented!("Mock function call")
    }

    fn register_chan(&mut self, _reg: &ID) -> Result<(), SemanticError> {
        unimplemented!("Mock function call")
    }

    fn register_var(&mut self, _reg: Var) -> Result<(), SemanticError> {
        unimplemented!("Mock function call")
    }

    fn register_event(&mut self, _reg: Event) -> Result<(), SemanticError> {
        unimplemented!("Mock function call")
    }

    fn find_chan(&self) -> Result<&Chan, SemanticError> {
        unimplemented!("Mock function call")
    }

    fn find_type(&self, _id: &ID) -> Result<Rc<UserType>, SemanticError> {
        unimplemented!("Mock function call")
    }

    fn find_event(&self) -> Result<&Event, SemanticError> {
        unimplemented!("Mock function call")
    }

    fn state(&self) -> ScopeState {
        unimplemented!("Mock function call")
    }

    fn to_closure(&mut self, state: ClosureState) {
        unimplemented!("Mock function call")
    }

    fn to_generator(&mut self) {
        unimplemented!("Mock function call")
    }

    fn to_loop(&mut self) {
        unimplemented!("Mock function call")
    }

    fn find_var(&self, id: &ID) -> Result<Rc<Var>, SemanticError> {
        unimplemented!("Mock function call")
    }

    fn env_vars(&self) -> Vec<Rc<Var>> {
        unimplemented!("Mock function call")
    }
    fn capture(&self, var: Rc<Var>) -> bool {
        unimplemented!("Mock function call")
    }
    fn vars(&self) -> Iter<(Rc<Var>, Cell<Offset>)> {
        unimplemented!("Mock function call")
    }
    fn stack_top(&self) -> Option<usize> {
        unimplemented!("Mock function call")
    }
    fn update_stack_top(&self, top: usize) -> Result<(), CodeGenerationError> {
        unimplemented!("Mock function call")
    }
    fn update_var_offset(&self, id: &ID, offset: Offset) -> Result<Rc<Var>, CodeGenerationError> {
        unimplemented!("Mock function call")
    }

    fn access_var(&self, id: &ID) -> Result<(Rc<Var>, Offset, AccessLevel), CodeGenerationError> {
        unimplemented!("Mock function call")
    }

    fn access_var_in_parent(
        &self,
        id: &ID,
    ) -> Result<(Rc<Var>, Offset, AccessLevel), CodeGenerationError> {
        unimplemented!("Mock function call")
    }
}
