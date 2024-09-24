use alloc::{AllocCasm, AllocFn};
use io::{IOCasm, IOFn};
use iter::{IterCasm, IterFn};
use map::{MapCasm, MapFn};
use math::{MathCasm, MathFn};
use string::{StringCasm, StringFn};
use strings::{StringsCasm, StringsFn};
use thread::{ThreadCasm, ThreadFn};
use vector::{VectorCasm, VectorFn};

use crate::{
    ast::{expressions::Expression, utils::strings::ID},
    e_static,
    semantic::{
        scope::static_types::{PrimitiveType, StaticType},
        EType, Resolve, ResolvePlatform, SemanticError, TypeOf,
    },
};

use crate::semantic::scope::scope::ScopeManager;

use super::{
    allocator::{heap::Heap, stack::Stack},
    asm::{operation::OpPrimitive, Asm, Program},
    stdio::StdIO,
    vm::{
        CasmMetadata, CodeGenerationError, Executable, GameEngineStaticFn, GenerateCode,
        RuntimeError,
    },
};

pub mod alloc;
pub mod io;
pub mod iter;
pub mod lexem;
pub mod map;
pub mod math;
pub mod string;
pub mod strings;
pub mod thread;
pub mod vector;

#[derive(Debug, Clone, PartialEq)]
pub enum Core {
    Vec(VectorFn),
    Map(MapFn),
    String(StringFn),
    Alloc(AllocFn),
    Thread(ThreadFn),
    IO(IOFn),
    Math(MathFn),
    Strings(StringsFn),
    Assert(bool),
    Iter(IterFn),
    Error,
    Ok,
}

pub trait PathFinder {
    fn find(path: &[String], name: &str) -> Option<Self>
    where
        Self: Sized;
}

impl PathFinder for Core {
    fn find(path: &[String], name: &str) -> Option<Self>
    where
        Self: Sized,
    {
        let path = if path.len() > 1 && path[0] == lexem::CORE {
            &path[1..]
        } else {
            path
        };

        if path.len() == 0 {
            match name {
                lexem::ASSERT => return Some(Core::Assert(false)),
                lexem::ERROR => return Some(Core::Error),
                lexem::OK => return Some(Core::Ok),
                _ => {}
            }
        }
        if let Some(core) = VectorFn::find(path, name) {
            return Some(Core::Vec(core));
        }
        if let Some(core) = MapFn::find(path, name) {
            return Some(Core::Map(core));
        }
        if let Some(core) = StringFn::find(path, name) {
            return Some(Core::String(core));
        }
        if let Some(core) = AllocFn::find(path, name) {
            return Some(Core::Alloc(core));
        }
        if let Some(core) = ThreadFn::find(path, name) {
            return Some(Core::Thread(core));
        }
        if let Some(core) = IOFn::find(path, name) {
            return Some(Core::IO(core));
        }
        if let Some(core) = MathFn::find(path, name) {
            return Some(Core::Math(core));
        }
        if let Some(core) = StringsFn::find(path, name) {
            return Some(Core::Strings(core));
        }
        if let Some(core) = IterFn::find(path, name) {
            return Some(Core::Iter(core));
        }

        return None;
    }
}

impl ResolvePlatform for Core {
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        match self {
            Core::Vec(value) => value.resolve::<G>(scope_manager, scope_id, context, parameters),
            Core::Map(value) => value.resolve::<G>(scope_manager, scope_id, context, parameters),
            Core::String(value) => value.resolve::<G>(scope_manager, scope_id, context, parameters),
            Core::Alloc(value) => value.resolve::<G>(scope_manager, scope_id, context, parameters),
            Core::Thread(value) => value.resolve::<G>(scope_manager, scope_id, context, parameters),
            Core::IO(value) => value.resolve::<G>(scope_manager, scope_id, context, parameters),
            Core::Math(value) => value.resolve::<G>(scope_manager, scope_id, context, parameters),
            Core::Strings(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, parameters)
            }
            Core::Iter(value) => value.resolve::<G>(scope_manager, scope_id, context, parameters),
            Core::Assert(expect_err) => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let size = &mut parameters[0];

                let _ = size.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(e_static!(StaticType::Primitive(PrimitiveType::Bool))),
                    &mut None,
                )?;
                let size_type = size.type_of(&scope_manager, scope_id)?;
                match &size_type {
                    EType::Static(value) => match value {
                        StaticType::Primitive(PrimitiveType::Bool) => *expect_err = false,
                        StaticType::Error => *expect_err = true,
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(e_static!(StaticType::Error))
            }
            Core::Error => {
                if parameters.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                return Ok(e_static!(StaticType::Error));
            }
            Core::Ok => {
                if parameters.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                return Ok(e_static!(StaticType::Error));
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoreCasm {
    Vec(VectorCasm),
    Map(MapCasm),
    String(StringCasm),
    Alloc(AllocCasm),
    Thread(ThreadCasm),
    IO(IOCasm),
    Math(MathCasm),
    Strings(StringsCasm),
    AssertBool,
    AssertErr,
    Error,
    Ok,
    Iter(IterCasm),
    Engine(String),
}

impl GenerateCode for Core {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Core::Vec(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Core::Map(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Core::String(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Core::Alloc(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Core::Thread(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Core::IO(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Core::Math(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Core::Strings(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Core::Iter(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Core::Assert(expect_err) => {
                if *expect_err {
                    instructions.push(Asm::Core(CoreCasm::AssertErr));
                } else {
                    instructions.push(Asm::Core(CoreCasm::AssertBool));
                }
                Ok(())
            }
            Core::Error => {
                instructions.push(Asm::Core(CoreCasm::Error));
                Ok(())
            }
            Core::Ok => {
                instructions.push(Asm::Core(CoreCasm::Ok));
                Ok(())
            }
        }
    }
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for CoreCasm {
    fn name(&self, stdio: &mut StdIO, program: &mut Program, engine: &mut G) {
        match self {
            CoreCasm::Engine(fn_id) => G::name_of_dynamic_fn(fn_id.clone(), stdio, program, engine),
            CoreCasm::Vec(value) => value.name(stdio, program, engine),
            CoreCasm::Map(value) => value.name(stdio, program, engine),
            CoreCasm::String(value) => value.name(stdio, program, engine),
            CoreCasm::Alloc(value) => value.name(stdio, program, engine),
            CoreCasm::Thread(value) => value.name(stdio, program, engine),
            CoreCasm::IO(value) => value.name(stdio, program, engine),
            CoreCasm::Math(value) => value.name(stdio, program, engine),
            CoreCasm::Strings(value) => value.name(stdio, program, engine),
            CoreCasm::Iter(value) => value.name(stdio, program, engine),
            CoreCasm::AssertBool => todo!(),
            CoreCasm::AssertErr => todo!(),
            CoreCasm::Error => todo!(),
            CoreCasm::Ok => todo!(),
        }
    }
    fn weight(&self) -> super::vm::CasmWeight {
        match self {
            CoreCasm::Engine(fn_id) => G::weight_of_dynamic_fn(fn_id.clone()),
            CoreCasm::Vec(value) => <VectorCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::Map(value) => <MapCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::String(value) => <StringCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::Alloc(value) => <AllocCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::Thread(value) => <ThreadCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::IO(value) => <IOCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::Math(value) => <MathCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::Strings(value) => <StringsCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::Iter(value) => <IterCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::AssertBool => todo!(),
            CoreCasm::AssertErr => todo!(),
            CoreCasm::Error => todo!(),
            CoreCasm::Ok => todo!(),
        }
    }
}

pub const ERROR_VALUE: u8 = 1;
pub const ERROR_SLICE: [u8; 1] = [ERROR_VALUE];
pub const OK_VALUE: u8 = 0;
pub const OK_SLICE: [u8; 1] = [OK_VALUE];

impl<G: crate::GameEngineStaticFn> Executable<G> for CoreCasm {
    fn execute(
        &self,
        program: &mut Program,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self {
            CoreCasm::Engine(fn_id) => {
                G::execute_dynamic_fn(fn_id.clone(), program, stack, heap, stdio, engine, tid)
            }
            CoreCasm::Vec(value) => value.execute(program, stack, heap, stdio, engine, tid),
            CoreCasm::Map(value) => value.execute(program, stack, heap, stdio, engine, tid),
            CoreCasm::String(value) => value.execute(program, stack, heap, stdio, engine, tid),
            CoreCasm::Alloc(value) => value.execute(program, stack, heap, stdio, engine, tid),
            CoreCasm::Thread(value) => value.execute(program, stack, heap, stdio, engine, tid),
            CoreCasm::IO(value) => value.execute(program, stack, heap, stdio, engine, tid),
            CoreCasm::Math(value) => value.execute(program, stack, heap, stdio, engine, tid),
            CoreCasm::Strings(value) => value.execute(program, stack, heap, stdio, engine, tid),
            CoreCasm::Iter(value) => value.execute(program, stack, heap, stdio, engine, tid),
            CoreCasm::AssertBool => {
                let condition = OpPrimitive::pop_bool(stack)?;
                program.incr();
                if condition {
                    // push NO_ERROR
                    let _ = stack.push_with(&OK_SLICE)?;
                    Ok(())
                } else {
                    // push ERROR
                    let _ = stack.push_with(&ERROR_SLICE)?;
                    Err(RuntimeError::AssertError)
                }
            }
            CoreCasm::AssertErr => {
                let condition = !OpPrimitive::pop_bool(stack)?;
                program.incr();
                if condition {
                    // push NO_ERROR
                    let _ = stack.push_with(&OK_SLICE)?;
                    Ok(())
                } else {
                    // push ERROR
                    let _ = stack.push_with(&ERROR_SLICE)?;
                    Err(RuntimeError::AssertError)
                }
            }
            CoreCasm::Error => {
                let _ = stack.push_with(&ERROR_SLICE)?;
                program.incr();
                Ok(())
            }
            CoreCasm::Ok => {
                let _ = stack.push_with(&OK_SLICE)?;
                program.incr();
                Ok(())
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
        let mut instructions = crate::vm::asm::Program::default();
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
