use crate::ast::utils::strings::ID;
use crate::e_static;
use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::scope::static_types::{PrimitiveType, StaticType};
use crate::semantic::EType;
use crate::vm::allocator::heap::Heap;
use crate::vm::allocator::stack::Stack;
use crate::vm::casm::operation::OpPrimitive;
use crate::vm::casm::Casm;
use crate::vm::stdio::StdIO;
use crate::vm::vm::CasmMetadata;
use crate::{
    ast::expressions::Expression,
    semantic::{Resolve, SemanticError, TypeOf},
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
                if *suffixe != lexem::STD {
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
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            StdFn::IO(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            StdFn::Math(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            StdFn::Strings(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            StdFn::Iter(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            StdFn::Assert(expect_err) => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let size = &mut extra[0];

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
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            StdFn::IO(value) => value.type_of(scope_manager, scope_id),
            StdFn::Math(value) => value.type_of(scope_manager, scope_id),
            StdFn::Iter(value) => value.type_of(scope_manager, scope_id),
            StdFn::Strings(value) => value.type_of(scope_manager, scope_id),
            StdFn::Assert(_) => Ok(e_static!(StaticType::Error)),
            StdFn::Error => Ok(e_static!(StaticType::Error)),
            StdFn::Ok => Ok(e_static!(StaticType::Error)),
        }
    }
}
impl GenerateCode for StdFn {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            StdFn::IO(value) => value.gencode(scope_manager, scope_id, instructions, context),
            StdFn::Math(value) => value.gencode(scope_manager, scope_id, instructions, context),
            StdFn::Strings(value) => value.gencode(scope_manager, scope_id, instructions, context),
            StdFn::Iter(value) => value.gencode(scope_manager, scope_id, instructions, context),
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

pub const ERROR_VALUE: u8 = 1;
pub const ERROR_SLICE: [u8; 1] = [ERROR_VALUE];
pub const OK_VALUE: u8 = 0;
pub const OK_SLICE: [u8; 1] = [OK_VALUE];

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
                    let _ = stack.push_with(&OK_SLICE)?;
                    Ok(())
                } else {
                    // push ERROR
                    let _ = stack.push_with(&ERROR_SLICE)?;
                    Err(RuntimeError::AssertError)
                }
            }
            StdCasm::AssertErr => {
                let condition = !OpPrimitive::get_bool(stack)?;
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
            StdCasm::Error => {
                let _ = stack.push_with(&ERROR_SLICE)?;
                program.incr();
                Ok(())
            }
            StdCasm::Ok => {
                let _ = stack.push_with(&OK_SLICE)?;
                program.incr();
                Ok(())
            }
        }
    }
}
