use crate::{
    p_num,
    vm::{
        allocator::MemoryAddress, asm::operation::PopNum, scheduler::Executable, AsmName, AsmWeight,
    },
};

use super::{
    Engine, ExternEnergyDispenser, ExternEventManager, ExternExecutionContext, ExternFunction,
    ExternIO, ExternPathFinder, ExternProcessIdentifier, ExternResolve, ExternThreadHandler,
    ExternThreadIdentifier,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct DefaultThreadID(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct DefaultProcessID(pub u64);

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct DefaultExecutionContext {}

impl Default for DefaultExecutionContext {
    fn default() -> Self {
        Self {}
    }
}

impl Default for DefaultThreadID {
    fn default() -> Self {
        Self(0)
    }
}

impl Default for DefaultProcessID {
    fn default() -> Self {
        Self(0)
    }
}

impl ExternProcessIdentifier for DefaultProcessID {}

impl ExternThreadIdentifier<DefaultProcessID> for DefaultThreadID {
    fn to_u64(&self) -> u64 {
        self.0
    }

    fn from_u64(tid: u64) -> Option<Self> {
        Some(Self(tid))
    }

    fn pid(&self) -> DefaultProcessID {
        DefaultProcessID::default()
    }
}
impl ExternExecutionContext for DefaultExecutionContext {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DefaultExternFunctionNoopEngine;

impl<E: Engine> AsmName<E> for DefaultExternFunctionNoopEngine {
    fn name(
        &self,
        stdio: &mut crate::vm::stdio::StdIO,
        program: &crate::vm::program::Program<E>,
        engine: &mut E,
    ) {
        unimplemented!()
    }
}

impl AsmWeight for DefaultExternFunctionNoopEngine {}
impl ExternEventManager<DefaultExecutionContext, DefaultProcessID, DefaultThreadID>
    for DefaultExternFunctionNoopEngine
{
    type E = NoopEngine;
}

impl<E: Engine> Executable<E> for DefaultExternFunctionNoopEngine {
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
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternResolve for DefaultExternFunctionNoopEngine {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        params: &mut Vec<crate::ast::expressions::Expression>,
    ) -> Result<crate::semantic::EType, crate::semantic::SemanticError> {
        unimplemented!()
    }
}
impl ExternFunction<NoopEngine> for DefaultExternFunctionNoopEngine {}

#[derive(Debug, Clone)]
pub struct NoopEngine {}

impl ExternIO for NoopEngine {
    fn stdout_print(&mut self, content: String) {}
    fn stdout_println(&mut self, content: String) {}

    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) -> Option<String> {
        None
    }
    fn stdin_request<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) {
    }

    fn stdasm_print(&mut self, content: String) {}
}

impl Engine for NoopEngine {
    type Function = DefaultExternFunctionNoopEngine;
    type FunctionContext = DefaultExecutionContext;
}

impl ExternThreadHandler for NoopEngine {
    type TID = DefaultThreadID;
    type PID = DefaultProcessID;

    fn spawn(&mut self) -> Result<Self::TID, crate::vm::runtime::RuntimeError> {
        Ok(DefaultThreadID(1))
    }

    fn close(&mut self, tid: &Self::TID) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> ExternEnergyDispenser<PID, TID>
    for NoopEngine
{
    fn get_energy(&self, pid: PID) -> usize {
        unimplemented!()
    }

    fn consume_energy(
        &mut self,
        energy: usize,
        pid: PID,
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternPathFinder for NoopEngine {
    fn find(path: &[String], name: &str) -> Option<<Self as Engine>::Function>
    where
        Self: super::Engine,
    {
        None
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DefaultExternFunctionStdoutTestEngine;

impl<E: Engine> AsmName<E> for DefaultExternFunctionStdoutTestEngine {
    fn name(
        &self,
        stdio: &mut crate::vm::stdio::StdIO,
        program: &crate::vm::program::Program<E>,
        engine: &mut E,
    ) {
        unimplemented!()
    }
}

impl AsmWeight for DefaultExternFunctionStdoutTestEngine {}
impl ExternEventManager<DefaultExecutionContext, DefaultProcessID, DefaultThreadID>
    for DefaultExternFunctionStdoutTestEngine
{
    type E = StdoutTestEngine;
}

impl<E: Engine> Executable<E> for DefaultExternFunctionStdoutTestEngine {
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
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternResolve for DefaultExternFunctionStdoutTestEngine {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        params: &mut Vec<crate::ast::expressions::Expression>,
    ) -> Result<crate::semantic::EType, crate::semantic::SemanticError> {
        unimplemented!()
    }
}
impl ExternFunction<StdoutTestEngine> for DefaultExternFunctionStdoutTestEngine {}

#[derive(Debug, Clone)]
pub struct StdoutTestEngine {
    pub out: String,
}

impl ExternIO for StdoutTestEngine {
    fn stdout_print(&mut self, content: String) {
        self.out = content;
    }
    fn stdout_println(&mut self, content: String) {
        self.out = format!("{}\n", content);
    }
    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) -> Option<String> {
        None
    }
    fn stdin_request<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) {
    }

    fn stdasm_print(&mut self, content: String) {
        // println!("{}", content);
    }
}

impl Engine for StdoutTestEngine {
    type Function = DefaultExternFunctionStdoutTestEngine;
    type FunctionContext = DefaultExecutionContext;
}

impl<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> ExternEnergyDispenser<PID, TID>
    for StdoutTestEngine
{
    fn get_energy(&self, pid: PID) -> usize {
        unimplemented!()
    }

    fn consume_energy(
        &mut self,
        energy: usize,
        pid: PID,
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternThreadHandler for StdoutTestEngine {
    type TID = DefaultThreadID;
    type PID = DefaultProcessID;

    fn spawn(&mut self) -> Result<Self::TID, crate::vm::runtime::RuntimeError> {
        Ok(DefaultThreadID(1))
    }

    fn close(&mut self, tid: &Self::TID) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternPathFinder for StdoutTestEngine {
    fn find(path: &[String], name: &str) -> Option<<Self as Engine>::Function>
    where
        Self: super::Engine,
    {
        unimplemented!()
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DefaultExternFunctionStdinTestEngine;

impl<E: Engine> AsmName<E> for DefaultExternFunctionStdinTestEngine {
    fn name(
        &self,
        stdio: &mut crate::vm::stdio::StdIO,
        program: &crate::vm::program::Program<E>,
        engine: &mut E,
    ) {
        unimplemented!()
    }
}

impl AsmWeight for DefaultExternFunctionStdinTestEngine {}
impl ExternEventManager<DefaultExecutionContext, DefaultProcessID, DefaultThreadID>
    for DefaultExternFunctionStdinTestEngine
{
    type E = StdinTestEngine;
}

impl<E: Engine> Executable<E> for DefaultExternFunctionStdinTestEngine {
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
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternResolve for DefaultExternFunctionStdinTestEngine {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        params: &mut Vec<crate::ast::expressions::Expression>,
    ) -> Result<crate::semantic::EType, crate::semantic::SemanticError> {
        unimplemented!()
    }
}
impl ExternFunction<StdinTestEngine> for DefaultExternFunctionStdinTestEngine {}

#[derive(Debug, Clone)]
pub struct StdinTestEngine {
    pub out: String,
    pub in_buf: String,
}

impl ExternIO for StdinTestEngine {
    fn stdout_print(&mut self, content: String) {
        self.out = content;
    }
    fn stdout_println(&mut self, content: String) {
        self.out = format!("{}\n", content);
    }
    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) -> Option<String> {
        if self.in_buf.is_empty() {
            None
        } else {
            Some(self.in_buf.clone())
        }
    }
    fn stdin_request<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) {
    }

    fn stdasm_print(&mut self, content: String) {
        println!("{}", content);
    }
}

impl Engine for StdinTestEngine {
    type Function = DefaultExternFunctionStdinTestEngine;
    type FunctionContext = DefaultExecutionContext;
}

impl<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> ExternEnergyDispenser<PID, TID>
    for StdinTestEngine
{
    fn get_energy(&self, pid: PID) -> usize {
        unimplemented!()
    }

    fn consume_energy(
        &mut self,
        energy: usize,
        pid: PID,
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternThreadHandler for StdinTestEngine {
    type TID = DefaultThreadID;
    type PID = DefaultProcessID;

    fn spawn(&mut self) -> Result<Self::TID, crate::vm::runtime::RuntimeError> {
        Ok(DefaultThreadID(0))
    }

    fn close(&mut self, tid: &Self::TID) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternPathFinder for StdinTestEngine {
    fn find(path: &[String], name: &str) -> Option<<Self as Engine>::Function>
    where
        Self: super::Engine,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct DbgEngine {}

impl ExternIO for DbgEngine {
    fn stdout_print(&mut self, content: String) {
        print!("{}", content);
    }
    fn stdout_println(&mut self, content: String) {
        println!("{}", content);
    }

    fn stderr_print(&mut self, content: String) {
        eprintln!("{}", content);
    }

    fn stdin_scan<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) -> Option<String> {
        Some("Hello World".to_string())
    }
    fn stdin_request<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) {
    }

    fn stdasm_print(&mut self, content: String) {
        println!("{}", content);
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DefaultExternFunctionDbgEngine;

impl<E: Engine> AsmName<E> for DefaultExternFunctionDbgEngine {
    fn name(
        &self,
        stdio: &mut crate::vm::stdio::StdIO,
        program: &crate::vm::program::Program<E>,
        engine: &mut E,
    ) {
        unimplemented!()
    }
}

impl AsmWeight for DefaultExternFunctionDbgEngine {}
impl ExternEventManager<DefaultExecutionContext, DefaultProcessID, DefaultThreadID>
    for DefaultExternFunctionDbgEngine
{
    type E = DbgEngine;
}

impl<E: Engine> Executable<E> for DefaultExternFunctionDbgEngine {
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
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternResolve for DefaultExternFunctionDbgEngine {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        params: &mut Vec<crate::ast::expressions::Expression>,
    ) -> Result<crate::semantic::EType, crate::semantic::SemanticError> {
        unimplemented!()
    }
}
impl ExternFunction<DbgEngine> for DefaultExternFunctionDbgEngine {}

impl Engine for DbgEngine {
    type Function = DefaultExternFunctionDbgEngine;
    type FunctionContext = DefaultExecutionContext;
}

impl<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> ExternEnergyDispenser<PID, TID>
    for DbgEngine
{
    fn get_energy(&self, pid: PID) -> usize {
        unimplemented!()
    }

    fn consume_energy(
        &mut self,
        energy: usize,
        pid: PID,
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternThreadHandler for DbgEngine {
    type TID = DefaultThreadID;
    type PID = DefaultProcessID;

    fn spawn(&mut self) -> Result<Self::TID, crate::vm::runtime::RuntimeError> {
        Ok(DefaultThreadID(1))
    }

    fn close(&mut self, tid: &Self::TID) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternPathFinder for DbgEngine {
    fn find(path: &[String], name: &str) -> Option<<Self as Engine>::Function>
    where
        Self: super::Engine,
    {
        unimplemented!()
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DefaultExternFunctionThreadTestEngine;

impl<E: Engine> AsmName<E> for DefaultExternFunctionThreadTestEngine {
    fn name(
        &self,
        stdio: &mut crate::vm::stdio::StdIO,
        program: &crate::vm::program::Program<E>,
        engine: &mut E,
    ) {
        unimplemented!()
    }
}

impl AsmWeight for DefaultExternFunctionThreadTestEngine {}
impl ExternEventManager<DefaultExecutionContext, DefaultProcessID, DefaultThreadID>
    for DefaultExternFunctionThreadTestEngine
{
    type E = ThreadTestEngine;
}

impl<E: Engine> Executable<E> for DefaultExternFunctionThreadTestEngine {
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
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternResolve for DefaultExternFunctionThreadTestEngine {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        params: &mut Vec<crate::ast::expressions::Expression>,
    ) -> Result<crate::semantic::EType, crate::semantic::SemanticError> {
        unimplemented!()
    }
}
impl ExternFunction<ThreadTestEngine> for DefaultExternFunctionThreadTestEngine {}

#[derive(Debug, Clone)]
pub struct ThreadTestEngine {
    pub id_auto_increment: u64,
}

impl ExternIO for ThreadTestEngine {
    fn stdout_print(&mut self, content: String) {}
    fn stdout_println(&mut self, content: String) {}

    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) -> Option<String> {
        None
    }
    fn stdin_request<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) {
    }

    fn stdasm_print(&mut self, content: String) {
        println!("{}", content);
    }
}

impl Engine for ThreadTestEngine {
    type Function = DefaultExternFunctionThreadTestEngine;
    type FunctionContext = DefaultExecutionContext;
}

impl<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> ExternEnergyDispenser<PID, TID>
    for ThreadTestEngine
{
    fn get_energy(&self, pid: PID) -> usize {
        unimplemented!()
    }

    fn consume_energy(
        &mut self,
        energy: usize,
        pid: PID,
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternThreadHandler for ThreadTestEngine {
    type TID = DefaultThreadID;
    type PID = DefaultProcessID;

    fn spawn(&mut self) -> Result<Self::TID, crate::vm::runtime::RuntimeError> {
        self.id_auto_increment += 1;
        Ok(DefaultThreadID(self.id_auto_increment))
    }

    fn close(&mut self, tid: &Self::TID) -> Result<(), crate::vm::runtime::RuntimeError> {
        Ok(())
    }
}

impl ExternPathFinder for ThreadTestEngine {
    fn find(path: &[String], name: &str) -> Option<<Self as Engine>::Function>
    where
        Self: super::Engine,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct ExternFuncTestEngine {}

impl ExternIO for ExternFuncTestEngine {
    fn stdout_print(&mut self, content: String) {}
    fn stdout_println(&mut self, content: String) {}

    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) -> Option<String> {
        None
    }
    fn stdin_request<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) {
    }

    fn stdasm_print(&mut self, content: String) {}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExternFuncTest {
    TEST_ADDER,
}

impl<E: Engine> AsmName<E> for ExternFuncTest {
    fn name(
        &self,
        stdio: &mut crate::vm::stdio::StdIO,
        program: &crate::vm::program::Program<E>,
        engine: &mut E,
    ) {
        match self {
            ExternFuncTest::TEST_ADDER => stdio.push_asm_lib(engine, "test_adder"),
        }
    }
}

impl AsmWeight for ExternFuncTest {}
impl ExternEventManager<DefaultExecutionContext, DefaultProcessID, DefaultThreadID>
    for ExternFuncTest
{
    type E = ExternFuncTestEngine;
}

impl<E: Engine> Executable<E> for ExternFuncTest {
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
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        match self {
            ExternFuncTest::TEST_ADDER => {
                let x = crate::vm::asm::operation::OpPrimitive::pop_num::<u64>(stack)?;
                let y = crate::vm::asm::operation::OpPrimitive::pop_num::<u64>(stack)?;

                let _ = stack.push_with(&(x + y).to_le_bytes())?;

                scheduler.next();
                Ok(())
            }
        }
    }
}

impl ExternResolve for ExternFuncTest {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        params: &mut Vec<crate::ast::expressions::Expression>,
    ) -> Result<crate::semantic::EType, crate::semantic::SemanticError> {
        match self {
            ExternFuncTest::TEST_ADDER => {
                if params.len() != 2 {
                    return Err(crate::semantic::SemanticError::IncorrectArguments);
                }
                let (first_part, second_part) = params.split_at_mut(1);
                let x = &mut first_part[0];
                let y = &mut second_part[0];

                let _ = crate::semantic::Resolve::resolve::<E>(
                    x,
                    scope_manager,
                    scope_id,
                    &Some(crate::p_num!(U64)),
                    &mut None,
                )?;
                let _ = crate::semantic::Resolve::resolve::<E>(
                    y,
                    scope_manager,
                    scope_id,
                    &Some(crate::p_num!(U64)),
                    &mut None,
                )?;

                let crate::p_num!(U64) =
                    crate::semantic::TypeOf::type_of(x, &scope_manager, scope_id)?
                else {
                    return Err(crate::semantic::SemanticError::IncorrectArguments);
                };
                let crate::p_num!(U64) =
                    crate::semantic::TypeOf::type_of(y, &scope_manager, scope_id)?
                else {
                    return Err(crate::semantic::SemanticError::IncorrectArguments);
                };

                Ok(crate::semantic::EType::Static(
                    crate::semantic::scope::static_types::StaticType::Primitive(
                        crate::semantic::scope::static_types::PrimitiveType::Number(
                            crate::semantic::scope::static_types::NumberType::U64,
                        ),
                    ),
                ))
            }
        }
    }
}

impl ExternFunction<ExternFuncTestEngine> for ExternFuncTest {}

impl Engine for ExternFuncTestEngine {
    type Function = ExternFuncTest;
    type FunctionContext = DefaultExecutionContext;
}

impl ExternThreadHandler for ExternFuncTestEngine {
    type TID = DefaultThreadID;
    type PID = DefaultProcessID;

    fn spawn(&mut self) -> Result<Self::TID, crate::vm::runtime::RuntimeError> {
        Ok(DefaultThreadID(1))
    }

    fn close(&mut self, tid: &Self::TID) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> ExternEnergyDispenser<PID, TID>
    for ExternFuncTestEngine
{
    fn get_energy(&self, pid: PID) -> usize {
        unimplemented!()
    }

    fn consume_energy(
        &mut self,
        energy: usize,
        pid: PID,
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternPathFinder for ExternFuncTestEngine {
    fn find(path: &[String], name: &str) -> Option<<Self as Engine>::Function>
    where
        Self: super::Engine,
    {
        if name == "test_adder" {
            Some(ExternFuncTest::TEST_ADDER)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExternEventTestEngine {}

impl ExternIO for ExternEventTestEngine {
    fn stdout_print(&mut self, content: String) {}
    fn stdout_println(&mut self, content: String) {}

    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) -> Option<String> {
        None
    }
    fn stdin_request<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
        &mut self,
        tid: TID,
    ) {
    }

    fn stdasm_print(&mut self, content: String) {
        println!("{}", content);
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExternFuncEventTest {
    TEST_EVENT,
    TEST_EVENT_WITH_ARG,
    TEST_EVENT_WITH_RETURN,
}

impl<E: Engine> AsmName<E> for ExternFuncEventTest {
    fn name(
        &self,
        stdio: &mut crate::vm::stdio::StdIO,
        program: &crate::vm::program::Program<E>,
        engine: &mut E,
    ) {
        match self {
            ExternFuncEventTest::TEST_EVENT => stdio.push_extern_lib(engine, "test_event"),
            ExternFuncEventTest::TEST_EVENT_WITH_ARG => {
                stdio.push_extern_lib(engine, "test_event_with_arg")
            }
            ExternFuncEventTest::TEST_EVENT_WITH_RETURN => {
                stdio.push_extern_lib(engine, "test_event_with_return")
            }
        }
    }
}

impl ExternEventManager<DefaultExecutionContext, DefaultProcessID, DefaultThreadID>
    for ExternFuncEventTest
{
    type E = ExternEventTestEngine;

    fn event_conclusion(
        &self,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut Self::E,
        context: &crate::vm::scheduler::ExecutionContext<
            DefaultExecutionContext,
            DefaultProcessID,
            DefaultThreadID,
        >,
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        match self {
            ExternFuncEventTest::TEST_EVENT => {}
            ExternFuncEventTest::TEST_EVENT_WITH_ARG => {}
            ExternFuncEventTest::TEST_EVENT_WITH_RETURN => {
                let res = crate::vm::asm::operation::OpPrimitive::pop_num::<u64>(stack)?;
                assert_eq!(res, 420);
            }
        }
        Ok(())
    }

    fn event_setup(
        &self,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut Self::E,
        context: &crate::vm::scheduler::ExecutionContext<
            DefaultExecutionContext,
            DefaultProcessID,
            DefaultThreadID,
        >,
    ) -> Result<usize, crate::vm::runtime::RuntimeError> {
        stack.push_with(&(69u64).to_le_bytes())?;
        match self {
            ExternFuncEventTest::TEST_EVENT => Ok(0),
            ExternFuncEventTest::TEST_EVENT_WITH_ARG => Ok(8),
            ExternFuncEventTest::TEST_EVENT_WITH_RETURN => Ok(0),
        }
    }

    fn event_trigger(&self, signal: u64, trigger: u64) -> bool {
        signal & trigger != 0
    }
}

impl AsmWeight for ExternFuncEventTest {
    fn weight(&self) -> crate::vm::Weight {
        crate::vm::Weight::END
    }
}

impl<
        E: Engine<
            FunctionContext = DefaultExecutionContext,
            PID = DefaultProcessID,
            TID = DefaultThreadID,
            Function = ExternFuncEventTest,
        >,
    > Executable<E> for ExternFuncEventTest
{
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
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        match self {
            ExternFuncEventTest::TEST_EVENT
            | ExternFuncEventTest::TEST_EVENT_WITH_ARG
            | ExternFuncEventTest::TEST_EVENT_WITH_RETURN => {
                let callback: MemoryAddress =
                    crate::vm::asm::operation::OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                fn signal_callback<E: crate::vm::external::Engine>(
                    response: crate::vm::signal::SignalResult<E>,
                    stack: &mut crate::vm::allocator::stack::Stack,
                ) -> Result<(), crate::vm::runtime::RuntimeError> {
                    Ok(())
                }
                let _ = signal_handler.notify(
                    crate::vm::signal::Signal::EventRegistration {
                        tid: context.tid.clone(),
                        trigger: 1,
                        callback: crate::vm::scheduler::EventCallback {
                            callback,
                            manager: *self,
                            _phantom: std::marker::PhantomData::default(),
                        },
                        conf: crate::vm::scheduler::EventConf {
                            kind: crate::vm::scheduler::EventKind::Once,
                            exclu: crate::vm::scheduler::EventExclusivity::PerPID,
                        },
                    },
                    stack,
                    engine,
                    context.tid.clone(),
                    signal_callback::<E>,
                )?;
                scheduler.next();
                Ok(())
            }
        }
    }
}

impl ExternResolve for ExternFuncEventTest {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        params: &mut Vec<crate::ast::expressions::Expression>,
    ) -> Result<crate::semantic::EType, crate::semantic::SemanticError> {
        match self {
            ExternFuncEventTest::TEST_EVENT => {
                if params.len() != 1 {
                    return Err(crate::semantic::SemanticError::IncorrectArguments);
                }
                let callback = &mut params[0];

                let _ = crate::semantic::Resolve::resolve::<E>(
                    callback,
                    scope_manager,
                    scope_id,
                    &Some(crate::semantic::EType::Static(
                        crate::semantic::scope::static_types::StaticType::Closure(
                            crate::semantic::scope::static_types::ClosureType {
                                params: Vec::default(),
                                ret: crate::semantic::EType::Static(
                                    crate::semantic::scope::static_types::StaticType::Unit,
                                )
                                .into(),
                            },
                        ),
                    )),
                    &mut None,
                )?;

                let callback_type =
                    crate::semantic::TypeOf::type_of(callback, &scope_manager, scope_id)?;

                match callback_type {
                    crate::semantic::EType::Static(
                        crate::semantic::scope::static_types::StaticType::Closure(
                            crate::semantic::scope::static_types::ClosureType { params, ret },
                        ),
                    ) => {
                        if params.len() != 0
                            || crate::semantic::EType::Static(
                                crate::semantic::scope::static_types::StaticType::Unit,
                            ) != *ret
                        {
                            return Err(crate::semantic::SemanticError::IncorrectArguments);
                        }
                    }
                    _ => return Err(crate::semantic::SemanticError::IncorrectArguments),
                }

                Ok(crate::semantic::EType::Static(
                    crate::semantic::scope::static_types::StaticType::Unit,
                ))
            }
            ExternFuncEventTest::TEST_EVENT_WITH_ARG => {
                if params.len() != 1 {
                    return Err(crate::semantic::SemanticError::IncorrectArguments);
                }
                let callback = &mut params[0];

                let _ = crate::semantic::Resolve::resolve::<E>(
                    callback,
                    scope_manager,
                    scope_id,
                    &Some(crate::semantic::EType::Static(
                        crate::semantic::scope::static_types::StaticType::Closure(
                            crate::semantic::scope::static_types::ClosureType {
                                params: vec![p_num!(I64)],
                                ret: crate::semantic::EType::Static(
                                    crate::semantic::scope::static_types::StaticType::Unit,
                                )
                                .into(),
                            },
                        ),
                    )),
                    &mut None,
                )?;

                let callback_type =
                    crate::semantic::TypeOf::type_of(callback, &scope_manager, scope_id)?;

                match callback_type {
                    crate::semantic::EType::Static(
                        crate::semantic::scope::static_types::StaticType::Closure(
                            crate::semantic::scope::static_types::ClosureType { params, ret },
                        ),
                    ) => {
                        if params.len() != 1
                            || crate::semantic::EType::Static(
                                crate::semantic::scope::static_types::StaticType::Unit,
                            ) != *ret
                        {
                            return Err(crate::semantic::SemanticError::IncorrectArguments);
                        }
                    }
                    _ => return Err(crate::semantic::SemanticError::IncorrectArguments),
                }

                Ok(crate::semantic::EType::Static(
                    crate::semantic::scope::static_types::StaticType::Unit,
                ))
            }
            ExternFuncEventTest::TEST_EVENT_WITH_RETURN => {
                if params.len() != 1 {
                    return Err(crate::semantic::SemanticError::IncorrectArguments);
                }
                let callback = &mut params[0];

                let _ = crate::semantic::Resolve::resolve::<E>(
                    callback,
                    scope_manager,
                    scope_id,
                    &Some(crate::semantic::EType::Static(
                        crate::semantic::scope::static_types::StaticType::Closure(
                            crate::semantic::scope::static_types::ClosureType {
                                params: Vec::default(),
                                ret: p_num!(I64).into(),
                            },
                        ),
                    )),
                    &mut None,
                )?;

                let callback_type =
                    crate::semantic::TypeOf::type_of(callback, &scope_manager, scope_id)?;

                match callback_type {
                    crate::semantic::EType::Static(
                        crate::semantic::scope::static_types::StaticType::Closure(
                            crate::semantic::scope::static_types::ClosureType { params, ret },
                        ),
                    ) => {
                        if params.len() != 0 || p_num!(I64) != *ret {
                            return Err(crate::semantic::SemanticError::IncorrectArguments);
                        }
                    }
                    _ => return Err(crate::semantic::SemanticError::IncorrectArguments),
                }

                Ok(crate::semantic::EType::Static(
                    crate::semantic::scope::static_types::StaticType::Unit,
                ))
            }
        }
    }
}

impl ExternFunction<ExternEventTestEngine> for ExternFuncEventTest {}

impl Engine for ExternEventTestEngine {
    type Function = ExternFuncEventTest;
    type FunctionContext = DefaultExecutionContext;
}

impl ExternThreadHandler for ExternEventTestEngine {
    type TID = DefaultThreadID;
    type PID = DefaultProcessID;

    fn spawn(&mut self) -> Result<Self::TID, crate::vm::runtime::RuntimeError> {
        Ok(DefaultThreadID(1))
    }

    fn close(&mut self, tid: &Self::TID) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>> ExternEnergyDispenser<PID, TID>
    for ExternEventTestEngine
{
    fn get_energy(&self, pid: PID) -> usize {
        unimplemented!()
    }

    fn consume_energy(
        &mut self,
        energy: usize,
        pid: PID,
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternPathFinder for ExternEventTestEngine {
    fn find(path: &[String], name: &str) -> Option<<Self as Engine>::Function>
    where
        Self: super::Engine,
    {
        match name {
            "test_event" => Some(ExternFuncEventTest::TEST_EVENT),
            "test_event_with_arg" => Some(ExternFuncEventTest::TEST_EVENT_WITH_ARG),
            "test_event_with_return" => Some(ExternFuncEventTest::TEST_EVENT_WITH_RETURN),
            _ => None,
        }
    }
}
