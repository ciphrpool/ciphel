use crate::ast::utils::strings::ID;
use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::scope::static_types::{NumberType, PrimitiveType, StaticType};
use crate::semantic::{EType, TypeOf};
use crate::vm::allocator::stack::Stack;
use crate::vm::casm::operation::OpPrimitive;
use crate::vm::casm::Casm;
use crate::vm::platform::stdlib::{ERROR_SLICE, ERROR_VALUE, OK_SLICE, OK_VALUE};
use crate::vm::platform::utils::lexem;
use crate::vm::platform::LibCasm;
use crate::vm::scheduler::{SchedulerContext, WaitingStatus};
use crate::vm::vm::{CasmMetadata, Executable, RuntimeError, Signal, ThreadState, Tid};
use crate::{
    ast::expressions::Expression,
    semantic::{Resolve, SemanticError},
    vm::{
        casm::CasmProgram,
        vm::{CodeGenerationError, GenerateCode},
    },
};
use crate::{e_static, err_tuple, p_num};

use super::CoreCasm;
#[derive(Debug, Clone, PartialEq)]
pub enum ThreadFn {
    Spawn,
    Close,
    Exit,
    Wait,
    Wake,
    Sleep,
    Join,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThreadCasm {
    Spawn,
    Close,
    Exit,
    Wait,
    Wake,
    Sleep,
    Join,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for ThreadCasm {
    fn name(&self, stdio: &mut crate::vm::stdio::StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            ThreadCasm::Spawn => stdio.push_casm_lib(engine, "spawn"),
            ThreadCasm::Close => stdio.push_casm_lib(engine, "close"),
            ThreadCasm::Exit => stdio.push_casm_lib(engine, "exit"),
            ThreadCasm::Wait => stdio.push_casm_lib(engine, "wait"),
            ThreadCasm::Wake => stdio.push_casm_lib(engine, "wake"),
            ThreadCasm::Sleep => stdio.push_casm_lib(engine, "sleep"),
            ThreadCasm::Join => stdio.push_casm_lib(engine, "join"),
        }
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            ThreadCasm::Spawn => crate::vm::vm::CasmWeight::MAX,
            ThreadCasm::Close => crate::vm::vm::CasmWeight::HIGH,
            ThreadCasm::Exit => crate::vm::vm::CasmWeight::ZERO,
            ThreadCasm::Wait => crate::vm::vm::CasmWeight::HIGH,
            ThreadCasm::Wake => crate::vm::vm::CasmWeight::HIGH,
            ThreadCasm::Sleep => crate::vm::vm::CasmWeight::HIGH,
            ThreadCasm::Join => crate::vm::vm::CasmWeight::HIGH,
        }
    }
}
impl ThreadFn {
    pub fn from(suffixe: &Option<ID>, id: &ID) -> Option<Self> {
        match suffixe {
            Some(suffixe) => {
                if *suffixe != lexem::CORE {
                    return None;
                }
            }
            None => {}
        }
        match id.as_str() {
            lexem::SPAWN => Some(ThreadFn::Spawn),
            lexem::CLOSE => Some(ThreadFn::Close),
            lexem::EXIT => Some(ThreadFn::Exit),
            lexem::WAIT => Some(ThreadFn::Wait),
            lexem::WAKE => Some(ThreadFn::Wake),
            lexem::SLEEP => Some(ThreadFn::Sleep),
            lexem::JOIN => Some(ThreadFn::Join),
            _ => None,
        }
    }
}
fn expect_one_u64<G: crate::GameEngineStaticFn>(
    params: &mut Vec<Expression>,
    scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
    scope_id: Option<u128>,
) -> Result<(), SemanticError> {
    if params.len() != 1 {
        return Err(SemanticError::IncorrectArguments);
    }

    let size = &mut params[0];

    let _ = size.resolve::<G>(scope_manager, scope_id, &Some(p_num!(U64)), &mut None)?;
    let size_type = size.type_of(&scope_manager, scope_id)?;
    match &size_type {
        EType::Static(value) => match value {
            StaticType::Primitive(PrimitiveType::Number(NumberType::U64)) => {}
            _ => return Err(SemanticError::IncorrectArguments),
        },
        _ => return Err(SemanticError::IncorrectArguments),
    }
    Ok(())
}
impl Resolve for ThreadFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression>;
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        _context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            ThreadFn::Spawn => {
                if extra.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                Ok(())
            }
            ThreadFn::Exit => {
                if extra.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                Ok(())
            }
            ThreadFn::Close => expect_one_u64::<G>(extra, scope_manager, scope_id),
            ThreadFn::Wait => {
                if extra.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                Ok(())
            }
            ThreadFn::Wake => expect_one_u64::<G>(extra, scope_manager, scope_id),
            ThreadFn::Sleep => expect_one_u64::<G>(extra, scope_manager, scope_id),
            ThreadFn::Join => expect_one_u64::<G>(extra, scope_manager, scope_id),
        }
    }
}
impl TypeOf for ThreadFn {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            ThreadFn::Spawn => Ok(err_tuple!(p_num!(U64))),
            ThreadFn::Close => Ok(e_static!(StaticType::Error)),
            ThreadFn::Wait => Ok(e_static!(StaticType::Error)),
            ThreadFn::Wake => Ok(e_static!(StaticType::Error)),
            ThreadFn::Exit => Ok(e_static!(StaticType::Unit)),
            ThreadFn::Sleep => Ok(e_static!(StaticType::Unit)),
            ThreadFn::Join => Ok(e_static!(StaticType::Error)),
        }
    }
}

impl GenerateCode for ThreadFn {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            ThreadFn::Spawn => instructions.push(Casm::Platform(LibCasm::Core(CoreCasm::Thread(
                ThreadCasm::Spawn,
            )))),
            ThreadFn::Exit => instructions.push(Casm::Platform(LibCasm::Core(CoreCasm::Thread(
                ThreadCasm::Exit,
            )))),
            ThreadFn::Close => instructions.push(Casm::Platform(LibCasm::Core(CoreCasm::Thread(
                ThreadCasm::Close,
            )))),
            ThreadFn::Wait => instructions.push(Casm::Platform(LibCasm::Core(CoreCasm::Thread(
                ThreadCasm::Wait,
            )))),
            ThreadFn::Wake => instructions.push(Casm::Platform(LibCasm::Core(CoreCasm::Thread(
                ThreadCasm::Wake,
            )))),
            ThreadFn::Sleep => instructions.push(Casm::Platform(LibCasm::Core(CoreCasm::Thread(
                ThreadCasm::Sleep,
            )))),
            ThreadFn::Join => instructions.push(Casm::Platform(LibCasm::Core(CoreCasm::Thread(
                ThreadCasm::Join,
            )))),
        }
        Ok(())
    }
}

pub fn sig_spawn(
    context: &mut SchedulerContext,
    program: &mut CasmProgram,
    stack: &mut Stack,
) -> Result<(), RuntimeError> {
    if let Ok(tid) = context.request_spawn() {
        let _ = stack.push_with(&(tid as u64).to_le_bytes())?;

        let _ = stack.push_with(&OK_SLICE)?;
        program.incr();
        Ok(())
    } else {
        // Error TooManyThread
        let _ = stack.push_with(&(0u64).to_le_bytes())?;

        let _ = stack.push_with(&ERROR_SLICE)?;
        program.incr();
        Err(RuntimeError::TooManyThread)
    }
}

pub fn sig_close(
    tid: Tid,
    context: &mut SchedulerContext,
    program: &mut CasmProgram,
    stack: &mut Stack,
) -> Result<(), RuntimeError> {
    if context.request_close(tid).is_ok() {
        let _ = stack.push_with(&OK_SLICE)?;
        program.incr();
        Ok(())
    } else {
        // Error InvalidTID
        let _ = stack.push_with(&(0u64).to_le_bytes())?;

        let _ = stack.push_with(&ERROR_SLICE)?;
        program.incr();
        Err(RuntimeError::InvalidTID(tid))
    }
}

pub fn sig_wait(state: &mut ThreadState, program: &mut CasmProgram) -> Result<(), RuntimeError> {
    let _ = state.to(ThreadState::WAITING)?;
    program.incr();
    Err(RuntimeError::Signal(Signal::WAIT))
}

pub fn sig_wake(
    tid: Tid,
    context: &mut SchedulerContext,
    program: &mut CasmProgram,
    stack: &mut Stack,
) -> Result<(), RuntimeError> {
    if context.request_wake(tid).is_ok() {
        let _ = stack.push_with(&OK_SLICE)?;
        program.incr();
        Ok(())
    } else {
        let _ = stack.push_with(&ERROR_SLICE)?;
        program.incr();
        Err(RuntimeError::InvalidTID(tid))
    }
}

pub fn sig_sleep(
    nb_maf: &usize,
    state: &mut ThreadState,
    program: &mut CasmProgram,
) -> Result<(), RuntimeError> {
    let _ = state.to(ThreadState::SLEEPING(*nb_maf))?;
    program.incr();
    Err(RuntimeError::Signal(Signal::SLEEP(*nb_maf)))
}

pub fn sig_join(
    own_tid: Tid,
    join_tid: Tid,
    context: &mut SchedulerContext,
    state: &mut ThreadState,
    program: &mut CasmProgram,
    stack: &mut Stack,
    waiting_list: &mut Vec<WaitingStatus>,
) -> Result<(), RuntimeError> {
    if let Ok(true) = context.is_alive(join_tid) {
        let _ = state.to(ThreadState::WAITING)?;
        waiting_list.push(WaitingStatus::Join {
            join_tid: own_tid,
            to_be_completed_tid: join_tid,
        });
        let _ = stack.push_with(&[true as u8])?;

        let _ = stack.push_with(&OK_SLICE)?;
        program.incr();
        Err(RuntimeError::Signal(Signal::JOIN(join_tid)))
    } else {
        let _ = stack.push_with(&ERROR_SLICE)?;
        program.incr();
        Err(RuntimeError::InvalidTID(join_tid))
    }
}
pub fn sig_exit(
    tid: Tid,
    context: &mut SchedulerContext,
    state: &mut ThreadState,
    waiting_list: &mut Vec<WaitingStatus>,
) -> Result<(), RuntimeError> {
    let _ = state.to(ThreadState::EXITED);
    context.request_close(tid)?;
    let idx_tid_list: Vec<(usize, usize)> = waiting_list
        .iter()
        .enumerate()
        .filter(|(_, status)| match status {
            WaitingStatus::Join {
                join_tid,
                to_be_completed_tid,
            } => *to_be_completed_tid == tid,
            WaitingStatus::ForStdin(_) => false,
        })
        .map(|(idx, status)| {
            (
                idx,
                match status {
                    WaitingStatus::Join {
                        join_tid,
                        to_be_completed_tid,
                    } => *join_tid,
                    WaitingStatus::ForStdin(id) => *id, // unreachable,
                },
            )
        })
        .collect();

    for (idx, tid) in idx_tid_list {
        waiting_list.remove(idx);
        context.request_wake(tid)?;
    }

    Err(RuntimeError::Signal(Signal::EXIT))
}

pub fn sig_wait_stdin(
    tid: Tid,
    state: &mut ThreadState,
    waiting_list: &mut Vec<WaitingStatus>,
) -> Result<(), RuntimeError> {
    let _ = state.to(ThreadState::WAITING)?;
    waiting_list.push(WaitingStatus::ForStdin(tid));
    Err(RuntimeError::Signal(Signal::WAIT_STDIN))
}

impl<G: crate::GameEngineStaticFn> Executable<G> for ThreadCasm {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self {
            ThreadCasm::Spawn => Err(RuntimeError::Signal(Signal::SPAWN)),
            ThreadCasm::Exit => Err(RuntimeError::Signal(Signal::EXIT)),
            ThreadCasm::Close => {
                let tid = OpPrimitive::get_num8::<u64>(stack)?;
                Err(RuntimeError::Signal(Signal::CLOSE(tid as usize)))
            }
            ThreadCasm::Wait => Err(RuntimeError::Signal(Signal::WAIT)),
            ThreadCasm::Wake => {
                let tid = OpPrimitive::get_num8::<u64>(stack)?;
                Err(RuntimeError::Signal(Signal::WAKE(tid as usize)))
            }
            ThreadCasm::Sleep => {
                let nb_maf = OpPrimitive::get_num8::<u64>(stack)?;
                Err(RuntimeError::Signal(Signal::SLEEP(nb_maf as usize)))
            }
            ThreadCasm::Join => {
                let tid = OpPrimitive::get_num8::<u64>(stack)?;
                Err(RuntimeError::Signal(Signal::JOIN(tid as usize)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        vm::vm::{NoopGameEngine, StdoutTestGameEngine, ThreadTestGameEngine, MAX_THREAD_COUNT},
        Ciphel,
    };

    #[test]
    fn valid_spawn() {
        let mut engine = ThreadTestGameEngine {
            closed_thread: 0,
            spawned_thread: 0,
        };
        let mut ciphel = Ciphel::new();
        let tid = ciphel
            .start_arena(&mut engine)
            .expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }

        let (child_tid,err) = spawn();
        assert(err);
        main();
        
        "##;

        ciphel
            .compile::<ThreadTestGameEngine>(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");

        ciphel.run(&mut engine).expect("no error should arise");

        let child_tid = engine.spawned_thread;
        assert_eq!(child_tid, 1)
    }

    #[test]
    fn valid_close() {
        let mut engine = ThreadTestGameEngine {
            closed_thread: 0,
            spawned_thread: 0,
        };
        let mut ciphel = Ciphel::new();
        let tid = ciphel
            .start_arena(&mut engine)
            .expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }

        let (child_tid,err) = spawn();
        assert(err);
        main();
        
        "##;

        ciphel
            .compile::<ThreadTestGameEngine>(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");

        ciphel.run(&mut engine).expect("no error should arise");

        let child_tid = engine.spawned_thread;
        assert_eq!(child_tid, 1);

        let child_src = r##"
        
        fn add(x:u64,y:u64) -> u64 {
            return x+y;
        }
        let res = add(10,10);

        print(f"result = {res}");
            
        "##;

        ciphel
            .compile::<ThreadTestGameEngine>(crate::vm::vm::Player::P1, child_tid, child_src)
            .expect("Compilation should have succeeded");

        ciphel.run(&mut engine).expect("no error should arise");

        let src = r##"
        
            let err = close(child_tid);
            assert(err);
                
        "##;

        ciphel
            .compile::<ThreadTestGameEngine>(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");
        ciphel.run(&mut engine).expect("no error should arise");

        let closed_tid = engine.closed_thread;
        assert_eq!(closed_tid, 1);
        let src = r##"
            
            main();
                
        "##;

        ciphel
            .compile::<ThreadTestGameEngine>(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");
        ciphel.run(&mut engine).expect("no error should arise");
        let tids = ciphel.available_tids(crate::vm::vm::Player::P1);
        assert_eq!(tids.len(), MAX_THREAD_COUNT - 1);
    }

    #[test]
    fn valid_sleep() {
        let mut engine = NoopGameEngine {};
        let mut ciphel = Ciphel::new();
        let tid = ciphel
            .start_arena(&mut engine)
            .expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }
        sleep(5);
        main();
        
        "##;

        ciphel
            .compile::<ThreadTestGameEngine>(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");

        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
    }

    #[test]
    fn valid_wait_wake() {
        let mut engine = ThreadTestGameEngine {
            closed_thread: 0,
            spawned_thread: 0,
        };
        let mut ciphel = Ciphel::new();
        let tid = ciphel
            .start_arena(&mut engine)
            .expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }
        let (child_tid,err) = spawn();
        assert(err);
        wait();
        main();
        
        "##;

        ciphel
            .compile::<ThreadTestGameEngine>(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");
        ciphel.run(&mut engine).expect("no error should arise");

        let child_tid = engine.spawned_thread;

        let child_src = r##"
        
        sleep(3);
        wake(1);
        "##;

        ciphel
            .compile::<ThreadTestGameEngine>(crate::vm::vm::Player::P1, child_tid, child_src)
            .expect("Compilation should have succeeded");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
    }

    #[test]
    fn valid_join() {
        let mut engine = ThreadTestGameEngine {
            closed_thread: 0,
            spawned_thread: 0,
        };
        let mut ciphel = Ciphel::new();
        let tid = ciphel
            .start_arena(&mut engine)
            .expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }
        let (child_tid,err) = spawn();
        assert(err);
        join(child_tid);
        main();
        
        "##;

        ciphel
            .compile::<ThreadTestGameEngine>(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");
        ciphel.run(&mut engine).expect("no error should arise");

        let child_tid = engine.spawned_thread;

        let child_src = r##"
        sleep(5);
        "##;

        ciphel
            .compile::<ThreadTestGameEngine>(crate::vm::vm::Player::P1, child_tid, child_src)
            .expect("Compilation should have succeeded");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
    }

    #[test]
    fn valid_exit() {
        let mut engine = StdoutTestGameEngine { out: String::new() };
        let mut ciphel = Ciphel::new();
        let tid = ciphel
            .start_arena(&mut engine)
            .expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }
        exit();
        main();
        
        "##;

        ciphel
            .compile::<ThreadTestGameEngine>(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");
        ciphel.run(&mut engine).expect("no error should arise");
        let tids = ciphel.available_tids(crate::vm::vm::Player::P1);
        let output = engine.out;
        assert!(tids.len() == MAX_THREAD_COUNT);
        assert!(output.is_empty())
    }

    #[test]
    fn valid_join_exit() {
        let mut engine = ThreadTestGameEngine {
            closed_thread: 0,
            spawned_thread: 0,
        };
        let mut ciphel = Ciphel::new();
        let tid = ciphel
            .start_arena(&mut engine)
            .expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }
        let (child_tid,err) = spawn();
        assert(err);
        join(child_tid);
        main();
        
        "##;

        ciphel
            .compile::<ThreadTestGameEngine>(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");
        ciphel.run(&mut engine).expect("no error should arise");

        let child_tid = engine.spawned_thread;
        let child_src = r##"
        sleep(3);
        exit();
        let x = 4+5;
        "##;

        ciphel
            .compile::<ThreadTestGameEngine>(crate::vm::vm::Player::P1, child_tid, child_src)
            .expect("Compilation should have succeeded");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        let tids = ciphel.available_tids(crate::vm::vm::Player::P1);
        assert_eq!(tids.len(), 3);
    }
}
