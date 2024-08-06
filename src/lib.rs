use ast::statements::{self, Statement};
use semantic::SemanticError;
use vm::{
    allocator::heap::Heap,
    casm::TransactionState,
    scheduler::Scheduler,
    stdio::StdIO,
    vm::{CodeGenerationError, GameEngineStaticFn, Runtime, RuntimeError},
};

use crate::{ast::statements::parse_statements, semantic::Resolve, vm::vm::GenerateCode};

pub mod ast;
pub mod semantic;
pub mod vm;

pub type CiphelResult<T> = Option<T>;

use thiserror::Error;

#[derive(Debug, Clone,Error)]
pub enum CompilationError {
    #[error("Parsing Error")]
    ParsingError(),

    #[error("Semantic Error : {0}")]
    SemanticError(#[from] SemanticError),

    #[error("Code Generation Error : {0}")]
    CodeGen(#[from] CodeGenerationError),

    #[error("Invalid thread id : {0}")]
    InvalidTID(usize),
    #[error("Transaction Error")]
    TransactionError,
}

#[derive(Debug, Clone)]
pub struct Ciphel {
    runtime: Runtime,
    heap: Heap,
    stdio: StdIO,
    scheduler: Scheduler,
}

impl Ciphel {
    pub fn new() -> Self {
        let (runtime, heap, stdio) = Runtime::new();
        Self {
            runtime,
            heap,
            stdio,
            scheduler: Scheduler::new(),
        }
    }

    pub fn start_arena<G: crate::GameEngineStaticFn>(
        &mut self,
        engine: &mut G,
    ) -> Result<usize, RuntimeError> {
        let main_tid = self.runtime.spawn(vm::vm::Player::P1, engine)?;
        Ok(main_tid)
    }

    pub fn available_tids(&self, player: crate::vm::vm::Player) -> Vec<usize> {
        match player {
            vm::vm::Player::P1 => self
                .runtime
                .p1_manager
                .alive()
                .iter()
                .enumerate()
                .filter(|(_, alive)| !**alive)
                .map(|(i, _)| i)
                .collect(),
            vm::vm::Player::P2 => self
                .runtime
                .p2_manager
                .alive()
                .iter()
                .enumerate()
                .filter(|(_, alive)| !**alive)
                .map(|(i, _)| i)
                .collect(),
        }
    }

    pub fn compile_with_transaction(
        &mut self,
        player: crate::vm::vm::Player,
        tid: usize,
        src_code: &str,
    ) -> Result<(), CompilationError> {
        let (scope, stack, program) = self
            .runtime
            .get_mut(player, tid)
            .map_err(|_| CompilationError::InvalidTID(tid))?;
        {
            let mut borrowed_scope = scope
                .write()
                .map_err(|_| CompilationError::SemanticError(SemanticError::ConcurrencyError))?;
            let _ = borrowed_scope.open_transaction()?;
        }

        let mut statements: Vec<ast::statements::Statement> = parse_statements(src_code.into())?;
        for statement in &mut statements {
            let _ = statement
                .resolve(scope, &None, &mut ())
                .map_err(|e| CompilationError::SemanticError(e))?;
        }
        program.statements_buffer.extend(statements);
        program.in_transaction = TransactionState::OPEN;
        Ok(())
    }

    pub fn commit_transaction(
        &mut self,
        player: crate::vm::vm::Player,
        tid: usize,
    ) -> Result<(), CompilationError> {
        let (scope, stack, program) = self
            .runtime
            .get_mut(player, tid)
            .map_err(|_| CompilationError::InvalidTID(tid))?;
        dbg!(program.in_transaction);
        if program.in_transaction == TransactionState::CLOSE {
            return Err(CompilationError::TransactionError);
        }
        {
            let mut borrowed_scope = scope
                .write()
                .map_err(|_| CompilationError::SemanticError(SemanticError::ConcurrencyError))?;
            let _ = borrowed_scope.commit_transaction()?;
            let _ = borrowed_scope.close_transaction()?;
        }

        program.in_transaction = TransactionState::COMMITED;
        Ok(())
    }

    pub fn reject_transaction(
        &mut self,
        player: crate::vm::vm::Player,
        tid: usize,
    ) -> Result<(), CompilationError> {
        let (scope, stack, program) = self
            .runtime
            .get_mut(player, tid)
            .map_err(|_| CompilationError::InvalidTID(tid))?;

        if program.in_transaction == TransactionState::CLOSE {
            return Err(CompilationError::TransactionError);
        }
        {
            let mut borrowed_scope = scope
                .write()
                .map_err(|_| CompilationError::SemanticError(SemanticError::ConcurrencyError))?;
            let _ = borrowed_scope.reject_transaction()?;
            let _ = borrowed_scope.close_transaction()?;
        }
        program.statements_buffer.clear();
        program.in_transaction = TransactionState::CLOSE;
        Ok(())
    }

    pub fn push_compilation(
        &mut self,
        player: crate::vm::vm::Player,
        tid: usize,
    ) -> Result<(), CompilationError> {
        let (scope, stack, program) = self
            .runtime
            .get_mut(player, tid)
            .map_err(|_| CompilationError::InvalidTID(tid))?;

        if program.in_transaction != TransactionState::COMMITED {
            return Err(CompilationError::TransactionError);
        }
        let _ = arw_read!(
            scope,
            CompilationError::SemanticError(SemanticError::ConcurrencyError)
        )?
        .update_stack_top(stack.top())
        .map_err(|e| CompilationError::CodeGen(e))?;

        let statements: Vec<Statement> = program.statements_buffer.drain(..).collect();

        for statement in statements {
            let _ = statement
                .gencode(scope, program)
                .map_err(|e| CompilationError::CodeGen(e))?;
        }

        Ok(())
    }

    pub fn compile(
        &mut self,
        player: crate::vm::vm::Player,
        tid: usize,
        src_code: &str,
    ) -> Result<(), CompilationError> {
        let (scope, stack, program) = self
            .runtime
            .get_mut(player, tid)
            .map_err(|_| CompilationError::InvalidTID(tid))?;

        let mut statements = parse_statements(src_code.into())?;

        for statement in &mut statements {
            let _ = statement
                .resolve(scope, &None, &mut ())
                .map_err(|e| CompilationError::SemanticError(e))?;
        }

        let _ = arw_read!(
            scope,
            CompilationError::SemanticError(SemanticError::ConcurrencyError)
        )?
        .update_stack_top(stack.top())
        .map_err(|e| CompilationError::CodeGen(e))?;

        for statement in &statements {
            let _ = statement
                .gencode(scope, program)
                .map_err(|e| CompilationError::CodeGen(e))?;
        }

        Ok(())
    }

    pub fn run<G: crate::GameEngineStaticFn>(
        &mut self,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        self.scheduler.prepare(&mut self.runtime);

        self.scheduler.run_major_frame(
            &mut self.runtime,
            &mut self.heap,
            &mut self.stdio,
            engine,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use vm::vm::{StdinTestGameEngine, StdoutTestGameEngine};

    use self::vm::vm::{DbgGameEngine, NoopGameEngine};

    use super::*;

    #[test]
    fn valid_hello_world() {
        let mut engine = DbgGameEngine {};
        let mut ciphel = Ciphel::new();
        let tid = ciphel
            .start_arena(&mut engine)
            .expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            println("Hello World");
        }

        main();
        
        "##;

        ciphel
            .compile(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");

        ciphel.run(&mut engine).expect("no error should arise");
    }

    #[test]
    fn valid_multiple_program() {
        let mut engine = NoopGameEngine {};
        let mut ciphel = Ciphel::new();
        let tid = ciphel
            .start_arena(&mut engine)
            .expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }

        fn add(x:u64,y:u64) -> u64 {
            return x+y;
        }

        main();
        
        "##;

        ciphel
            .compile(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");

        let src = r##"
        
        let res = add(10,10);
        
        print(f"result = {res}");
        
        "##;

        ciphel
            .compile(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");

        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
    }

    #[test]
    fn valid_multiple_thread() {
        let mut engine = NoopGameEngine {};
        let mut ciphel = Ciphel::new();
        let main_tid = ciphel
            .start_arena(&mut engine)
            .expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World\n");
        }
        main();
        
        "##;

        ciphel
            .compile(crate::vm::vm::Player::P1, main_tid, src)
            .expect("Compilation should have succeeded");

        let tid = ciphel
            .runtime
            .spawn(crate::vm::vm::Player::P1, &mut engine)
            .expect("spawning should have succeeded");

        let src = r##"
        fn add(x:u64,y:u64) -> u64 {
            return x+y;
        }
        let res = add(10,10);

        print(f"result = {res}");
        
        "##;

        ciphel
            .compile(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");

        ciphel.run(&mut engine).expect("no error should arise");
    }

    #[test]
    fn valid_commit_transaction() {
        let mut engine = StdoutTestGameEngine { out: String::new() };
        let mut ciphel = Ciphel::new();
        let tid = ciphel
            .start_arena(&mut engine)
            .expect("starting should not fail");

        let src = r##"
        println("Hello world");
        "##;

        ciphel
            .compile_with_transaction(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");

        ciphel
            .commit_transaction(crate::vm::vm::Player::P1, tid)
            .expect("Compilation should have succeeded");
        ciphel
            .push_compilation(crate::vm::vm::Player::P1, tid)
            .expect("Compilation should have succeeded");
        ciphel.run(&mut engine).expect("no error should arise");

        let output = engine.out;
        assert_eq!(output, "Hello world\n");
    }
    #[test]
    fn valid_reject_transaction() {
        let mut engine = StdoutTestGameEngine { out: String::new() };
        let mut ciphel = Ciphel::new();
        let tid = ciphel
            .start_arena(&mut engine)
            .expect("starting should not fail");

        let src = r##"
        println("Hello world");
        "##;

        ciphel
            .compile_with_transaction(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");

        ciphel
            .reject_transaction(crate::vm::vm::Player::P1, tid)
            .expect("Compilation should have succeeded");

        let src = r##"
        println("Hi");
        "##;

        ciphel
            .compile_with_transaction(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");

        ciphel
            .commit_transaction(crate::vm::vm::Player::P1, tid)
            .expect("Compilation should have succeeded");
        ciphel
            .push_compilation(crate::vm::vm::Player::P1, tid)
            .expect("Compilation should have succeeded");
        ciphel.run(&mut engine).expect("no error should arise");

        let output = engine.out;
        assert_eq!(output, "Hi\n");
    }
}
