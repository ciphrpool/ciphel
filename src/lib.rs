use semantic::SemanticError;
use vm::{
    allocator::heap::Heap,
    scheduler::Scheduler,
    stdio::StdIO,
    vm::{CodeGenerationError, GameEngineStaticFn, Runtime, RuntimeError},
};

use crate::{ast::statements::parse_statements, semantic::Resolve, vm::vm::GenerateCode};

pub mod ast;
pub mod semantic;
pub mod vm;

#[derive(Debug)]
pub enum CompilationError {
    ParsingError(),
    SemanticError(SemanticError),
    CodeGen(CodeGenerationError),
    InvalidTID(usize),
}

pub struct Ciphel<G: GameEngineStaticFn + Clone> {
    runtime: Runtime<G>,
    heap: Heap,
    stdio: StdIO<G>,
    scheduler: Scheduler<G>,
}

impl<G: crate::GameEngineStaticFn + Clone> Ciphel<G> {
    pub fn new() -> Self {
        let (runtime, heap, stdio) = Runtime::new();
        Self {
            runtime,
            heap,
            stdio,
            scheduler: Scheduler::new(),
        }
    }

    pub fn start(&mut self) -> Result<usize, RuntimeError> {
        let main_tid = self.runtime.spawn()?;
        Ok(main_tid)
    }

    pub fn available_tids(&self) -> Vec<usize> {
        self.runtime.threads.iter().map(|t| t.tid).collect()
    }

    pub fn compile(&mut self, tid: usize, src_code: &str) -> Result<(), CompilationError> {
        let (scope, stack, program) = self
            .runtime
            .get_mut(tid)
            .map_err(|_| CompilationError::InvalidTID(tid))?;

        let statements = parse_statements(src_code.into())?;

        for statement in &statements {
            let _ = statement
                .resolve(scope, &None, &())
                .map_err(|e| CompilationError::SemanticError(e))?;
        }

        let _ = scope
            .borrow()
            .update_stack_top(stack.top())
            .map_err(|e| CompilationError::CodeGen(e))?;

        for statement in &statements {
            let _ = statement
                .gencode(scope, program)
                .map_err(|e| CompilationError::CodeGen(e))?;
        }

        Ok(())
    }

    pub fn run(&mut self, engine: &mut G) -> Result<(), RuntimeError> {
        self.scheduler.prepare(self.runtime.threads.len());

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
    use self::vm::vm::{DbgGameEngine, NoopGameEngine};

    use super::*;

    #[test]
    fn valid_hello_world() {
        let mut engine = DbgGameEngine {};
        let mut ciphel = Ciphel::new();
        let tid = ciphel.start().expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            println("Hello World");
        }

        main();
        
        "##;

        ciphel
            .compile(tid, src)
            .expect("Compilation should have succeeded");

        ciphel.run(&mut engine).expect("no error should arise");
    }

    #[test]
    fn valid_multiple_program() {
        let mut engine = NoopGameEngine {};
        let mut ciphel = Ciphel::<NoopGameEngine>::new();
        let tid = ciphel.start().expect("starting should not fail");

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
            .compile(tid, src)
            .expect("Compilation should have succeeded");

        let src = r##"
        
        let res = add(10,10);
        
        print(f"result = {res}");
        
        "##;

        ciphel
            .compile(tid, src)
            .expect("Compilation should have succeeded");

        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
    }

    #[test]
    fn valid_multiple_thread() {
        let mut engine = NoopGameEngine {};
        let mut ciphel = Ciphel::<NoopGameEngine>::new();
        let main_tid = ciphel.start().expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World\n");
        }
        main();
        
        "##;

        ciphel
            .compile(main_tid, src)
            .expect("Compilation should have succeeded");

        let tid = ciphel
            .runtime
            .spawn()
            .expect("spawning should have succeeded");

        let src = r##"
        fn add(x:u64,y:u64) -> u64 {
            return x+y;
        }
        let res = add(10,10);

        print(f"result = {res}");
        
        "##;

        ciphel
            .compile(tid, src)
            .expect("Compilation should have succeeded");

        ciphel.run(&mut engine).expect("no error should arise");
    }
}
