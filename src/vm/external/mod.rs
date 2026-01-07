use std::{fmt::Debug, hash::Hash};

use super::{allocator::MemoryAddress, runtime::RuntimeError};

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

#[derive(Debug, Default)]
pub struct EventSetupResult {
    pub parameters_size: usize,
    pub function_offset: usize,
    pub callback: u64,
}
pub trait ExternEventManager<
    EC: ExternExecutionContext,
    PID: ExternProcessIdentifier,
    TID: ExternThreadIdentifier<PID>,
>: Clone + Copy + Debug + PartialEq
{
    type E: Engine<FunctionContext = EC, PID = PID, TID = TID>;

    fn event_setup(
        &self,
        callback_address: MemoryAddress,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut Self::E,
        context: &crate::vm::scheduler::ExecutionContext<EC, PID, TID>,
    ) -> Result<EventSetupResult, crate::vm::runtime::RuntimeError> {
        Ok(EventSetupResult::default())
    }
    fn event_conclusion(
        &self,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut Self::E,
        context: &crate::vm::scheduler::ExecutionContext<EC, PID, TID>,
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        Ok(())
    }
    fn event_cleanup(
        &self,
        callback_address: MemoryAddress,
        event_state: crate::vm::scheduler::EventState,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut Self::E,
        context: &crate::vm::scheduler::ExecutionContext<EC, PID, TID>,
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        Ok(())
    }
    fn event_trigger(&self, signal: u64, trigger: u64) -> bool {
        false
    }
}

pub trait ExternFunction<E: Engine>:
    super::AsmName<E>
    + super::AsmWeight
    + super::scheduler::Executable<E>
    + Sized
    + ExternResolve
    + Clone
    + Copy
    + Debug
    + PartialEq
    + ExternEventManager<E::FunctionContext, E::PID, E::TID, E = E>
{
}

pub trait ExternIO<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> {
    fn stdout_print(&mut self, pid: PID, content: String);
    fn stdout_println(&mut self, pid: PID, content: String);
    fn stderr_print(&mut self, pid: PID, content: String);
    fn stdin_scan(
        &mut self,
        tid: TID,
    ) -> Option<String>;
    fn stdin_request(
        &mut self,
        tid: TID,
    );
    fn stdasm_print(&mut self, pid: PID, content: String);
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
    fn spawn(&mut self, pid: &Self::PID) -> Result<Self::TID, RuntimeError>;
    fn close(&mut self, pid: &Self::PID, tid: &Self::TID) -> Result<(), RuntimeError>;
}

pub trait Engine:
    ExternIO<Self::PID, Self::TID>
    + ExternPathFinder
    + ExternEnergyDispenser<Self::PID, Self::TID>
    + ExternThreadHandler
    + Sized
{
    type Function: ExternFunction<Self>;
    type FunctionContext: ExternExecutionContext;
}

pub trait ExternExecutionContext: Default + Debug + Clone + Copy {}

pub trait ExternThreadIdentifier<PID: ExternProcessIdentifier>:
    Hash + PartialEq + std::cmp::Eq + Clone + Sized + Debug + Default + Copy
{
    fn to_u64(&self) -> u64;
    fn from_u64(tid: u64) -> Option<Self>;
    fn pid(&self) -> PID;
}

pub trait ExternProcessIdentifier:
    Hash + PartialEq + std::cmp::Eq + Clone + Sized + Debug + Default + Copy
{
}
