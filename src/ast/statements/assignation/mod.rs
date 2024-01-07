use crate::{
    ast::expressions::{
        data::{PtrAccess, Variable},
        Expression,
    },
    semantic::{scope::ScopeApi},
};

use super::scope::Scope;
pub mod assignation_parse;
pub mod assignation_resolve;
pub mod assignation_typeof;

#[derive(Debug, Clone, PartialEq)]
pub struct Assignation<InnerScope: ScopeApi> {
    left: Assignee,
    right: AssignValue<InnerScope>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum AssignValue<InnerScope: ScopeApi> {
    Scope(Scope<InnerScope>),
    Expr(Box<Expression<InnerScope>>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Assignee {
    Variable(Variable),
    PtrAccess(PtrAccess),
}
