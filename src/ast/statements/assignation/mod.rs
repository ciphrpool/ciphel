use crate::ast::expressions::Expression;

use super::block::Block;
pub mod assignation_gencode;
pub mod assignation_parse;
pub mod assignation_resolve;
pub mod assignation_typeof;

#[derive(Debug, Clone, PartialEq)]
pub struct Assignation {
    left: Expression,
    right: AssignValue,
}
#[derive(Debug, Clone, PartialEq)]
pub enum AssignValue {
    Scope(Block),
    Expr(Box<Expression>),
}
