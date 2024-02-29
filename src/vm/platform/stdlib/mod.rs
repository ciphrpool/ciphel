use std::cell::Ref;

use crate::{
    ast::expressions::Expression,
    semantic::{scope::ScopeApi, EType, MutRc, Resolve, SemanticError, TypeOf},
    vm::{
        allocator::Memory,
        casm::CasmProgram,
        vm::{CodeGenerationError, Executable, RuntimeError},
    },
};

use self::{
    io::{IOCasm, IOFn},
    math::{MathCasm, MathFn},
    strings::{StringsCasm, StringsFn},
};

use super::GenerateCodePlatform;

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
    pub fn from(id: &String) -> Option<Self> {
        if let Some(value) = IOFn::from(id) {
            return Some(StdFn::IO(value));
        }
        if let Some(value) = MathFn::from(id) {
            return Some(StdFn::Math(value));
        }
        if let Some(value) = StringsFn::from(id) {
            return Some(StdFn::Strings(value));
        }
        None
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for StdFn {
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
            StdFn::IO(value) => value.resolve(scope, context, extra),
            StdFn::Math(value) => value.resolve(scope, context, extra),
            StdFn::Strings(value) => value.resolve(scope, context, extra),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for StdFn {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            StdFn::IO(value) => value.type_of(scope),
            StdFn::Math(value) => value.type_of(scope),
            StdFn::Strings(value) => value.type_of(scope),
        }
    }
}
impl<Scope: ScopeApi> GenerateCodePlatform<Scope> for StdFn {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
        params_size: usize,
    ) -> Result<(), CodeGenerationError> {
        match self {
            StdFn::IO(value) => value.gencode(scope, instructions, params_size),
            StdFn::Math(value) => value.gencode(scope, instructions, params_size),
            StdFn::Strings(value) => value.gencode(scope, instructions, params_size),
        }
    }
}

impl Executable for StdCasm {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match self {
            StdCasm::IO(value) => value.execute(program, memory),
            StdCasm::Math(_) => todo!(),
            StdCasm::Strings(_) => todo!(),
        }
    }
}
