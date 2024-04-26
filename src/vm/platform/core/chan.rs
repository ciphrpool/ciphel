use std::cell::Ref;

use crate::semantic::scope::scope::Scope;
use crate::semantic::TypeOf;
use crate::vm::platform::utils::lexem;
use crate::{
    ast::expressions::Expression,
    semantic::{EType, MutRc, Resolve, SemanticError},
    vm::{
        casm::CasmProgram,
        vm::{CodeGenerationError, GenerateCode},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum ChanFn {
    Receive,
    Send,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChanCasm {
    Receive,
    Send,
}

impl ChanFn {
    pub fn from(suffixe: &Option<String>, id: &String) -> Option<Self> {
        match suffixe {
            Some(suffixe) => {
                if suffixe != lexem::CORE {
                    return None;
                }
            }
            None => {}
        }
        match id.as_str() {
            lexem::RECEIVE => Some(ChanFn::Receive),
            lexem::SEND => Some(ChanFn::Send),
            _ => None,
        }
    }
}

impl Resolve for ChanFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression>;
    fn resolve(
        &self,
        _scope: &MutRc<Scope>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            ChanFn::Receive => todo!(),
            ChanFn::Send => todo!(),
        }
    }
}
impl TypeOf for ChanFn {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            ChanFn::Receive => todo!(),
            ChanFn::Send => todo!(),
        }
    }
}

impl GenerateCode for ChanFn {
    fn gencode(
        &self,
        _scope: &MutRc<Scope>,
        _instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}
