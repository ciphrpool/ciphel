use std::{collections::HashMap, rc::Rc};

use crate::ast::utils::strings::ID;

use super::{
    chan_impl::Chan, event_impl::Event, static_type_impl::StaticType, user_type_impl::UserType,
    var_impl::Var, ScopeApi,
};

#[derive(Debug, Clone)]
pub struct ScopeData {
    vars: HashMap<ID, Var>,
}

#[derive(Debug, Clone)]
pub enum Scope {
    Inner {
        parent: Option<Rc<Scope>>,
        general: Rc<Scope>,
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
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::General {
            data: ScopeData::new(),
            events: HashMap::new(),
            channels: HashMap::new(),
        }
    }
}

impl ScopeApi for Scope {
    type UserType = UserType;

    type StaticType = StaticType;

    type Var = Var;

    type Chan = Chan;

    type Event = Event;

    fn child_scope(&self) -> Result<Self, crate::semantic::SemanticError> {
        todo!()
    }

    fn attach(&mut self, vars: impl Iterator<Item = Self::Var>) {
        todo!()
    }

    fn register_type(&mut self, reg: Self::UserType) -> Result<(), crate::semantic::SemanticError> {
        todo!()
    }

    fn register_chan(&mut self, reg: &ID) -> Result<(), crate::semantic::SemanticError> {
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

    fn register_event(&mut self, reg: Self::Event) -> Result<(), crate::semantic::SemanticError> {
        todo!()
    }

    fn find_var(&self, id: &ID) -> Result<&Self::Var, crate::semantic::SemanticError> {
        todo!()
    }

    fn find_chan(&self) -> Result<&Self::Chan, crate::semantic::SemanticError> {
        todo!()
    }

    fn find_type(&self, id: &ID) -> Result<&Self::UserType, crate::semantic::SemanticError> {
        todo!()
    }

    fn find_event(&self) -> Result<&Self::Event, crate::semantic::SemanticError> {
        todo!()
    }
}
