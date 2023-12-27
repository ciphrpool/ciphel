pub mod scope;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Scope(scope::Scope),
}
