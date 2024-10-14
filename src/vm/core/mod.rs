use alloc::{AllocAsm, AllocFn};
use format::{FormatAsm, FormatFn};
use io::{IOAsm, IOFn};
use iter::{IterAsm, IterFn};
use map::{MapAsm, MapFn};
use math::{MathAsm, MathFn};
use string::{StringAsm, StringFn};
use thread::{ThreadAsm, ThreadFn};
use vector::{VectorAsm, VectorFn};

use crate::{
    ast::expressions::Expression,
    e_static,
    semantic::{
        scope::static_types::{PrimitiveType, StaticType},
        EType, Resolve, ResolveCore, SemanticError, TypeOf,
    },
};

use super::{
    asm::{operation::OpPrimitive, Asm},
    runtime::RuntimeError,
    scheduler::Executable,
    stdio::StdIO,
    GenerateCode,
};

pub mod alloc;
pub mod format;
pub mod io;
pub mod iter;
pub mod lexem;
pub mod map;
pub mod math;
pub mod string;
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
    Format(FormatFn),
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
        if let Some(core) = FormatFn::find(path, name) {
            return Some(Core::Format(core));
        }
        if let Some(core) = IterFn::find(path, name) {
            return Some(Core::Iter(core));
        }

        return None;
    }
}

impl ResolveCore for Core {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        match self {
            Core::Vec(value) => value.resolve::<E>(scope_manager, scope_id, context, parameters),
            Core::Map(value) => value.resolve::<E>(scope_manager, scope_id, context, parameters),
            Core::String(value) => value.resolve::<E>(scope_manager, scope_id, context, parameters),
            Core::Alloc(value) => value.resolve::<E>(scope_manager, scope_id, context, parameters),
            Core::Thread(value) => value.resolve::<E>(scope_manager, scope_id, context, parameters),
            Core::IO(value) => value.resolve::<E>(scope_manager, scope_id, context, parameters),
            Core::Math(value) => value.resolve::<E>(scope_manager, scope_id, context, parameters),
            Core::Format(value) => value.resolve::<E>(scope_manager, scope_id, context, parameters),
            Core::Iter(value) => value.resolve::<E>(scope_manager, scope_id, context, parameters),
            Core::Assert(expect_err) => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let size = &mut parameters[0];

                let _ = size.resolve::<E>(
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
pub enum CoreAsm {
    Vec(VectorAsm),
    Map(MapAsm),
    String(StringAsm),
    Alloc(AllocAsm),
    Thread(ThreadAsm),
    IO(IOAsm),
    Math(MathAsm),
    Format(FormatAsm),
    AssertBool,
    AssertErr,
    Error,
    Ok,
    Iter(IterAsm),
}

impl GenerateCode for Core {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            Core::Vec(value) => value.gencode::<E>(scope_manager, scope_id, instructions, context),
            Core::Map(value) => value.gencode::<E>(scope_manager, scope_id, instructions, context),
            Core::String(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Core::Alloc(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Core::Thread(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Core::IO(value) => value.gencode::<E>(scope_manager, scope_id, instructions, context),
            Core::Math(value) => value.gencode::<E>(scope_manager, scope_id, instructions, context),
            Core::Format(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Core::Iter(value) => value.gencode::<E>(scope_manager, scope_id, instructions, context),
            Core::Assert(expect_err) => {
                if *expect_err {
                    instructions.push(Asm::Core(CoreAsm::AssertErr));
                } else {
                    instructions.push(Asm::Core(CoreAsm::AssertBool));
                }
                Ok(())
            }
            Core::Error => {
                instructions.push(Asm::Core(CoreAsm::Error));
                Ok(())
            }
            Core::Ok => {
                instructions.push(Asm::Core(CoreAsm::Ok));
                Ok(())
            }
        }
    }
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for CoreAsm {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            CoreAsm::Vec(value) => value.name(stdio, program, engine),
            CoreAsm::Map(value) => value.name(stdio, program, engine),
            CoreAsm::String(value) => value.name(stdio, program, engine),
            CoreAsm::Alloc(value) => value.name(stdio, program, engine),
            CoreAsm::Thread(value) => value.name(stdio, program, engine),
            CoreAsm::IO(value) => value.name(stdio, program, engine),
            CoreAsm::Math(value) => value.name(stdio, program, engine),
            CoreAsm::Format(value) => value.name(stdio, program, engine),
            CoreAsm::Iter(value) => value.name(stdio, program, engine),
            CoreAsm::AssertBool => stdio.push_asm_lib(engine, "assert_true"),
            CoreAsm::AssertErr => stdio.push_asm_lib(engine, "assert_ok"),
            CoreAsm::Error => stdio.push_asm_lib(engine, "error"),
            CoreAsm::Ok => stdio.push_asm_lib(engine, "ok"),
        }
    }
}

impl crate::vm::AsmWeight for CoreAsm {
    fn weight(&self) -> super::Weight {
        match self {
            CoreAsm::Vec(value) => value.weight(),
            CoreAsm::Map(value) => value.weight(),
            CoreAsm::String(value) => value.weight(),
            CoreAsm::Alloc(value) => value.weight(),
            CoreAsm::Thread(value) => value.weight(),
            CoreAsm::IO(value) => value.weight(),
            CoreAsm::Math(value) => value.weight(),
            CoreAsm::Format(value) => value.weight(),
            CoreAsm::Iter(value) => value.weight(),
            CoreAsm::AssertBool => super::Weight::ZERO,
            CoreAsm::AssertErr => super::Weight::ZERO,
            CoreAsm::Error => super::Weight::ZERO,
            CoreAsm::Ok => super::Weight::ZERO,
        }
    }
}

pub const ERROR_VALUE: u8 = 1;
pub const ERROR_SLICE: [u8; 1] = [ERROR_VALUE];
pub const OK_VALUE: u8 = 0;
pub const OK_SLICE: [u8; 1] = [OK_VALUE];

impl<E: crate::vm::external::Engine> Executable<E> for CoreAsm {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match self {
            CoreAsm::Vec(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            CoreAsm::Map(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            CoreAsm::String(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            CoreAsm::Alloc(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            CoreAsm::Thread(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            CoreAsm::IO(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            CoreAsm::Math(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            CoreAsm::Format(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            CoreAsm::Iter(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            CoreAsm::AssertBool => {
                let condition = OpPrimitive::pop_bool(stack)?;
                scheduler.next();
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
            CoreAsm::AssertErr => {
                let condition = !OpPrimitive::pop_bool(stack)?;
                scheduler.next();
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
            CoreAsm::Error => {
                let _ = stack.push_with(&ERROR_SLICE)?;
                scheduler.next();
                Ok(())
            }
            CoreAsm::Ok => {
                let _ = stack.push_with(&OK_SLICE)?;
                scheduler.next();
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn valid_extern_fn() {
    //     let mut statement = crate::ast::statements::Statement::parse(
    //         r##"
    //     {
    //         dynamic_fn();
    //     }
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut engine = crate::vm::vm::TestDynamicEngine {
    //         dynamic_fn_provider: crate::vm::vm::TestDynamicFnProvider {},
    //         out: String::new(),
    //     };
    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = statement
    //         .resolve::<crate::vm::vm::TestDynamicEngine>(
    //             &mut scope_manager,
    //             None,
    //             &None,
    //             &mut (),
    //         )
    //         .expect("Resolution should have succeeded");
    //     // Code generation.
    //     let mut instructions = crate::vm::program::Program::default();
    //     statement
    //         .gencode::<E>(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0, "No instructions generated");
    //     // Execute the instructions.
    //     let (mut runtime, mut heap, mut stdio) = crate::vm::vm::Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let output = engine.out;
    //     assert_eq!(&output, "Hello World from Dynamic function");
    // }
}
