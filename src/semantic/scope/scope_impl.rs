use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{ast::utils::strings::ID, semantic::SemanticError};

use super::{
    chan_impl::Chan, event_impl::Event, static_type_impl::StaticType, user_type_impl::UserType,
    var_impl::Var, ScopeApi,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeData {
    vars: HashMap<ID, Var>,
    types: HashMap<ID, UserType>,
}

#[derive(Debug, Clone)]
pub enum Scope {
    Inner {
        parent: Option<Weak<RefCell<Scope>>>,
        general: Rc<RefCell<Scope>>,
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
            Scope::Inner { general,  .. } => {
                let mut child = Self::Inner {
                    parent: Some(Rc::downgrade(parent)),
                    general: general.clone(),
                    data: ScopeData::new(),
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
                let child = Self::Inner {
                    parent: None,
                    general: parent.clone(),
                    data: ScopeData::new(),
                };

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
            } => {
                data.types.insert(id.clone(), reg);
                Ok(())
            }
            Scope::General {
                data,
                events: _,
                channels: _,
            } => {
                data.types.insert(id.clone(), reg);
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
                data.vars.insert(reg.id.clone(), reg);
                Ok(())
            }
            Scope::General { data, .. } => {
                data.vars.insert(reg.id.clone(), reg);
                Ok(())
            }
        }
    }

    fn register_event(&mut self, _reg: Self::Event) -> Result<(), crate::semantic::SemanticError> {
        todo!()
    }

    fn find_var(&self, id: &ID) -> Result<Self::Var, crate::semantic::SemanticError> {
        match self {
            Scope::Inner { data, parent, .. } => data
                .vars
                .get(id)
                .cloned()
                .or_else(|| {
                    parent.as_ref()?.upgrade().and_then(|p| {
                        let borrowed_scope =
                            <Rc<RefCell<Scope>> as Borrow<RefCell<Scope>>>::borrow(&p);
                        let borrowed_scope = borrowed_scope.borrow();
                        borrowed_scope.find_var(id).ok()
                    })
                })
                .ok_or(SemanticError::UnknownVar),
            Scope::General { data, .. } => {
                data.vars.get(id).cloned().ok_or(SemanticError::UnknownVar)
            }
        }
    }

    fn find_chan(&self) -> Result<&Self::Chan, crate::semantic::SemanticError> {
        todo!()
    }

    fn find_type(&self, id: &ID) -> Result<Self::UserType, crate::semantic::SemanticError> {
        match self {
            Scope::Inner { data, parent, .. } => data
                .types
                .get(id)
                .cloned()
                .or_else(|| {
                    parent.as_ref()?.upgrade().and_then(|p| {
                        let borrowed_scope =
                            <Rc<RefCell<Scope>> as Borrow<RefCell<Scope>>>::borrow(&p);
                        let borrowed_scope = borrowed_scope.borrow();
                        borrowed_scope.find_type(id).ok()
                    })
                })
                .ok_or(SemanticError::UnknownVar),
            Scope::General { data, .. } => {
                data.types.get(id).cloned().ok_or(SemanticError::UnknownVar)
            }
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

    fn register_type(&mut self, _id: &ID, _reg: Self::UserType) -> Result<(), SemanticError> {
        todo!()
    }

    fn register_chan(&mut self, _reg: &ID) -> Result<(), SemanticError> {
        todo!()
    }

    fn register_var(&mut self, _reg: Self::Var) -> Result<(), SemanticError> {
        todo!()
    }

    fn register_event(&mut self, _reg: Self::Event) -> Result<(), SemanticError> {
        todo!()
    }

    fn find_var(&self, _id: &ID) -> Result<Self::Var, SemanticError> {
        todo!()
    }

    fn find_chan(&self) -> Result<&Self::Chan, SemanticError> {
        todo!()
    }

    fn find_type(&self, _id: &ID) -> Result<Self::UserType, SemanticError> {
        todo!()
    }

    fn find_event(&self) -> Result<&Self::Event, SemanticError> {
        todo!()
    }

    fn child_scope_with(
        _parent: &Rc<RefCell<Self>>,
        _vars: Vec<Self::Var>,
    ) -> Result<Rc<RefCell<Self>>, SemanticError> {
        todo!()
    }
}
