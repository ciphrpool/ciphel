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
pub enum CursorFn {
    Left,
    Right,
    Lock,
    Unlock,
    Show,
    Hide,
    Write,
    Clear,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CursorCasm {
    Left,
    Right,
    Lock,
    Unlock,
    Show,
    Hide,
    Write,
    Clear,
}

impl CursorFn {
    pub fn from(suffixe: &Option<String>, id: &String) -> Option<Self> {
        match id.as_str() {
            lexem::LEFT => Some(CursorFn::Left),
            lexem::RIGHT => Some(CursorFn::Right),
            lexem::LOCK => Some(CursorFn::Lock),
            lexem::UNLOCK => Some(CursorFn::Unlock),
            lexem::SHOW => Some(CursorFn::Show),
            lexem::HIDE => Some(CursorFn::Hide),
            lexem::WRITE => Some(CursorFn::Write),
            lexem::CLEAR_CELL => Some(CursorFn::Clear),
            _ => None,
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for CursorFn {
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
            CursorFn::Left => todo!(),
            CursorFn::Right => todo!(),
            CursorFn::Lock => todo!(),
            CursorFn::Unlock => todo!(),
            CursorFn::Show => todo!(),
            CursorFn::Hide => todo!(),
            CursorFn::Write => todo!(),
            CursorFn::Clear => todo!(),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for CursorFn {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            CursorFn::Left => todo!(),
            CursorFn::Right => todo!(),
            CursorFn::Lock => todo!(),
            CursorFn::Unlock => todo!(),
            CursorFn::Show => todo!(),
            CursorFn::Hide => todo!(),
            CursorFn::Write => todo!(),
            CursorFn::Clear => todo!(),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for CursorFn {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}
