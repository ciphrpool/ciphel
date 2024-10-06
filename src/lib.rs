#![allow(unused_variables)]

use ast::{
    modules::{parse_module},
    statements::parse_statements,
};
use semantic::{Resolve, SemanticError};

pub mod ast;
pub mod semantic;
pub mod vm;

pub type CiphelResult<T> = Option<T>;

use thiserror::Error;
use vm::{
    allocator::heap::Heap,
    external::ExternThreadIdentifier,
    runtime::{Runtime, RuntimeError},
    scheduler::SchedulingPolicy,
    stdio::StdIO,
    CodeGenerationError, GenerateCode,
};

#[derive(Debug, Clone, Error)]
pub enum CompilationError<TID: ExternThreadIdentifier> {
    #[error("Parsing Error :\n{0}")]
    ParsingError(String),

    #[error("Compilation Error at line {0}:\n{1}")]
    SemanticError(usize, SemanticError),

    #[error("Compilation Error at line {0}:\n{1}")]
    CodeGen(usize, CodeGenerationError),

    #[error("Invalid thread id : {0}")]
    InvalidTID(TID),
    #[error("Transaction Error : {0}")]
    TransactionError(&'static str),
}

pub struct Ciphel<E: crate::vm::external::Engine, P: SchedulingPolicy> {
    pub runtime: Runtime<E, P>,
    pub heap: Heap,
    pub stdio: StdIO,
}

impl<E: crate::vm::external::Engine, P: SchedulingPolicy> Default for Ciphel<E, P> {
    fn default() -> Self {
        Self {
            runtime: Runtime::default(),
            heap: Heap::new(),
            stdio: StdIO::default(),
        }
    }
}

impl<E: crate::vm::external::Engine, P: SchedulingPolicy> Ciphel<E, P> {
    pub fn import(
        &mut self,
        tids: &[E::TID],
        module: &str,
        line_offset: usize,
    ) -> Result<(), CompilationError<E::TID>> {
        let mut module = parse_module(module.into(), 0)?;
        for tid in tids {
            let tid = tid.clone();
            let vm::runtime::ThreadContext {
                scope_manager,
                program,
                ..
            } = self
                .runtime
                .context_of(&tid)
                .map_err(|_| CompilationError::InvalidTID::<E::TID>(tid.clone()))?;

            module
                .resolve::<E>(scope_manager, None, &(), &mut ())
                .map_err(|err| CompilationError::SemanticError::<E::TID>(0, err))?;
            module
                .gencode::<E>(
                    scope_manager,
                    None,
                    program,
                    &crate::vm::CodeGenerationContext::default(),
                )
                .map_err(|err| CompilationError::CodeGen::<E::TID>(0, err))?;
            scope_manager.modules.push(module.clone());
        }
        Ok(())
    }

    pub fn compile(
        &mut self,
        tid: E::TID,
        src_code: &str,
        line_offset: usize,
    ) -> Result<(), CompilationError<E::TID>> {
        let mut statements = parse_statements(src_code.into(), line_offset)?;
        let vm::runtime::ThreadContext {
            scope_manager,
            program,
            ..
        } = self
            .runtime
            .context_of(&tid)
            .map_err(|_| CompilationError::InvalidTID::<E::TID>(tid))?;

        for statement in statements.iter_mut() {
            statement
                .resolve::<E>(scope_manager, None, &None, &mut ())
                .map_err(|err| CompilationError::SemanticError::<E::TID>(statement.line, err))?;
        }

        for statement in statements {
            statement
                .gencode::<E>(
                    scope_manager,
                    None,
                    program,
                    &crate::vm::CodeGenerationContext::default(),
                )
                .map_err(|err| CompilationError::CodeGen::<E::TID>(statement.line, err))?;
        }
        Ok(())
    }

    pub fn run(&mut self, engine: &mut E) -> Result<(), RuntimeError> {
        let _ = self.runtime.run(&mut self.heap, &mut self.stdio, engine)?;
        Ok(())
    }
}

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
        .find_var_by_name(variable_name, None, None)
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
        .find_var_by_name(variable_name, None, None)
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
    let mut statements =
        parse_statements::<E::TID>(input.into(), 0).expect("Parsing should have succeeded");

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
//     use vm::vm::StdoutTestEngine;

//     use self::vm::vm::{DbgEngine, NoopEngine};

//     use super::*;

//     #[test]
//     fn valid_hello_world() {
//         let mut engine = DbgEngine {};
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
//             .compile::<DbgEngine>(crate::vm::vm::Player::P1, tid, src)
//             .expect("Compilation should have succeeded");

//         ciphel.run(&mut engine).expect("no error should arise");
//     }

//     #[test]
//     fn valid_multiple_program() {
//         let mut engine = NoopEngine {};
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
//             .compile::<NoopEngine>(crate::vm::vm::Player::P1, tid, src)
//             .expect("Compilation should have succeeded");

//         let src = r##"

//         let res = add(10,10);

//         print(f"result = {res}");

//         "##;

//         ciphel
//             .compile::<NoopEngine>(crate::vm::vm::Player::P1, tid, src)
//             .expect("Compilation should have succeeded");

//         ciphel.run(&mut engine).expect("no error should arise");
//         ciphel.run(&mut engine).expect("no error should arise");
//     }

//     #[test]
//     fn valid_multiple_thread() {
//         let mut engine = NoopEngine {};
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
//             .compile::<NoopEngine>(crate::vm::vm::Player::P1, main_tid, src)
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
//             .compile::<NoopEngine>(crate::vm::vm::Player::P1, tid, src)
//             .expect("Compilation should have succeeded");

//         ciphel.run(&mut engine).expect("no error should arise");
//     }

//     #[test]
//     fn valid_commit_transaction() {
//         let mut engine = StdoutTestEngine { out: String::new() };
//         let mut ciphel = Ciphel::new();
//         let tid = ciphel
//             .start_arena(&mut engine)
//             .expect("starting should not fail");

//         let src = r##"
//         println("Hello world");
//         "##;

//         ciphel
//             .compile_with_transaction::<StdoutTestEngine>(
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
//         let mut engine = StdoutTestEngine { out: String::new() };
//         let mut ciphel = Ciphel::new();
//         let tid = ciphel
//             .start_arena(&mut engine)
//             .expect("starting should not fail");

//         let src = r##"
//         println("Hello world");
//         "##;

//         ciphel
//             .compile_with_transaction::<StdoutTestEngine>(
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
//             .compile_with_transaction::<StdoutTestEngine>(
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
