use std::{fmt::Debug, hash::Hash};

use super::runtime::RuntimeError;

pub mod test;

pub trait ExternResolve {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        params: &mut Vec<crate::ast::expressions::Expression>,
    ) -> Result<crate::semantic::EType, crate::semantic::SemanticError>;
}

pub trait ExternPathFinder {
    fn find(path: &[String], name: &str) -> Option<Self::Function>
    where
        Self: Engine;
}

pub trait ExternFunction<E: Engine>:
    super::AsmName<E> + super::AsmWeight + super::scheduler::Executable<E> + Sized
{
}

pub trait ExternIO {
    fn stdout_print(&mut self, content: String);
    fn stdout_println(&mut self, content: String);
    fn stderr_print(&mut self, content: String);
    fn stdin_scan(&mut self) -> Option<String>;
    fn stdin_request(&mut self);
    fn stdasm_print(&mut self, content: String);
}

pub trait ExternEnergyDispenser {
    fn get_energy(&self) -> usize;
    fn consume_energy(&mut self, energy: usize) -> Result<(), super::runtime::RuntimeError>;
}

pub trait ExternThreadHandler {
    type TID: ExternThreadIdentifier;
    fn spawn(&mut self) -> Result<Self::TID, RuntimeError>;
    fn close(&mut self, tid: &Self::TID) -> Result<(), RuntimeError>;
}

pub trait Engine:
    ExternIO + ExternPathFinder + ExternEnergyDispenser + ExternThreadHandler + Sized
{
    type Function: ExternFunction<Self>;
    type FunctionContext: ExternExecutionContext;
}

pub trait ExternExecutionContext: Default {}

pub trait ExternThreadIdentifier:
    Hash + PartialEq + std::cmp::Eq + Clone + Sized + Debug + Default
{
    fn to_u64(&self) -> u64;
    fn from_u64(tid: u64) -> Self;
    // fn gen<E: ExternThreadHandler>(engine: &mut E) -> Self;
}
