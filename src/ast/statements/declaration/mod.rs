use crate::{
    ast::{types::Type, utils::strings::ID},
    semantic::scope::ScopeApi,
};

use super::assignation::AssignValue;

pub mod declaration_gencode;
pub mod declaration_parse;
pub mod declaration_resolve;
pub mod declaration_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration<InnerScope: ScopeApi> {
    Declared(TypedVar),
    Assigned {
        left: DeclaredVar,
        right: AssignValue<InnerScope>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct TypedVar {
    pub id: ID,
    pub signature: Type,
}
#[derive(Debug, Clone, PartialEq)]
pub enum DeclaredVar {
    Id(ID),
    Typed(TypedVar),
    Pattern(PatternVar),
}
#[derive(Debug, Clone, PartialEq)]
pub enum PatternVar {
    // UnionFields {
    //     typename: ID,
    //     variant: ID,
    //     vars: Vec<ID>,
    // },
    StructFields { typename: ID, vars: Vec<ID> },
    Tuple(Vec<ID>),
}
