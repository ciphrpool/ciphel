use crate::ast::utils::strings::ID;
use crate::e_static;
use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::static_types::{PrimitiveType, StaticType};
use crate::semantic::Either;
use crate::vm::allocator::heap::Heap;
use crate::vm::allocator::stack::Stack;
use crate::vm::casm::operation::OpPrimitive;
use crate::vm::casm::Casm;
use crate::vm::stdio::StdIO;
use crate::vm::vm::CasmMetadata;
use crate::{
    ast::expressions::Expression,
    semantic::{EType, Resolve, SemanticError, TypeOf},
    vm::{
        casm::CasmProgram,
        vm::{CodeGenerationError, Executable, GenerateCode, RuntimeError},
    },
};

use self::iter::{IterCasm, IterFn};
use self::{
    io::{IOCasm, IOFn},
    math::{MathCasm, MathFn},
    strings::{StringsCasm, StringsFn},
};

use super::utils::lexem;

pub mod io;
pub mod iter;
pub mod math;
pub mod strings;

#[derive(Debug, Clone, PartialEq)]
pub enum StdFn {
    IO(IOFn),
    Math(MathFn),
    Strings(StringsFn),
    Assert(bool),
    Iter(IterFn),
    Error,
    Ok,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StdCasm {
    IO(IOCasm),
    Math(MathCasm),
    Strings(StringsCasm),
    AssertBool,
    AssertErr,
    Error,
    Ok,
    Iter(IterCasm),
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for StdCasm {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            StdCasm::IO(value) => value.name(stdio, program, engine),
            StdCasm::Math(value) => value.name(stdio, program, engine),
            StdCasm::Strings(value) => value.name(stdio, program, engine),
            StdCasm::AssertBool => stdio.push_casm_lib(engine, "assert"),
            StdCasm::AssertErr => stdio.push_casm_lib(engine, "assert"),
            StdCasm::Iter(value) => value.name(stdio, program, engine),
            StdCasm::Error => stdio.push_casm_lib(engine, "err"),
            StdCasm::Ok => stdio.push_casm_lib(engine, "ok"),
        }
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            StdCasm::IO(value) => <IOCasm as CasmMetadata<G>>::weight(value),
            StdCasm::Math(value) => <MathCasm as CasmMetadata<G>>::weight(value),
            StdCasm::Strings(value) => <StringsCasm as CasmMetadata<G>>::weight(value),
            StdCasm::AssertBool => crate::vm::vm::CasmWeight::LOW,
            StdCasm::AssertErr => crate::vm::vm::CasmWeight::LOW,
            StdCasm::Error => crate::vm::vm::CasmWeight::LOW,
            StdCasm::Ok => crate::vm::vm::CasmWeight::LOW,
            StdCasm::Iter(value) => <IterCasm as CasmMetadata<G>>::weight(value),
        }
    }
}

impl StdFn {
    pub fn from(suffixe: &Option<ID>, id: &ID) -> Option<Self> {
        if let Some(value) = IOFn::from(suffixe, id) {
            return Some(StdFn::IO(value));
        }
        if let Some(value) = MathFn::from(suffixe, id) {
            return Some(StdFn::Math(value));
        }
        if let Some(value) = StringsFn::from(suffixe, id) {
            return Some(StdFn::Strings(value));
        }
        if let Some(value) = IterFn::from(suffixe, id) {
            return Some(StdFn::Iter(value));
        }
        match suffixe {
            Some(suffixe) => {
                if **suffixe != lexem::STD {
                    return None;
                }
            }
            None => {}
        }
        match id.as_str() {
            lexem::ASSERT => Some(StdFn::Assert(false.into())),
            lexem::ERROR => Some(StdFn::Error),
            lexem::OK => Some(StdFn::Ok),
            _ => None,
        }
    }
}

impl Resolve for StdFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression>;
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            StdFn::IO(value) => value.resolve::<G>(scope, context, extra),
            StdFn::Math(value) => value.resolve::<G>(scope, context, extra),
            StdFn::Strings(value) => value.resolve::<G>(scope, context, extra),
            StdFn::Iter(value) => value.resolve::<G>(scope, context, extra),
            StdFn::Assert(expect_err) => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let size = &mut extra[0];

                let _ = size.resolve::<G>(
                    scope,
                    &Some(e_static!(StaticType::Primitive(PrimitiveType::Bool))),
                    &mut None,
                )?;
                let size_type =
                    size.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                match &size_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Primitive(PrimitiveType::Bool) => *expect_err = false,
                        StaticType::Error => *expect_err = true,
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(())
            }
            StdFn::Error => {
                if extra.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                return Ok(());
            }
            StdFn::Ok => {
                if extra.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                return Ok(());
            }
        }
    }
}

impl TypeOf for StdFn {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            StdFn::IO(value) => value.type_of(scope),
            StdFn::Math(value) => value.type_of(scope),
            StdFn::Iter(value) => value.type_of(scope),
            StdFn::Strings(value) => value.type_of(scope),
            StdFn::Assert(_) => Ok(e_static!(StaticType::Error)),
            StdFn::Error => Ok(e_static!(StaticType::Error)),
            StdFn::Ok => Ok(e_static!(StaticType::Error)),
        }
    }
}
impl GenerateCode for StdFn {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            StdFn::IO(value) => value.gencode(scope, instructions),
            StdFn::Math(value) => value.gencode(scope, instructions),
            StdFn::Strings(value) => value.gencode(scope, instructions),
            StdFn::Iter(value) => value.gencode(scope, instructions),
            StdFn::Assert(expect_err) => {
                if *expect_err {
                    instructions.push(Casm::Platform(super::LibCasm::Std(StdCasm::AssertErr)));
                } else {
                    instructions.push(Casm::Platform(super::LibCasm::Std(StdCasm::AssertBool)));
                }
                Ok(())
            }
            StdFn::Error => {
                instructions.push(Casm::Platform(super::LibCasm::Std(StdCasm::Error)));
                Ok(())
            }
            StdFn::Ok => {
                instructions.push(Casm::Platform(super::LibCasm::Std(StdCasm::Ok)));
                Ok(())
            }
        }
    }
}

pub const ERROR_VALUE: [u8; 1] = [1];
pub const OK_VALUE: [u8; 1] = [0];

impl<G: crate::GameEngineStaticFn> Executable<G> for StdCasm {
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
            StdCasm::IO(value) => value.execute(program, stack, heap, stdio, engine, tid),
            StdCasm::Math(value) => value.execute(program, stack, heap, stdio, engine, tid),
            StdCasm::Strings(value) => value.execute(program, stack, heap, stdio, engine, tid),
            StdCasm::Iter(value) => value.execute(program, stack, heap, stdio, engine, tid),
            StdCasm::AssertBool => {
                let condition = OpPrimitive::get_bool(stack)?;
                program.incr();
                if condition {
                    // push NO_ERROR
                    let _ = stack.push_with(&OK_VALUE)?;
                    Ok(())
                } else {
                    // push ERROR
                    let _ = stack.push_with(&ERROR_VALUE)?;
                    Err(RuntimeError::AssertError)
                }
            }
            StdCasm::AssertErr => {
                let condition = !OpPrimitive::get_bool(stack)?;
                program.incr();
                if condition {
                    // push NO_ERROR
                    let _ = stack.push_with(&OK_VALUE)?;
                    Ok(())
                } else {
                    // push ERROR
                    let _ = stack.push_with(&ERROR_VALUE)?;
                    Err(RuntimeError::AssertError)
                }
            }
            StdCasm::Error => {
                let _ = stack.push_with(&ERROR_VALUE)?;
                program.incr();
                Ok(())
            }
            StdCasm::Ok => {
                let _ = stack.push_with(&OK_VALUE)?;
                program.incr();
                Ok(())
            }
        }
    }
}
