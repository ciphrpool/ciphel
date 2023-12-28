use crate::ast::{types::Type, utils::strings::ID};

use super::assignation::AssignValue;

pub mod declaration_parse;
pub mod declaration_resolve;
pub mod declaration_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Declared(TypedVar),
    Assigned {
        left: DeclaredVar,
        right: AssignValue,
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
    UnionInline {
        typename: ID,
        variant: ID,
        vars: Vec<ID>,
    },
    UnionFields {
        typename: ID,
        variant: ID,
        vars: Vec<ID>,
    },
    StructInline {
        typename: ID,
        vars: Vec<ID>,
    },
    StructFields {
        typename: ID,
        vars: Vec<ID>,
    },
    Tuple(Vec<ID>),
}
