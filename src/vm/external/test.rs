use crate::vm::{scheduler::Executable, AsmName, AsmWeight};

use super::{
    Engine, ExternEnergyDispenser, ExternExecutionContext, ExternFunction, ExternIO,
    ExternPathFinder, ExternThreadHandler, ExternThreadIdentifier,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultThreadID(pub u64);

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

impl ExternThreadIdentifier for DefaultThreadID {
    fn to_u64(&self) -> u64 {
        self.0
    }

    fn from_u64(tid: u64) -> Self {
        Self(tid)
    }
    // fn gen<E: ExternThreadHandler>(engine: &mut E) -> Self {
    //     DefaultThreadID(2)
    // }
}

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
        signal_handler: &mut crate::vm::runtime::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::TID>,
    ) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternExecutionContext for DefaultExecutionContext {}

impl<E: Engine> ExternFunction<E> for DefaultExternFunction {}

#[derive(Debug, Clone)]
pub struct NoopGameEngine {}

impl ExternIO for NoopGameEngine {
    fn stdout_print(&mut self, content: String) {}
    fn stdout_println(&mut self, content: String) {}

    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan(&mut self) -> Option<String> {
        None
    }
    fn stdin_request(&mut self) {}

    fn stdasm_print(&mut self, content: String) {}
}

impl Engine for NoopGameEngine {
    type Function = DefaultExternFunction;
    type FunctionContext = DefaultExecutionContext;
}

impl ExternThreadHandler for NoopGameEngine {
    type TID = DefaultThreadID;

    fn spawn(&mut self) -> Result<Self::TID, crate::vm::runtime::RuntimeError> {
        Ok(DefaultThreadID(1))
    }

    fn close(&mut self, tid: &Self::TID) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternEnergyDispenser for NoopGameEngine {
    fn get_energy(&self) -> usize {
        unimplemented!()
    }

    fn consume_energy(&mut self, energy: usize) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternPathFinder for NoopGameEngine {
    fn find(path: &[String], name: &str) -> Option<<Self as Engine>::Function>
    where
        Self: super::Engine,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct StdoutTestGameEngine {
    pub out: String,
}

impl ExternIO for StdoutTestGameEngine {
    fn stdout_print(&mut self, content: String) {
        self.out = content;
    }
    fn stdout_println(&mut self, content: String) {
        self.out = format!("{}\n", content);
    }
    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan(&mut self) -> Option<String> {
        None
    }
    fn stdin_request(&mut self) {}

    fn stdasm_print(&mut self, content: String) {
        // println!("{}", content);
    }
}

impl Engine for StdoutTestGameEngine {
    type Function = DefaultExternFunction;
    type FunctionContext = DefaultExecutionContext;
}

impl ExternEnergyDispenser for StdoutTestGameEngine {
    fn get_energy(&self) -> usize {
        unimplemented!()
    }

    fn consume_energy(&mut self, energy: usize) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternThreadHandler for StdoutTestGameEngine {
    type TID = DefaultThreadID;

    fn spawn(&mut self) -> Result<Self::TID, crate::vm::runtime::RuntimeError> {
        Ok(DefaultThreadID(1))
    }

    fn close(&mut self, tid: &Self::TID) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternPathFinder for StdoutTestGameEngine {
    fn find(path: &[String], name: &str) -> Option<<Self as Engine>::Function>
    where
        Self: super::Engine,
    {
        unimplemented!()
    }
}
#[derive(Debug, Clone)]
pub struct StdinTestGameEngine {
    pub out: String,
    pub in_buf: String,
}

impl ExternIO for StdinTestGameEngine {
    fn stdout_print(&mut self, content: String) {
        self.out = content;
    }
    fn stdout_println(&mut self, content: String) {
        self.out = format!("{}\n", content);
    }
    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan(&mut self) -> Option<String> {
        if self.in_buf.is_empty() {
            None
        } else {
            Some(self.in_buf.clone())
        }
    }
    fn stdin_request(&mut self) {}

    fn stdasm_print(&mut self, content: String) {
        println!("{}", content);
    }
}

impl Engine for StdinTestGameEngine {
    type Function = DefaultExternFunction;
    type FunctionContext = DefaultExecutionContext;
}

impl ExternEnergyDispenser for StdinTestGameEngine {
    fn get_energy(&self) -> usize {
        unimplemented!()
    }

    fn consume_energy(&mut self, energy: usize) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternThreadHandler for StdinTestGameEngine {
    type TID = DefaultThreadID;

    fn spawn(&mut self) -> Result<Self::TID, crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }

    fn close(&mut self, tid: &Self::TID) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternPathFinder for StdinTestGameEngine {
    fn find(path: &[String], name: &str) -> Option<<Self as Engine>::Function>
    where
        Self: super::Engine,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct DbgGameEngine {}

impl ExternIO for DbgGameEngine {
    fn stdout_print(&mut self, content: String) {
        print!("{}", content);
    }
    fn stdout_println(&mut self, content: String) {
        println!("{}", content);
    }

    fn stderr_print(&mut self, content: String) {
        eprintln!("{}", content);
    }

    fn stdin_scan(&mut self) -> Option<String> {
        Some("Hello World".to_string())
    }
    fn stdin_request(&mut self) {}

    fn stdasm_print(&mut self, content: String) {
        println!("{}", content);
    }
}

impl Engine for DbgGameEngine {
    type Function = DefaultExternFunction;
    type FunctionContext = DefaultExecutionContext;
}

impl ExternEnergyDispenser for DbgGameEngine {
    fn get_energy(&self) -> usize {
        unimplemented!()
    }

    fn consume_energy(&mut self, energy: usize) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternThreadHandler for DbgGameEngine {
    type TID = DefaultThreadID;

    fn spawn(&mut self) -> Result<Self::TID, crate::vm::runtime::RuntimeError> {
        Ok(DefaultThreadID(1))
    }

    fn close(&mut self, tid: &Self::TID) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternPathFinder for DbgGameEngine {
    fn find(path: &[String], name: &str) -> Option<<Self as Engine>::Function>
    where
        Self: super::Engine,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct ThreadTestGameEngine {
    pub id_auto_increment: u64,
}

impl ExternIO for ThreadTestGameEngine {
    fn stdout_print(&mut self, content: String) {}
    fn stdout_println(&mut self, content: String) {}

    fn stderr_print(&mut self, content: String) {}

    fn stdin_scan(&mut self) -> Option<String> {
        None
    }
    fn stdin_request(&mut self) {}

    fn stdasm_print(&mut self, content: String) {
        println!("{}", content);
    }
}

impl Engine for ThreadTestGameEngine {
    type Function = DefaultExternFunction;
    type FunctionContext = DefaultExecutionContext;
}

impl ExternEnergyDispenser for ThreadTestGameEngine {
    fn get_energy(&self) -> usize {
        unimplemented!()
    }

    fn consume_energy(&mut self, energy: usize) -> Result<(), crate::vm::runtime::RuntimeError> {
        unimplemented!()
    }
}

impl ExternThreadHandler for ThreadTestGameEngine {
    type TID = DefaultThreadID;

    fn spawn(&mut self) -> Result<Self::TID, crate::vm::runtime::RuntimeError> {
        self.id_auto_increment += 1;
        Ok(DefaultThreadID(self.id_auto_increment))
    }

    fn close(&mut self, tid: &Self::TID) -> Result<(), crate::vm::runtime::RuntimeError> {
        Ok(())
    }
}

impl ExternPathFinder for ThreadTestGameEngine {
    fn find(path: &[String], name: &str) -> Option<<Self as Engine>::Function>
    where
        Self: super::Engine,
    {
        unimplemented!()
    }
}

// #[derive(Debug, Clone)]
// pub struct TestDynamicGameEngine {
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
//         context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::TID>,
//     ) -> Result<(), RuntimeError> {
//         stdio.stdout.push("\"Hello World from Dynamic function\"");
//         stdio.stdout.flush(engine);
//         scheduler.next();
//         Ok(())
//     }
// }

// impl ExternIO for TestDynamicGameEngine {
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
