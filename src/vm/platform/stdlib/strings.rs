use std::cell::Ref;

use crate::semantic::TypeOf;
use crate::vm::platform::utils::lexem;
use crate::{
    ast::expressions::Expression,
    semantic::{scope::ScopeApi, EType, MutRc, Resolve, SemanticError},
    vm::{
        casm::CasmProgram,
        vm::{CodeGenerationError, GenerateCode},
    },
};
#[derive(Debug, Clone, PartialEq)]
pub enum StringsFn {}
#[derive(Debug, Clone, PartialEq)]
pub enum StringsCasm {}

impl StringsFn {
    pub fn from(id: &String) -> Option<Self> {
        match id.as_str() {
            _ => None,
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for StringsFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression<Scope>>;
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            _ => todo!(),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for StringsFn {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            _ => todo!(),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for StringsFn {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}
