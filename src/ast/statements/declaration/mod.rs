use crate::ast::{expressions::data::Closure, types::Type, utils::strings::ID};

use super::assignation::AssignValue;

pub mod declaration_gencode;
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
    RecClosure {
        left: TypedVar,
        right: Closure,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct TypedVar {
    pub name: String,
    pub id: Option<u64>,
    pub signature: Type,
}
#[derive(Debug, Clone, PartialEq)]
pub enum DeclaredVar {
    Id { name: String, id: Option<u64> },
    Typed(TypedVar),
    Pattern(PatternVar),
}
#[derive(Debug, Clone, PartialEq)]
pub enum PatternVar {
    StructFields {
        typename: String,
        vars: Vec<String>,
        ids: Option<Vec<u64>>,
    },
    Tuple {
        names: Vec<String>,
        ids: Option<Vec<u64>>,
    },
}
