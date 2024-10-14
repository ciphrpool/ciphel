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

pub trait ExternPathFinderFunctions {
    fn find(path: &[String], name: &str) -> Option<Self>
    where
        Self: Sized;
}

pub trait ExternFunction<E: Engine>:
    super::AsmName<E> + super::AsmWeight + super::scheduler::Executable<E> + Sized + ExternResolve
{
}

pub trait ExternIO {
    fn stdout_print(&mut self, content: String);
    fn stdout_println(&mut self, content: String);
    fn stderr_print(&mut self, content: String);
    fn stdin_scan<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) -> Option<String>;
    fn stdin_request<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    );
    fn stdasm_print(&mut self, content: String);
}

pub trait ExternEnergyDispenser<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> {
    fn get_energy(&self, pid: PID) -> usize;
    fn consume_energy(
        &mut self,
        energy: usize,
        pid: PID,
    ) -> Result<(), super::runtime::RuntimeError>;
}

pub trait ExternThreadHandler {
    type PID: ExternProcessIdentifier;
    type TID: ExternThreadIdentifier<Self::PID>;
    fn spawn(&mut self) -> Result<Self::TID, RuntimeError>;
    fn close(&mut self, tid: &Self::TID) -> Result<(), RuntimeError>;
}

pub trait Engine:
    ExternIO
    + ExternPathFinder
    + ExternEnergyDispenser<Self::PID, Self::TID>
    + ExternThreadHandler
    + Sized
{
    type Function: ExternFunction<Self>;
    type FunctionContext: ExternExecutionContext;
}

pub trait ExternExecutionContext: Default {}

pub trait ExternThreadIdentifier<PID: ExternProcessIdentifier>:
    Hash + PartialEq + std::cmp::Eq + Clone + Sized + Debug + Default
{
    fn to_u64(&self) -> u64;
    fn from_u64(tid: u64) -> Option<Self>;
    fn pid(&self) -> PID;
}

pub trait ExternProcessIdentifier:
    Hash + PartialEq + std::cmp::Eq + Clone + Sized + Debug + Default
{
}
