use std::{
    borrow::Borrow,
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    ast::utils::strings::ID,
    semantic::{AccessLevel, MutRc, SemanticError, SizeOf},
};

use super::{
    chan_impl::Chan, event_impl::Event, user_type_impl::UserType, var_impl::Var, ScopeApi,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeData {
    vars: Vec<Rc<Var>>,
    types: HashMap<ID, Rc<UserType>>,
}

#[derive(Debug, Clone)]
pub enum Scope {
    Inner {
        parent: Option<Weak<RefCell<Scope>>>,
        general: MutRc<Scope>,
        outer_vars: MutRc<Vec<(ID, (Rc<Var>, AccessLevel))>>,
        data: ScopeData,
        can_capture: Cell<bool>,
    },
    General {
        data: ScopeData,
        events: HashMap<ID, Event>,
        channels: HashMap<ID, Chan>,
    },
}

impl ScopeData {
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            types: HashMap::new(),
        }
    }
}

impl Scope {
    pub fn new() -> MutRc<Self> {
        Rc::new(RefCell::new(Self::General {
            data: ScopeData::new(),
            events: HashMap::new(),
            channels: HashMap::new(),
        }))
    }
}

impl ScopeApi for Scope {
    // type UserType = UserType;

    // type StaticType = StaticType;

    // type Var = Var;

    // type Chan = Chan;

    // type Event = Event;

    fn child_scope_with(
        parent: &MutRc<Self>,
        vars: Vec<Var>,
    ) -> Result<MutRc<Self>, SemanticError> {
        let borrowed_parent = parent.as_ref().borrow();
        match &*borrowed_parent {
            Scope::Inner { general, .. } => {
                let mut child = Self::Inner {
                    parent: Some(Rc::downgrade(parent)),
                    general: general.clone(),
                    data: ScopeData::new(),
                    outer_vars: Rc::new(RefCell::new(Vec::default())),
                    can_capture: Cell::new(false),
                };
                for variable in vars {
                    let _ = child.register_var(variable);
                }
                Ok(Rc::new(RefCell::new(child)))
            }
            Scope::General {
                data: _,
                events: _,
                channels: _,
            } => {
                let mut child = Self::Inner {
                    parent: None,
                    general: parent.clone(),
                    data: ScopeData::new(),
                    can_capture: Cell::new(false),
                    outer_vars: Rc::new(RefCell::new(Vec::default())),
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
            Scope::Inner {
                parent: _,
                general: _,
                data,
                ..
            } => {
                data.types.insert(id.clone(), reg.into());
                Ok(())
            }
            Scope::General {
                data,
                events: _,
                channels: _,
            } => {
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
                data.vars.push(reg.into());
                Ok(())
            }
            Scope::General { data, .. } => {
                data.vars.push(reg.into());
                Ok(())
            }
        }
    }

    fn register_event(&mut self, _reg: Event) -> Result<(), SemanticError> {
        todo!()
    }
    fn to_capturing(&self) {
        match self {
            Scope::Inner {
                parent,
                general,
                outer_vars,
                data,
                can_capture,
            } => can_capture.set(true),
            Scope::General {
                data,
                events,
                channels,
            } => todo!(),
        }
    }

    fn find_var(&self, id: &ID) -> Result<(Rc<Var>, AccessLevel), SemanticError> {
        match self {
            Scope::Inner {
                data,
                parent,
                general,
                outer_vars,
                can_capture,
            } => data
                .vars
                .iter()
                .find(|var| &var.as_ref().id == id)
                .cloned()
                .map(|v| (v, AccessLevel::Direct))
                .or_else(|| match parent {
                    Some(parent) => parent.upgrade().and_then(|p| {
                        let borrowed_scope = <MutRc<Scope> as Borrow<RefCell<Scope>>>::borrow(&p);
                        let borrowed_scope = borrowed_scope.borrow();
                        match borrowed_scope.find_var(id) {
                            Ok((var, level)) => {
                                let level = match level {
                                    AccessLevel::General => AccessLevel::General,
                                    AccessLevel::Direct => AccessLevel::Backward(1),
                                    AccessLevel::Backward(l) => AccessLevel::Backward(l + 1),
                                };
                                let mut borrowerd_mut = outer_vars.borrow_mut();
                                if borrowerd_mut.iter().find(|(id, _)| id == &var.id).is_none() {
                                    // if can_capture.get() {
                                    //     var.is_captured.set((borrowerd_mut.len(), true));
                                    // } else {
                                    //     borrowerd_mut.push((var.id.clone(), (var.clone(), level)));
                                    // }
                                    borrowerd_mut.push((var.id.clone(), (var.clone(), level)));
                                }
                                // if can_capture.get() {
                                //     return Some((var, AccessLevel::Direct));
                                // }
                                Some((var, level))
                            }
                            Err(_) => None,
                        }
                    }),
                    None => {
                        let borrowed_scope =
                            <MutRc<Scope> as Borrow<RefCell<Scope>>>::borrow(&general);
                        let borrowed_scope = borrowed_scope.borrow();

                        match borrowed_scope.find_var(id) {
                            Ok((var, _)) => {
                                let mut borrowerd_mut = outer_vars.borrow_mut();
                                if borrowerd_mut.iter().find(|(id, _)| id == &var.id).is_none() {
                                    // if can_capture.get() {
                                    //     var.is_captured.set((borrowerd_mut.len(), true));
                                    // }
                                    borrowerd_mut.push((
                                        var.id.clone(),
                                        (var.clone(), AccessLevel::General),
                                    ));
                                }
                                // if can_capture.get() {
                                //     return Some((var, AccessLevel::Direct));
                                // }
                                Some((var, AccessLevel::General))
                            }
                            Err(_) => None,
                        }
                    }
                })
                .ok_or(SemanticError::UnknownVar(id.clone())),
            Scope::General { data, .. } => data
                .vars
                .iter()
                .find(|var| &var.as_ref().id == id)
                .cloned()
                .ok_or(SemanticError::UnknownVar(id.clone()))
                .map(|v| (v, AccessLevel::General)),
        }
    }

    fn find_outer_vars(&self) -> Vec<(ID, (Rc<Var>, AccessLevel))> {
        match self {
            Scope::Inner { outer_vars, .. } => outer_vars.as_ref().borrow().clone(),
            Scope::General { .. } => Vec::default(),
        }
    }

    fn capture_needed_vars(&mut self) {
        match self {
            Scope::Inner {
                outer_vars,
                data,
                can_capture,
                ..
            } => {
                if can_capture.get() {
                    for (idx, (_, (var, _))) in outer_vars.as_ref().borrow().iter().enumerate() {
                        let cloned_var = var.as_ref().clone();
                        cloned_var.is_captured.set((idx, true));
                        data.vars.push(Rc::new(cloned_var));
                    }
                }
            }
            Scope::General { .. } => {}
        }
    }

    fn inner_vars(&self) -> &Vec<Rc<Var>> {
        match self {
            Scope::Inner { data, .. } => &data.vars,
            Scope::General { data, .. } => &data.vars,
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
                .ok_or(SemanticError::UnknownVar(id.clone())),
            Scope::General { data, .. } => data
                .types
                .get(id)
                .cloned()
                .ok_or(SemanticError::UnknownType),
        }
    }

    fn parameters_size(&self) -> usize {
        match self {
            Scope::Inner {
                parent,
                general,
                outer_vars,
                data,
                can_capture,
            } => data
                .vars
                .iter()
                .filter_map(|v| v.is_parameter.get().1.then(|| v.type_sig.size_of()))
                .sum(),
            Scope::General {
                data,
                events,
                channels,
            } => data
                .vars
                .iter()
                .filter_map(|v| v.is_parameter.get().1.then(|| v.type_sig.size_of()))
                .sum(),
        }
    }
    fn find_event(&self) -> Result<&Event, SemanticError> {
        todo!()
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

    fn child_scope_with(
        _parent: &MutRc<Self>,
        _vars: Vec<Var>,
    ) -> Result<MutRc<Self>, SemanticError> {
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
    fn to_capturing(&self) {
        unimplemented!("Mock function call")
    }
    fn capture_needed_vars(&mut self) {
        unimplemented!("Mock function call")
    }
    fn find_var(&self, _id: &ID) -> Result<(Rc<Var>, AccessLevel), SemanticError> {
        unimplemented!("Mock function call")
    }
    fn find_outer_vars(&self) -> Vec<(ID, (Rc<Var>, AccessLevel))> {
        unimplemented!("Mock function call")
    }
    fn parameters_size(&self) -> usize {
        unimplemented!("Mock function call")
    }
    fn inner_vars(&self) -> &Vec<Rc<Var>> {
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
}
