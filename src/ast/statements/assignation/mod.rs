use crate::ast::expressions::Expression;

use super::block::{Block, ExprBlock};
pub mod assignation_gencode;
pub mod assignation_parse;
pub mod assignation_resolve;
pub mod assignation_typeof;

#[derive(Debug, Clone, PartialEq)]
pub struct Assignation {
    pub left: Expression,
    pub right: AssignValue,
}
#[derive(Debug, Clone, PartialEq)]
pub enum AssignValue {
    Block(ExprBlock),
    Expr(Box<Expression>),
}
