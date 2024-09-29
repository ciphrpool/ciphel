#![allow(unused)]
use ast::statements::parse_statements;
use semantic::{Resolve, SemanticError};

pub mod ast;
pub mod semantic;
pub mod vm;

pub type CiphelResult<T> = Option<T>;

use thiserror::Error;
use vm::{
    allocator::heap::Heap, runtime::Runtime, stdio::StdIO, CodeGenerationError, GenerateCode,
};

#[derive(Debug, Clone, Error)]
pub enum CompilationError {
    #[error("Parsing Error :\n{0}")]
    ParsingError(String),

    #[error("Compilation Error at line {0}:\n{1}")]
    SemanticError(usize, SemanticError),

    #[error("Compilation Error at line {0}:\n{1}")]
    CodeGen(usize, CodeGenerationError),

    #[error("Invalid thread id : {0}")]
    InvalidTID(usize),
    #[error("Transaction Error : {0}")]
    TransactionError(&'static str),
}

// #[derive(Debug, Clone)]
// pub struct Ciphel {
//     runtime: Runtime,
//     heap: Heap,
//     stdio: StdIO,
//     scheduler: Scheduler,
// }

// impl Ciphel {
//     pub fn new() -> Self {
//         let (runtime, heap, stdio) = Runtime::new();
//         Self {
//             runtime,
//             heap,
//             stdio,
//             scheduler: Scheduler::new(),
//         }
//     }

//     pub fn start_arena<E: crate::vm::external::Engine>(
//         &mut self,
//         engine: &mut E,
//     ) -> Result<usize, RuntimeError> {
//         let main_tid = self.runtime.spawn(vm::vm::Player::P1, engine)?;
//         Ok(main_tid)
//     }

//     pub fn available_tids(&self, player: crate::vm::vm::Player) -> Vec<usize> {
//         match player {
//             vm::vm::Player::P1 => self
//                 .runtime
//                 .p1_manager
//                 .alive()
//                 .iter()
//                 .enumerate()
//                 .filter(|(_, alive)| !**alive)
//                 .map(|(i, _)| i)
//                 .collect(),
//             vm::vm::Player::P2 => self
//                 .runtime
//                 .p2_manager
//                 .alive()
//                 .iter()
//                 .enumerate()
//                 .filter(|(_, alive)| !**alive)
//                 .map(|(i, _)| i)
//                 .collect(),
//         }
//     }

//     pub fn compile_with_transaction<E: crate::vm::external::Engine>(
//         &mut self,
//         player: crate::vm::vm::Player,
//         tid: usize,
//         src_code: &str,
//         line_offset: usize,
//     ) -> Result<(), CompilationError> {
//         let (scope_manager, stack, program) = self
//             .runtime
//             .get_mut(player, tid)
//             .map_err(|_| CompilationError::InvalidTID(tid))?;
//         {
//             let _ = scope_manager.open_transaction()?;
//         }
//         program.in_transaction = TransactionState::OPEN;

//         let mut statements = parse_statements(src_code.into(), line_offset)?;
//         for statement in &mut statements {
//             let _ = statement
//                 .resolve::<E>(scope_manager, None, &None, &mut ())
//                 .map_err(|e| CompilationError::SemanticError(statement.line, e))?;
//         }
//         program.statements_buffer.extend(statements);
//         Ok(())
//     }

//     pub fn commit_transaction(
//         &mut self,
//         player: crate::vm::vm::Player,
//         tid: usize,
//     ) -> Result<(), CompilationError> {
//         let (scope_manager, stack, program) = self
//             .runtime
//             .get_mut(player, tid)
//             .map_err(|_| CompilationError::InvalidTID(tid))?;

//         if program.in_transaction == TransactionState::CLOSE {
//             return Err(CompilationError::TransactionError(
//                 "Cannot commit in a closed transaction",
//             ));
//         }
//         {
//             let _ = scope_manager.commit_transaction()?;
//         }

//         program.in_transaction = TransactionState::COMMITED;
//         Ok(())
//     }

//     pub fn reject_transaction(
//         &mut self,
//         player: crate::vm::vm::Player,
//         tid: usize,
//     ) -> Result<(), CompilationError> {
//         let (scope_manager, stack, program) = self
//             .runtime
//             .get_mut(player, tid)
//             .map_err(|_| CompilationError::InvalidTID(tid))?;

//         if program.in_transaction == TransactionState::CLOSE {
//             return Err(CompilationError::TransactionError(
//                 "Cannot reject in closed transaction",
//             ));
//         }
//         {
//             let _ = scope_manager.reject_transaction()?;
//             // let _ = scope_manager.close_transaction()?;
//         }
//         program.statements_buffer.clear();
//         program.in_transaction = TransactionState::CLOSE;
//         Ok(())
//     }

//     pub fn push_compilation(
//         &mut self,
//         player: crate::vm::vm::Player,
//         tid: usize,
//     ) -> Result<(), CompilationError> {
//         let (scope_manager, stack, program) = self
//             .runtime
//             .get_mut(player, tid)
//             .map_err(|_| CompilationError::InvalidTID(tid))?;

//         if program.in_transaction != TransactionState::COMMITED {
//             return Err(CompilationError::TransactionError(
//                 "Cannot push without a commit",
//             ));
//         }

//         let statements: Vec<WithLine<Statement>> = program.statements_buffer.drain(..).collect();

//         for statement in statements {
//             let _ = statement
//                 .gencode::<E>(
//                     scope_manager,
//                     None,
//                     program,
//                     &crate::vm::CodeGenerationContext::default(),
//                 )
//                 .map_err(|e| CompilationError::CodeGen(statement.line, e))?;
//         }

//         Ok(())
//     }

//     pub fn compile<E: crate::vm::external::Engine>(
//         &mut self,
//         player: crate::vm::vm::Player,
//         tid: usize,
//         src_code: &str,
//     ) -> Result<(), CompilationError> {
//         let (scope_manager, stack, program) = self
//             .runtime
//             .get_mut(player, tid)
//             .map_err(|_| CompilationError::InvalidTID(tid))?;

//         let mut statements = parse_statements(src_code.into(), 0)?;

//         for statement in &mut statements {
//             let _ = statement
//                 .resolve::<E>(scope_manager, None, &None, &mut ())
//                 .map_err(|e| CompilationError::SemanticError(statement.line, e))?;
//         }

//         for statement in &statements {
//             let _ = statement
//                 .gencode::<E>(
//                     scope_manager,
//                     None,
//                     program,
//                     &crate::vm::CodeGenerationContext::default(),
//                 )
//                 .map_err(|e| CompilationError::CodeGen(statement.line, e))?;
//         }

//         Ok(())
//     }

//     pub fn run<E: crate::vm::external::Engine>(
//         &mut self,
//         engine: &mut E,
//     ) -> Result<(), RuntimeError> {
//         self.scheduler.prepare(&mut self.runtime);

//         self.scheduler.run_major_frame(
//             &mut self.runtime,
//             &mut self.heap,
//             &mut self.stdio,
//             engine,
//         )?;
//         Ok(())
//     }
// }

pub fn test_extract_variable_with(
    variable_name: &str,
    callback: impl Fn(
        vm::allocator::MemoryAddress,
        &crate::vm::allocator::stack::Stack,
        &crate::vm::allocator::heap::Heap,
    ),
    scope_manager: &crate::semantic::scope::scope::ScopeManager,
    stack: &crate::vm::allocator::stack::Stack,
    heap: &crate::vm::allocator::heap::Heap,
) {
    let crate::semantic::scope::scope::Variable { id, .. } = scope_manager
        .find_var_by_name(variable_name, None)
        .expect("The variable should have been found");
    let crate::semantic::scope::scope::VariableInfo { address, .. } = scope_manager
        .find_var_by_id(id)
        .expect("The variable should have been found");

    let address: vm::allocator::MemoryAddress = (*address)
        .try_into()
        .expect("the address should have been known");

    callback(address, stack, heap);
}

pub fn test_extract_variable<N: num_traits::PrimInt>(
    variable_name: &str,
    scope_manager: &crate::semantic::scope::scope::ScopeManager,
    stack: &crate::vm::allocator::stack::Stack,
    heap: &crate::vm::allocator::heap::Heap,
) -> Option<N> {
    let crate::semantic::scope::scope::Variable { id, .. } = scope_manager
        .find_var_by_name(variable_name, None)
        .expect("The variable should have been found");

    let crate::semantic::scope::scope::VariableInfo { address, .. } = scope_manager
        .find_var_by_id(id)
        .expect("The variable should have been found");

    let address: vm::allocator::MemoryAddress = (*address)
        .try_into()
        .expect("the address should have been known");
    let res =
        <vm::asm::operation::OpPrimitive as vm::asm::operation::GetNumFrom>::get_num_from::<N>(
            address, stack, heap,
        )
        .expect("Deserialization should have succeeded");

    return Some(res);
}

pub fn test_statements<E: crate::vm::external::Engine>(
    input: &str,
    engine: &mut E,
    assert_fn: fn(
        &crate::semantic::scope::scope::ScopeManager,
        &vm::allocator::stack::Stack,
        &vm::allocator::heap::Heap,
    ) -> bool,
) {
    let mut statements = parse_statements(input.into(), 0).expect("Parsing should have succeeded");

    let mut heap = Heap::new();
    let mut stdio = StdIO::default();
    let mut runtime: Runtime<E, vm::scheduler::ToCompletion> = Runtime::default();

    let tid = runtime
        .spawn(engine)
        .expect("Spawning should have succeeded");
    {
        let vm::runtime::ThreadContext {
            scope_manager,
            program,
            ..
        } = runtime
            .context_of(&tid)
            .expect("Thread should have been found");

        for statement in statements.iter_mut() {
            statement
                .resolve::<E>(scope_manager, None, &None, &mut ())
                .expect(&format!("Resulotion should have succeeded {:?}", statement));
        }

        for statement in statements {
            statement
                .gencode::<E>(
                    scope_manager,
                    None,
                    program,
                    &crate::vm::CodeGenerationContext::default(),
                )
                .expect(&format!(
                    "Code generation should have succeeded {:?}",
                    statement
                ));
        }
    }

    let _ = runtime
        .run(&mut heap, &mut stdio, engine)
        .expect("Execution should have succeeded");

    let (vm::runtime::Thread { stack, .. }, vm::runtime::ThreadContext { scope_manager, .. }) =
        runtime
            .thread_with_context_of(&tid)
            .expect("Thread should have been found");

    assert!(assert_fn(scope_manager, stack, &mut heap));
}

// #[cfg(test)]
// mod tests {
//     use vm::vm::StdoutTestGameEngine;

//     use self::vm::vm::{DbgGameEngine, NoopGameEngine};

//     use super::*;

//     #[test]
//     fn valid_hello_world() {
//         let mut engine = DbgGameEngine {};
//         let mut ciphel = Ciphel::new();
//         let tid = ciphel
//             .start_arena(&mut engine)
//             .expect("starting should not fail");

//         let src = r##"

//         fn main() -> Unit {
//             println("Hello World");
//         }

//         main();

//         "##;

//         ciphel
//             .compile::<DbgGameEngine>(crate::vm::vm::Player::P1, tid, src)
//             .expect("Compilation should have succeeded");

//         ciphel.run(&mut engine).expect("no error should arise");
//     }

//     #[test]
//     fn valid_multiple_program() {
//         let mut engine = NoopGameEngine {};
//         let mut ciphel = Ciphel::new();
//         let tid = ciphel
//             .start_arena(&mut engine)
//             .expect("starting should not fail");

//         let src = r##"

//         fn main() -> Unit {
//             print("Hello World");
//         }

//         fn add(x:u64,y:u64) -> u64 {
//             return x+y;
//         }

//         main();

//         "##;

//         ciphel
//             .compile::<NoopGameEngine>(crate::vm::vm::Player::P1, tid, src)
//             .expect("Compilation should have succeeded");

//         let src = r##"

//         let res = add(10,10);

//         print(f"result = {res}");

//         "##;

//         ciphel
//             .compile::<NoopGameEngine>(crate::vm::vm::Player::P1, tid, src)
//             .expect("Compilation should have succeeded");

//         ciphel.run(&mut engine).expect("no error should arise");
//         ciphel.run(&mut engine).expect("no error should arise");
//     }

//     #[test]
//     fn valid_multiple_thread() {
//         let mut engine = NoopGameEngine {};
//         let mut ciphel = Ciphel::new();
//         let main_tid = ciphel
//             .start_arena(&mut engine)
//             .expect("starting should not fail");

//         let src = r##"

//         fn main() -> Unit {
//             print("Hello World\n");
//         }
//         main();

//         "##;

//         ciphel
//             .compile::<NoopGameEngine>(crate::vm::vm::Player::P1, main_tid, src)
//             .expect("Compilation should have succeeded");

//         let tid = ciphel
//             .runtime
//             .spawn(crate::vm::vm::Player::P1, &mut engine)
//             .expect("spawning should have succeeded");

//         let src = r##"
//         fn add(x:u64,y:u64) -> u64 {
//             return x+y;
//         }
//         let res = add(10,10);

//         print(f"result = {res}");

//         "##;

//         ciphel
//             .compile::<NoopGameEngine>(crate::vm::vm::Player::P1, tid, src)
//             .expect("Compilation should have succeeded");

//         ciphel.run(&mut engine).expect("no error should arise");
//     }

//     #[test]
//     fn valid_commit_transaction() {
//         let mut engine = StdoutTestGameEngine { out: String::new() };
//         let mut ciphel = Ciphel::new();
//         let tid = ciphel
//             .start_arena(&mut engine)
//             .expect("starting should not fail");

//         let src = r##"
//         println("Hello world");
//         "##;

//         ciphel
//             .compile_with_transaction::<StdoutTestGameEngine>(
//                 crate::vm::vm::Player::P1,
//                 tid,
//                 src,
//                 0,
//             )
//             .expect("Compilation should have succeeded");

//         ciphel
//             .commit_transaction(crate::vm::vm::Player::P1, tid)
//             .expect("Compilation should have succeeded");
//         ciphel
//             .push_compilation(crate::vm::vm::Player::P1, tid)
//             .expect("Compilation should have succeeded");
//         ciphel.run(&mut engine).expect("no error should arise");

//         let output = engine.out;
//         assert_eq!(output, "Hello world\n");
//     }
//     #[test]
//     fn valid_reject_transaction() {
//         let mut engine = StdoutTestGameEngine { out: String::new() };
//         let mut ciphel = Ciphel::new();
//         let tid = ciphel
//             .start_arena(&mut engine)
//             .expect("starting should not fail");

//         let src = r##"
//         println("Hello world");
//         "##;

//         ciphel
//             .compile_with_transaction::<StdoutTestGameEngine>(
//                 crate::vm::vm::Player::P1,
//                 tid,
//                 src,
//                 0,
//             )
//             .expect("Compilation should have succeeded");

//         ciphel
//             .reject_transaction(crate::vm::vm::Player::P1, tid)
//             .expect("Compilation should have succeeded");

//         let src = r##"
//         println("Hi");
//         "##;

//         ciphel
//             .compile_with_transaction::<StdoutTestGameEngine>(
//                 crate::vm::vm::Player::P1,
//                 tid,
//                 src,
//                 0,
//             )
//             .expect("Compilation should have succeeded");

//         ciphel
//             .commit_transaction(crate::vm::vm::Player::P1, tid)
//             .expect("Compilation should have succeeded");
//         ciphel
//             .push_compilation(crate::vm::vm::Player::P1, tid)
//             .expect("Compilation should have succeeded");
//         ciphel.run(&mut engine).expect("no error should arise");

//         let output = engine.out;
//         assert_eq!(output, "Hi\n");
//     }
// }
