use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{ast::utils::strings::ID, semantic::SemanticError};

use super::{
    chan_impl::Chan, event_impl::Event, static_types::StaticType, user_type_impl::UserType,
    var_impl::Var, ScopeApi,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeData {
    vars: HashMap<ID, Rc<Var>>,
    types: HashMap<ID, Rc<UserType>>,
}

#[derive(Debug, Clone)]
pub enum Scope {
    Inner {
        parent: Option<Weak<RefCell<Scope>>>,
        general: Rc<RefCell<Scope>>,
        outer_vars: Rc<RefCell<HashMap<ID, Rc<Var>>>>,
        data: ScopeData,
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
            vars: HashMap::new(),
            types: HashMap::new(),
        }
    }
}

impl Scope {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::General {
            data: ScopeData::new(),
            events: HashMap::new(),
            channels: HashMap::new(),
        }))
    }
}

impl ScopeApi for Scope {
    type UserType = UserType;

    type StaticType = StaticType;

    type Var = Var;

    type Chan = Chan;

    type Event = Event;

    fn child_scope_with(
        parent: &Rc<RefCell<Self>>,
        vars: Vec<Self::Var>,
    ) -> Result<Rc<RefCell<Self>>, SemanticError> {
        let borrowed_parent = parent.as_ref().borrow();
        match &*borrowed_parent {
            Scope::Inner { general, .. } => {
                let mut child = Self::Inner {
                    parent: Some(Rc::downgrade(parent)),
                    general: general.clone(),
                    data: ScopeData::new(),
                    outer_vars: Rc::new(RefCell::new(HashMap::default())),
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
                    outer_vars: Rc::new(RefCell::new(HashMap::default())),
                };
                for variable in vars {
                    let _ = child.register_var(variable);
                }
                Ok(Rc::new(RefCell::new(child)))
            }
        }
    }

    fn register_type(
        &mut self,
        id: &ID,
        reg: Self::UserType,
    ) -> Result<(), crate::semantic::SemanticError> {
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

    fn register_chan(&mut self, _reg: &ID) -> Result<(), crate::semantic::SemanticError> {
        todo!()
    }

    fn register_var(&mut self, reg: Self::Var) -> Result<(), crate::semantic::SemanticError> {
        match self {
            Scope::Inner { data, .. } => {
                data.vars.insert(reg.id.clone(), reg.into());
                Ok(())
            }
            Scope::General { data, .. } => {
                data.vars.insert(reg.id.clone(), reg.into());
                Ok(())
            }
        }
    }

    fn register_event(&mut self, _reg: Self::Event) -> Result<(), crate::semantic::SemanticError> {
        todo!()
    }

    fn find_var(&self, id: &ID) -> Result<Rc<Self::Var>, crate::semantic::SemanticError> {
        match self {
            Scope::Inner {
                data,
                parent,
                general,
                outer_vars,
            } => data
                .vars
                .get(id)
                .cloned()
                .or_else(|| match parent {
                    Some(parent) => parent.upgrade().and_then(|p| {
                        let borrowed_scope =
                            <Rc<RefCell<Scope>> as Borrow<RefCell<Scope>>>::borrow(&p);
                        let borrowed_scope = borrowed_scope.borrow();
                        match borrowed_scope.find_var(id) {
                            Ok(var) => {
                                {
                                    let mut captured = var.captured.borrow_mut();
                                    *captured = true;
                                }
                                let mut borrowerd_mut = outer_vars.borrow_mut();
                                borrowerd_mut.insert(var.id.clone(), var.clone());
                                Some(var)
                            }
                            Err(_) => None,
                        }
                    }),
                    None => {
                        let borrowed_scope =
                            <Rc<RefCell<Scope>> as Borrow<RefCell<Scope>>>::borrow(&general);
                        let borrowed_scope = borrowed_scope.borrow();

                        match borrowed_scope.find_var(id) {
                            Ok(var) => {
                                let mut borrowerd_mut = outer_vars.borrow_mut();
                                borrowerd_mut.insert(var.id.clone(), var.clone());
                                Some(var)
                            }
                            Err(_) => None,
                        }
                    }
                })
                .ok_or(SemanticError::UnknownVar(id.clone())),
            Scope::General { data, .. } => data
                .vars
                .get(id)
                .cloned()
                .ok_or(SemanticError::UnknownVar(id.clone())),
        }
    }

    fn find_outer_vars(&self) -> HashMap<ID, Rc<Self::Var>> {
        match self {
            Scope::Inner { outer_vars, .. } => outer_vars.as_ref().borrow().clone(),
            Scope::General { .. } => HashMap::default(),
        }
    }

    fn find_chan(&self) -> Result<&Self::Chan, crate::semantic::SemanticError> {
        todo!()
    }

    fn find_type(&self, id: &ID) -> Result<Rc<Self::UserType>, crate::semantic::SemanticError> {
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
                        let borrowed_scope =
                            <Rc<RefCell<Scope>> as Borrow<RefCell<Scope>>>::borrow(&p);
                        let borrowed_scope = borrowed_scope.borrow();
                        borrowed_scope.find_type(id).ok()
                    }),
                    None => {
                        let borrowed_scope =
                            <Rc<RefCell<Scope>> as Borrow<RefCell<Scope>>>::borrow(&general);
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

    fn find_event(&self) -> Result<&Self::Event, crate::semantic::SemanticError> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MockScope {}

impl ScopeApi for MockScope {
    type UserType = UserType;

    type StaticType = StaticType;

    type Var = Var;

    type Chan = Chan;

    type Event = Event;

    fn child_scope_with(
        parent: &Rc<RefCell<Self>>,
        vars: Vec<Self::Var>,
    ) -> Result<Rc<RefCell<Self>>, SemanticError> {
        unimplemented!("Mock function call")
    }

    fn register_type(&mut self, id: &ID, reg: Self::UserType) -> Result<(), SemanticError> {
        unimplemented!("Mock function call")
    }

    fn register_chan(&mut self, reg: &ID) -> Result<(), SemanticError> {
        unimplemented!("Mock function call")
    }

    fn register_var(&mut self, reg: Self::Var) -> Result<(), SemanticError> {
        unimplemented!("Mock function call")
    }

    fn register_event(&mut self, reg: Self::Event) -> Result<(), SemanticError> {
        unimplemented!("Mock function call")
    }

    fn find_var(&self, id: &ID) -> Result<Rc<Self::Var>, SemanticError> {
        unimplemented!("Mock function call")
    }
    fn find_outer_vars(&self) -> HashMap<ID, Rc<Self::Var>> {
        unimplemented!("Mock function call")
    }
    fn find_chan(&self) -> Result<&Self::Chan, SemanticError> {
        unimplemented!("Mock function call")
    }

    fn find_type(&self, id: &ID) -> Result<Rc<Self::UserType>, SemanticError> {
        unimplemented!("Mock function call")
    }

    fn find_event(&self) -> Result<&Self::Event, SemanticError> {
        unimplemented!("Mock function call")
    }
}
