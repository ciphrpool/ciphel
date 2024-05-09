use std::cell::{Cell, Ref};

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
    semantic::{EType, MutRc, Resolve, SemanticError, TypeOf},
    vm::{
        casm::CasmProgram,
        vm::{CodeGenerationError, Executable, GenerateCode, RuntimeError},
    },
};

use self::{
    io::{IOCasm, IOFn},
    math::{MathCasm, MathFn},
    strings::{StringsCasm, StringsFn},
};

use super::utils::lexem;

pub mod io;
pub mod math;
pub mod strings;

#[derive(Debug, Clone, PartialEq)]
pub enum StdFn {
    IO(IOFn),
    Math(MathFn),
    Strings(StringsFn),
    Assert(Cell<bool>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StdCasm {
    IO(IOCasm),
    Math(MathCasm),
    Strings(StringsCasm),
    AssertBool,
    AssertErr,
}

impl<G: crate::GameEngineStaticFn + Clone> CasmMetadata<G> for StdCasm {
    fn name(&self, stdio: &mut StdIO<G>, program: &CasmProgram, engine: &mut G) {
        match self {
            StdCasm::IO(value) => value.name(stdio, program, engine),
            StdCasm::Math(value) => value.name(stdio, program, engine),
            StdCasm::Strings(value) => value.name(stdio, program, engine),
            StdCasm::AssertBool => stdio.push_casm_lib(engine, "assert"),
            StdCasm::AssertErr => stdio.push_casm_lib(engine, "assert"),
        }
    }
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
        match suffixe {
            Some(suffixe) => {
                if suffixe != lexem::STD {
                    return None;
                }
            }
            None => {}
        }
        match id.as_str() {
            lexem::ASSERT => Some(StdFn::Assert(false.into())),
            _ => None,
        }
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
            StdFn::Assert(expect_err) => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let size = &extra[0];

                let _ = size.resolve(
                    scope,
                    &Some(e_static!(StaticType::Primitive(PrimitiveType::Bool))),
                    &None,
                )?;
                let size_type = size.type_of(&scope.borrow())?;
                match &size_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Primitive(PrimitiveType::Bool) => expect_err.set(false),
                        StaticType::Error => expect_err.set(true),
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(())
            }
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
            StdFn::Assert(_) => Ok(e_static!(StaticType::Error)),
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
            StdFn::Assert(expect_err) => {
                if expect_err.get() {
                    instructions.push(Casm::Platform(super::LibCasm::Std(StdCasm::AssertErr)));
                } else {
                    instructions.push(Casm::Platform(super::LibCasm::Std(StdCasm::AssertBool)));
                }
                Ok(())
            }
        }
    }
}

impl<G: crate::GameEngineStaticFn + Clone> Executable<G> for StdCasm {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO<G>,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match self {
            StdCasm::IO(value) => value.execute(program, stack, heap, stdio, engine),
            StdCasm::Math(value) => value.execute(program, stack, heap, stdio, engine),
            StdCasm::Strings(value) => value.execute(program, stack, heap, stdio, engine),
            StdCasm::AssertBool => {
                let condition = OpPrimitive::get_bool(stack)?;
                program.incr();
                if condition {
                    // push NO_ERROR
                    // TODO : NO_ERROR value
                    let _ = stack.push_with(&[0u8]).map_err(|e| e.into())?;
                    Ok(())
                } else {
                    // push ERROR
                    // TODO : ERROR value
                    let _ = stack.push_with(&[1u8]).map_err(|e| e.into())?;
                    Err(RuntimeError::AssertError)
                }
            }
            StdCasm::AssertErr => {
                let condition = !OpPrimitive::get_bool(stack)?;
                program.incr();
                if condition {
                    // push NO_ERROR
                    // TODO : NO_ERROR value
                    let _ = stack.push_with(&[0u8]).map_err(|e| e.into())?;
                    Ok(())
                } else {
                    // push ERROR
                    // TODO : ERROR value
                    let _ = stack.push_with(&[1u8]).map_err(|e| e.into())?;
                    Err(RuntimeError::AssertError)
                }
            }
        }
    }
}
