use std::cell::Ref;

use crate::semantic::scope::scope_impl::Scope;
use crate::{
    ast::expressions::Expression,
    semantic::{EType, MutRc, Resolve, SemanticError, TypeOf},
    vm::{
        casm::CasmProgram,
        scheduler::Thread,
        vm::{CodeGenerationError, Executable, GenerateCode, RuntimeError},
    },
};

use self::{
    io::{IOCasm, IOFn},
    math::{MathCasm, MathFn},
    strings::{StringsCasm, StringsFn},
};

pub mod io;
pub mod math;
pub mod strings;

#[derive(Debug, Clone, PartialEq)]
pub enum StdFn {
    IO(IOFn),
    Math(MathFn),
    Strings(StringsFn),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StdCasm {
    IO(IOCasm),
    Math(MathCasm),
    Strings(StringsCasm),
}

impl StdFn {
    pub fn from(suffixe: &Option<String>, id: &String) -> Option<Self> {
        if let Some(value) = IOFn::from(suffixe, id) {
            return Some(StdFn::IO(value));
        }
        if let Some(value) = MathFn::from(suffixe, id) {
            return Some(StdFn::Math(value));
        }
        if let Some(value) = StringsFn::from(suffixe, id) {
            return Some(StdFn::Strings(value));
        }
        None
    }
}

impl Resolve for StdFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression>;
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            StdFn::IO(value) => value.resolve(scope, context, extra),
            StdFn::Math(value) => value.resolve(scope, context, extra),
            StdFn::Strings(value) => value.resolve(scope, context, extra),
        }
    }
}

impl TypeOf for StdFn {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            StdFn::IO(value) => value.type_of(scope),
            StdFn::Math(value) => value.type_of(scope),
            StdFn::Strings(value) => value.type_of(scope),
        }
    }
}
impl GenerateCode for StdFn {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            StdFn::IO(value) => value.gencode(scope, instructions),
            StdFn::Math(value) => value.gencode(scope, instructions),
            StdFn::Strings(value) => value.gencode(scope, instructions),
        }
    }
}

impl Executable for StdCasm {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match self {
            StdCasm::IO(value) => value.execute(thread),
            StdCasm::Math(value) => value.execute(thread),
            StdCasm::Strings(_) => todo!(),
        }
    }
}
