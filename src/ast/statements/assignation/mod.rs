use crate::ast::{
    expressions::{
        data::{PtrAccess, Variable},
        Expression,
    },
};

use super::scope::Scope;
pub mod assignation_parse;
pub mod assignation_resolve;
pub mod assignation_typeof;

#[derive(Debug, Clone, PartialEq)]
pub struct Assignation {
    left: Assignee,
    right: AssignValue,
}
#[derive(Debug, Clone, PartialEq)]
pub enum AssignValue {
    Scope(Scope),
    Expr(Box<Expression>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Assignee {
    Variable(Variable),
    PtrAccess(PtrAccess),
}
