use crate::{
    ast::{expressions::Expression, utils::strings::ID},
    semantic::{EType, Resolve, SemanticError, TypeOf},
};

use self::{
    core::{CoreCasm, CoreFn},
    stdlib::{StdCasm, StdFn},
};
use crate::semantic::scope::scope::ScopeManager;

use super::{
    allocator::{heap::Heap, stack::Stack},
    casm::CasmProgram,
    stdio::StdIO,
    vm::{
        CasmMetadata, CodeGenerationError, Executable, GameEngineStaticFn, GenerateCode,
        RuntimeError,
    },
};

pub mod core;
pub mod stdlib;
pub mod utils;

#[derive(Debug, Clone, PartialEq)]
pub enum Lib {
    Core(CoreFn),
    Std(StdFn),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LibCasm {
    Core(CoreCasm),
    Std(StdCasm),
    Engine(String),
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for LibCasm {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            LibCasm::Core(value) => value.name(stdio, program, engine),
            LibCasm::Std(value) => value.name(stdio, program, engine),
            LibCasm::Engine(fn_id) => G::name_of_dynamic_fn(fn_id.clone(), stdio, program, engine),
        }
    }
    fn weight(&self) -> super::vm::CasmWeight {
        match self {
            LibCasm::Core(value) => <CoreCasm as CasmMetadata<G>>::weight(value),
            LibCasm::Std(value) => <StdCasm as CasmMetadata<G>>::weight(value),
            LibCasm::Engine(fn_id) => G::weight_of_dynamic_fn(fn_id.clone()),
        }
    }
}

impl Lib {
    pub fn from(suffixe: &Option<ID>, id: &ID) -> Option<Self> {
        if let Some(value) = CoreFn::from(suffixe, id) {
            return Some(Lib::Core(value));
        }
        if let Some(value) = StdFn::from(suffixe, id) {
            return Some(Lib::Std(value));
        }
        None
    }
}

impl Resolve for Lib {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression>;
    fn resolve<GE: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            Lib::Core(value) => value.resolve::<GE>(scope_manager, scope_id, context, extra),
            Lib::Std(value) => value.resolve::<GE>(scope_manager, scope_id, context, extra),
        }
    }
}

impl TypeOf for Lib {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Lib::Core(value) => value.type_of(scope_manager, scope_id),
            Lib::Std(value) => value.type_of(scope_manager, scope_id),
        }
    }
}

impl GenerateCode for Lib {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Lib::Core(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Lib::Std(value) => value.gencode(scope_manager, scope_id, instructions, context),
        }
    }
}
impl<G: crate::GameEngineStaticFn> Executable<G> for LibCasm {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self {
            LibCasm::Core(value) => value.execute(program, stack, heap, stdio, engine, tid),
            LibCasm::Std(value) => value.execute(program, stack, heap, stdio, engine, tid),
            LibCasm::Engine(fn_id) => {
                G::execute_dynamic_fn(fn_id.clone(), program, stack, heap, stdio, engine, tid)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::TryParse, semantic::Resolve, vm::vm::GenerateCode};

    #[test]
    fn valid_dynamic_fn() {
        let mut statement = crate::ast::statements::Statement::parse(
            r##"
        {
            dynamic_fn();
        }
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let mut engine = crate::vm::vm::TestDynamicGameEngine {
            dynamic_fn_provider: crate::vm::vm::TestDynamicFnProvider {},
            out: String::new(),
        };
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = statement
            .resolve::<crate::vm::vm::TestDynamicGameEngine>(
                &mut scope_manager,
                None,
                &None,
                &mut (),
            )
            .expect("Resolution should have succeeded");
        // Code generation.
        let mut instructions = crate::vm::casm::CasmProgram::default();
        statement
            .gencode(
                &mut scope_manager,
                None,
                &mut instructions,
                &crate::vm::vm::CodeGenerationContext::default(),
            )
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let (mut runtime, mut heap, mut stdio) = crate::vm::vm::Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        program.merge(instructions);

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
            .expect("Execution should have succeeded");
        let output = engine.out;
        assert_eq!(&output, "Hello World from Dynamic function");
    }
}
