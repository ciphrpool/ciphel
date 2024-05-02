use std::cell::Ref;

use crate::semantic::scope::scope::Scope;
use crate::vm::allocator::heap::Heap;
use crate::vm::allocator::stack::Stack;
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

impl CasmMetadata for CoreCasm {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram) {
        match self {
            CoreCasm::Alloc(value) => value.name(stdio, program),
            CoreCasm::Thread(value) => value.name(stdio, program),
            CoreCasm::Chan(value) => {}
            CoreCasm::Cursor(value) => {}
        }
    }
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

impl Resolve for CoreFn {
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
            CoreFn::Alloc(value) => value.resolve(scope, context, extra),
            CoreFn::Thread(value) => value.resolve(scope, context, extra),
            CoreFn::Chan(value) => value.resolve(scope, context, extra),
            CoreFn::Cursor(value) => value.resolve(scope, context, extra),
        }
    }
}

impl TypeOf for CoreFn {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            CoreFn::Alloc(value) => value.type_of(scope),
            CoreFn::Thread(value) => value.type_of(scope),
            CoreFn::Chan(value) => value.type_of(scope),
            CoreFn::Cursor(value) => value.type_of(scope),
        }
    }
}

impl GenerateCode for CoreFn {
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
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
    ) -> Result<(), RuntimeError> {
        match self {
            CoreCasm::Alloc(value) => value.execute(program, stack, heap, stdio),
            CoreCasm::Thread(value) => value.execute(program, stack, heap, stdio),
            CoreCasm::Chan(_value) => todo!(),
            CoreCasm::Cursor(_value) => todo!(),
        }
    }
}
