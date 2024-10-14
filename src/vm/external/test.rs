use crate::vm::{asm::operation::PopNum, scheduler::Executable, AsmName, AsmWeight};

use super::{
    Engine, ExternEnergyDispenser, ExternExecutionContext, ExternFunction, ExternIO,
    ExternPathFinder, ExternProcessIdentifier, ExternResolve, ExternThreadHandler,
    ExternThreadIdentifier,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultThreadID(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultProcessID(pub u64);

#[derive(Debug, Clone, PartialEq)]
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

pub struct DefaultExternFunction;

impl<E: Engine> AsmName<E> for DefaultExternFunction {
    fn name(
        &self,
        stdio: &mut crate::vm::stdio::StdIO,
        program: &crate::vm::program::Program<E>,
        engine: &mut E,
    ) {
        unimplemented!()
    }
}

impl AsmWeight for DefaultExternFunction {}

impl<E: Engine> Executable<E> for DefaultExternFunction {
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

impl ExternResolve for DefaultExternFunction {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        params: &mut Vec<crate::ast::expressions::Expression>,
    ) -> Result<crate::semantic::EType, crate::semantic::SemanticError> {
        unimplemented!()
    }
}
impl<E: Engine> ExternFunction<E> for DefaultExternFunction {}

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
    type Function = DefaultExternFunction;
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
    type Function = DefaultExternFunction;
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
    type Function = DefaultExternFunction;
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

impl Engine for DbgEngine {
    type Function = DefaultExternFunction;
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
    type Function = DefaultExternFunction;
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
impl<E: Engine> ExternFunction<E> for ExternFuncTest {}

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

// #[derive(Debug, Clone)]
// pub struct TestDynamicEngine {
//     pub dynamic_fn_provider: TestDynamicFnProvider,
//     pub out: String,
// }

// #[derive(Debug, Clone)]
// pub struct TestDynamicFnProvider {}

// impl DynamicFnProvider for TestDynamicFnProvider {
//     type DynamicFunctions = TestDynamicFn;
//     fn get_dynamic_fn(prefix: &Option<ID>, id: &str) -> Option<Self::DynamicFunctions> {
//         if "dynamic_fn" == id {
//             return Some(TestDynamicFn {});
//         } else {
//             return None;
//         }
//     }
// }
// pub struct TestDynamicFn {}

// impl DynamicFnResolver for TestDynamicFn {
//     fn resolve<E: crate::vm::external::Engine>(
//         &mut self,
//         scope: &mut semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//         params: &mut Vec<crate::ast::expressions::Expression>,
//     ) -> Result<EType, SemanticError> {
//         Ok(e_static!(StaticType::Unit))
//     }
// }

// impl<E: crate::vm::external::Engine> Executable<E> for TestDynamicFn {
//     fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
//         &self,
//         program: &crate::vm::program::Program<E>,
//         scheduler: &mut crate::vm::scheduler::Scheduler<P>,
//         stack: &mut crate::vm::allocator::stack::Stack,
//         heap: &mut crate::vm::allocator::heap::Heap,
//         stdio: &mut crate::vm::stdio::StdIO,
//         engine: &mut E,
//         context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID: ExternProcessIdentifier, E::TID>,
//     ) -> Result<(), RuntimeError> {
//         stdio.stdout.push("\"Hello World from Dynamic function\"");
//         stdio.stdout.flush(engine);
//         scheduler.next();
//         Ok(())
//     }
// }

// impl ExternIO for TestDynamicEngine {
//     fn stdout_print(&mut self, content: String) {
//         self.out = content;
//     }
//     fn stdout_println(&mut self, content: String) {
//         self.out = format!("{}\n", content);
//     }
//     fn stderr_print(&mut self, content: String) {}

//     fn stdin_scan(&mut self) -> Option<String> {
//         None
//     }
//     fn stdin_request(&mut self) {}

//     fn stdasm_print(&mut self, content: String) {}

//     fn execute_dynamic_fn(
//         fn_id: String,
//         program: &mut crate::vm::program::Program,
//         stack: &mut Stack,
//         heap: &mut Heap,
//         stdio: &mut StdIO,
//         engine: &mut Self,
//         tid: usize,
//     ) -> Result<(), RuntimeError> {
//         if let Some(dynamic_fn) = TestDynamicFnProvider::get_dynamic_fn(&None, &fn_id) {
//             dynamic_fn.execute(program, scheduler, signal_handler, stack, heap, stdio, engine, context)?;
//         }
//         Ok(())
//     }
//     fn is_dynamic_fn(preffixe: &Option<ID>, id: &ID) -> Option<impl DynamicFnResolver> {
//         TestDynamicFnProvider::get_dynamic_fn(preffixe, id.to_string().as_str())
//     }
//     fn name_of_dynamic_fn(
//         fn_id: String,
//         stdio: &mut StdIO,
//         program: &mut crate::vm::program::Program,
//         engine: &mut Self,
//     ) {
//         stdio.push_asm_lib(engine, &fn_id);
//     }
//     fn weight_of_dynamic_fn(fn_id: String) -> Weight {
//         Weight::LOW
//     }
// }
