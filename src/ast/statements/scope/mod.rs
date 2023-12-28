use super::Statement;

pub mod scope_parse;
pub mod scope_resolve;
pub mod scope_typeof;

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub instructions: Vec<Statement>,
}
