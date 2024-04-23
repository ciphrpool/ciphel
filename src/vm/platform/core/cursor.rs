use std::cell::Ref;

use crate::semantic::scope::scope_impl::Scope;
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
    pub fn from(_suffixe: &Option<String>, id: &String) -> Option<Self> {
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
impl Resolve for CursorFn {
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
impl TypeOf for CursorFn {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
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

impl GenerateCode for CursorFn {
    fn gencode(
        &self,
        _scope: &MutRc<Scope>,
        _instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}
