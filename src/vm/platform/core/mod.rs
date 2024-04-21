use std::cell::Ref;

use crate::{
    ast::expressions::Expression,
    semantic::{scope::ScopeApi, EType, MutRc, Resolve, SemanticError, TypeOf},
    vm::{
        casm::CasmProgram,
        scheduler::Thread,
        vm::{CodeGenerationError, Executable, GenerateCode, RuntimeError},
    },
};

use self::{
    alloc::{AllocCasm, AllocFn},
    chan::{ChanCasm, ChanFn},
    cursor::{CursorCasm, CursorFn},
    thread::{ThreadCasm, ThreadFn},
};

pub mod alloc;
pub mod chan;
pub mod cursor;
pub mod thread;

#[derive(Debug, Clone, PartialEq)]
pub enum CoreFn {
    Alloc(AllocFn),
    Thread(ThreadFn),
    Chan(ChanFn),
    Cursor(CursorFn),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoreCasm {
    Alloc(AllocCasm),
    Thread(ThreadCasm),
    Chan(ChanCasm),
    Cursor(CursorCasm),
}

impl CoreFn {
    pub fn from(suffixe: &Option<String>, id: &String) -> Option<Self> {
        if let Some(value) = AllocFn::from(suffixe, id) {
            return Some(CoreFn::Alloc(value));
        }
        if let Some(value) = ChanFn::from(suffixe, id) {
            return Some(CoreFn::Chan(value));
        }
        if let Some(value) = ThreadFn::from(suffixe, id) {
            return Some(CoreFn::Thread(value));
        }
        if let Some(value) = CursorFn::from(suffixe, id) {
            return Some(CoreFn::Cursor(value));
        }
        None
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for CoreFn {
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
            CoreFn::Alloc(value) => value.resolve(scope, context, extra),
            CoreFn::Thread(value) => value.resolve(scope, context, extra),
            CoreFn::Chan(value) => value.resolve(scope, context, extra),
            CoreFn::Cursor(value) => value.resolve(scope, context, extra),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for CoreFn {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            CoreFn::Alloc(value) => value.type_of(scope),
            CoreFn::Thread(value) => value.type_of(scope),
            CoreFn::Chan(value) => value.type_of(scope),
            CoreFn::Cursor(value) => value.type_of(scope),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for CoreFn {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            CoreFn::Alloc(value) => value.gencode(scope, instructions),
            CoreFn::Thread(value) => value.gencode(scope, instructions),
            CoreFn::Chan(value) => value.gencode(scope, instructions),
            CoreFn::Cursor(value) => value.gencode(scope, instructions),
        }
    }
}

impl Executable for CoreCasm {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match self {
            CoreCasm::Alloc(value) => value.execute(thread),
            CoreCasm::Thread(value) => todo!(),
            CoreCasm::Chan(value) => todo!(),
            CoreCasm::Cursor(value) => todo!(),
        }
    }
}
