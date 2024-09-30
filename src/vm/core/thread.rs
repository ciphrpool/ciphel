use crate::semantic::scope::static_types::{NumberType, PrimitiveType, StaticType};
use crate::semantic::{EType, ResolveCore, TypeOf};
use crate::vm::asm::operation::{OpPrimitive, PopNum};
use crate::vm::asm::Asm;
use crate::vm::core::lexem;
use crate::vm::core::CoreAsm;
use crate::vm::core::{ERROR_SLICE, OK_SLICE};
use crate::vm::external::ExternThreadIdentifier;
use crate::vm::runtime::RuntimeError;
use crate::vm::scheduler::Executable;
use crate::vm::signal::{SignalAction, SignalResult};
use crate::vm::GenerateCode;
use crate::{
    ast::expressions::Expression,
    semantic::{Resolve, SemanticError},
};
use crate::{e_static, err_tuple, p_num};

use super::PathFinder;

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
pub enum ThreadAsm {
    Spawn,
    Close,
    Exit,
    Wait,
    Wake,
    Sleep,
    Join,
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for ThreadAsm {
    fn name(
        &self,
        stdio: &mut crate::vm::stdio::StdIO,
        program: &crate::vm::program::Program<E>,
        engine: &mut E,
    ) {
        match self {
            ThreadAsm::Spawn => stdio.push_asm_lib(engine, "spawn"),
            ThreadAsm::Close => stdio.push_asm_lib(engine, "close"),
            ThreadAsm::Exit => stdio.push_asm_lib(engine, "exit"),
            ThreadAsm::Wait => stdio.push_asm_lib(engine, "wait"),
            ThreadAsm::Wake => stdio.push_asm_lib(engine, "wake"),
            ThreadAsm::Sleep => stdio.push_asm_lib(engine, "sleep"),
            ThreadAsm::Join => stdio.push_asm_lib(engine, "join"),
        }
    }
}

impl crate::vm::AsmWeight for ThreadAsm {
    fn weight(&self) -> crate::vm::Weight {
        match self {
            ThreadAsm::Spawn => crate::vm::Weight::END,
            ThreadAsm::Close => crate::vm::Weight::HIGH,
            ThreadAsm::Exit => crate::vm::Weight::END,
            ThreadAsm::Wait => crate::vm::Weight::END,
            ThreadAsm::Wake => crate::vm::Weight::HIGH,
            ThreadAsm::Sleep => crate::vm::Weight::END,
            ThreadAsm::Join => crate::vm::Weight::END,
        }
    }
}

impl PathFinder for ThreadFn {
    fn find(path: &[String], name: &str) -> Option<Self>
    where
        Self: Sized,
    {
        if (path.len() == 1 && path[0] == lexem::THREAD) || path.len() == 0 {
            return match name {
                lexem::SPAWN => Some(ThreadFn::Spawn),
                lexem::CLOSE => Some(ThreadFn::Close),
                lexem::EXIT => Some(ThreadFn::Exit),
                lexem::WAIT => Some(ThreadFn::Wait),
                lexem::WAKE => Some(ThreadFn::Wake),
                lexem::SLEEP => Some(ThreadFn::Sleep),
                lexem::JOIN => Some(ThreadFn::Join),
                _ => None,
            };
        }
        None
    }
}

fn expect_one_u64<E: crate::vm::external::Engine>(
    params: &mut Vec<Expression>,
    scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
    scope_id: Option<u128>,
) -> Result<(), SemanticError> {
    if params.len() != 1 {
        return Err(SemanticError::IncorrectArguments);
    }

    let size = &mut params[0];

    let _ = size.resolve::<E>(scope_manager, scope_id, &Some(p_num!(U64)), &mut None)?;
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
impl ResolveCore for ThreadFn {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        match self {
            ThreadFn::Spawn => {
                if parameters.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                Ok(err_tuple!(p_num!(U64)))
            }
            ThreadFn::Exit => {
                if parameters.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                Ok(e_static!(StaticType::Unit))
            }
            ThreadFn::Close => {
                let _ = expect_one_u64::<E>(parameters, scope_manager, scope_id);
                Ok(e_static!(StaticType::Error))
            }
            ThreadFn::Wait => {
                if parameters.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                Ok(e_static!(StaticType::Unit))
            }
            ThreadFn::Wake => {
                let _ = expect_one_u64::<E>(parameters, scope_manager, scope_id);
                Ok(e_static!(StaticType::Error))
            }
            ThreadFn::Sleep => {
                let _ = expect_one_u64::<E>(parameters, scope_manager, scope_id);
                Ok(e_static!(StaticType::Unit))
            }
            ThreadFn::Join => {
                let _ = expect_one_u64::<E>(parameters, scope_manager, scope_id);
                Ok(e_static!(StaticType::Error))
            }
        }
    }
}

impl GenerateCode for ThreadFn {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            ThreadFn::Spawn => instructions.push(Asm::Core(CoreAsm::Thread(ThreadAsm::Spawn))),
            ThreadFn::Exit => instructions.push(Asm::Core(CoreAsm::Thread(ThreadAsm::Exit))),
            ThreadFn::Close => instructions.push(Asm::Core(CoreAsm::Thread(ThreadAsm::Close))),
            ThreadFn::Wait => instructions.push(Asm::Core(CoreAsm::Thread(ThreadAsm::Wait))),
            ThreadFn::Wake => instructions.push(Asm::Core(CoreAsm::Thread(ThreadAsm::Wake))),
            ThreadFn::Sleep => instructions.push(Asm::Core(CoreAsm::Thread(ThreadAsm::Sleep))),
            ThreadFn::Join => instructions.push(Asm::Core(CoreAsm::Thread(ThreadAsm::Join))),
        }
        Ok(())
    }
}

// pub fn sig_spawn(
//     context: &mut SchedulerContext,
//     program: &mut crate::vm::program::Program,
//     stack: &mut Stack,
// ) -> Result<(), RuntimeError> {
//     if let Ok(tid) = context.request_spawn() {
//         let _ = stack.push_with(&(tid as u64).to_le_bytes())?;

//         let _ = stack.push_with(&OK_SLICE)?;
//         scheduler.next();
//         Ok(())
//     } else {
//         // Error TooManyThread
//         let _ = stack.push_with(&(0u64).to_le_bytes())?;

//         let _ = stack.push_with(&ERROR_SLICE)?;
//         scheduler.next();
//         Err(RuntimeError::TooManyThread)
//     }
// }

// pub fn sig_close(
//     tid: Tid,
//     context: &mut SchedulerContext,
//     program: &mut crate::vm::program::Program,
//     stack: &mut Stack,
// ) -> Result<(), RuntimeError> {
//     if context.request_close(tid).is_ok() {
//         let _ = stack.push_with(&OK_SLICE)?;
//         scheduler.next();
//         Ok(())
//     } else {
//         // Error InvalidTID
//         let _ = stack.push_with(&(0u64).to_le_bytes())?;

//         let _ = stack.push_with(&ERROR_SLICE)?;
//         scheduler.next();
//         Err(RuntimeError::InvalidTID(tid))
//     }
// }

// pub fn sig_wait(
//     state: &mut ThreadState,
//     program: &mut crate::vm::program::Program,
// ) -> Result<(), RuntimeError> {
//     let _ = state.to(ThreadState::WAITING)?;
//     scheduler.next();
//     Err(RuntimeError::Signal(Signal::WAIT))
// }

// pub fn sig_wake(
//     tid: Tid,
//     context: &mut SchedulerContext,
//     program: &mut crate::vm::program::Program,
//     stack: &mut Stack,
// ) -> Result<(), RuntimeError> {
//     if context.request_wake(tid).is_ok() {
//         let _ = stack.push_with(&OK_SLICE)?;
//         scheduler.next();
//         Ok(())
//     } else {
//         let _ = stack.push_with(&ERROR_SLICE)?;
//         scheduler.next();
//         Err(RuntimeError::InvalidTID(tid))
//     }
// }

// pub fn sig_sleep(
//     nb_maf: &usize,
//     state: &mut ThreadState,
//     program: &mut crate::vm::program::Program,
// ) -> Result<(), RuntimeError> {
//     let _ = state.to(ThreadState::SLEEPING(*nb_maf))?;
//     scheduler.next();
//     Err(RuntimeError::Signal(Signal::SLEEP(*nb_maf)))
// }

// pub fn sig_join(
//     own_tid: Tid,
//     join_tid: Tid,
//     context: &mut SchedulerContext,
//     state: &mut ThreadState,
//     program: &mut crate::vm::program::Program,
//     stack: &mut Stack,
//     waiting_list: &mut Vec<WaitingStatus>,
// ) -> Result<(), RuntimeError> {
//     if let Ok(true) = context.is_alive(join_tid) {
//         let _ = state.to(ThreadState::WAITING)?;
//         waiting_list.push(WaitingStatus::Join {
//             join_tid: own_tid,
//             to_be_completed_tid: join_tid,
//         });
//         let _ = stack.push_with(&[true as u8])?;

//         let _ = stack.push_with(&OK_SLICE)?;
//         scheduler.next();
//         Err(RuntimeError::Signal(Signal::JOIN(join_tid)))
//     } else {
//         let _ = stack.push_with(&ERROR_SLICE)?;
//         scheduler.next();
//         Err(RuntimeError::InvalidTID(join_tid))
//     }
// }
// pub fn sig_exit(
//     tid: Tid,
//     context: &mut SchedulerContext,
//     state: &mut ThreadState,
//     waiting_list: &mut Vec<WaitingStatus>,
// ) -> Result<(), RuntimeError> {
//     let _ = state.to(ThreadState::EXITED);
//     context.request_close(tid)?;
//     let idx_tid_list: Vec<(usize, usize)> = waiting_list
//         .iter()
//         .enumerate()
//         .filter(|(_, status)| match status {
//             WaitingStatus::Join {
//                 join_tid,
//                 to_be_completed_tid,
//             } => *to_be_completed_tid == tid,
//             WaitingStatus::ForStdin(_) => false,
//         })
//         .map(|(idx, status)| {
//             (
//                 idx,
//                 match status {
//                     WaitingStatus::Join {
//                         join_tid,
//                         to_be_completed_tid,
//                     } => *join_tid,
//                     WaitingStatus::ForStdin(id) => *id, // unreachable,
//                 },
//             )
//         })
//         .collect();

//     for (idx, tid) in idx_tid_list {
//         waiting_list.remove(idx);
//         context.request_wake(tid)?;
//     }

//     Err(RuntimeError::Signal(Signal::EXIT))
// }

// pub fn sig_wait_stdin(
//     tid: Tid,
//     state: &mut ThreadState,
//     waiting_list: &mut Vec<WaitingStatus>,
// ) -> Result<(), RuntimeError> {
//     let _ = state.to(ThreadState::WAITING)?;
//     waiting_list.push(WaitingStatus::ForStdin(tid));
//     Err(RuntimeError::Signal(Signal::WAIT_STDIN))
// }

impl<E: crate::vm::external::Engine> Executable<E> for ThreadAsm {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::TID>,
    ) -> Result<(), RuntimeError> {
        match self {
            ThreadAsm::Spawn => {
                // request spawn of an other another thread
                fn spawn_callback<E: crate::vm::external::Engine>(
                    response: crate::vm::signal::SignalResult<E>,
                    stack: &mut crate::vm::allocator::stack::Stack,
                ) -> Result<(), RuntimeError> {
                    match response {
                        SignalResult::Ok(SignalAction::Spawn(tid)) => {
                            stack.push_with(&tid.to_u64().to_le_bytes())?;
                            stack.push_with(&OK_SLICE)?;
                        }
                        SignalResult::Error => {
                            stack.push_with(&0u64.to_le_bytes())?;
                            stack.push_with(&ERROR_SLICE)?;
                        }
                        SignalResult::Ok(_) => {} // Unreachable
                    }
                    Ok(())
                }
                let _ = signal_handler.notify(
                    crate::vm::signal::Signal::Spawn,
                    stack,
                    engine,
                    context.tid.clone(),
                    spawn_callback::<E>,
                )?;
                scheduler.next();
                Ok(())
            }
            ThreadAsm::Close => {
                // request close of an other another thread
                fn close_callback<E: crate::vm::external::Engine>(
                    response: crate::vm::signal::SignalResult<E>,
                    stack: &mut crate::vm::allocator::stack::Stack,
                ) -> Result<(), RuntimeError> {
                    match response {
                        SignalResult::Ok(SignalAction::Close(_)) => {
                            stack.push_with(&OK_SLICE)?;
                        }
                        SignalResult::Error => {
                            stack.push_with(&ERROR_SLICE)?;
                        }
                        SignalResult::Ok(_) => {} // Unreachable
                    }
                    Ok(())
                }
                let tid = E::TID::from_u64(OpPrimitive::pop_num::<u64>(stack)?);
                let _ = signal_handler.notify(
                    crate::vm::signal::Signal::Close(tid),
                    stack,
                    engine,
                    context.tid.clone(),
                    close_callback::<E>,
                )?;
                scheduler.next();
                Ok(())
            }
            ThreadAsm::Exit => {
                // request exit of an other another thread
                fn exit_callback<E: crate::vm::external::Engine>(
                    response: crate::vm::signal::SignalResult<E>,
                    stack: &mut crate::vm::allocator::stack::Stack,
                ) -> Result<(), RuntimeError> {
                    match response {
                        SignalResult::Ok(SignalAction::Exit(_)) => {}
                        SignalResult::Error => return Err(RuntimeError::SignalError),
                        SignalResult::Ok(_) => {} // Unreachable
                    }
                    Ok(())
                }
                let _ = signal_handler.notify(
                    crate::vm::signal::Signal::Exit,
                    stack,
                    engine,
                    context.tid.clone(),
                    exit_callback::<E>,
                )?;
                scheduler.next();
                Ok(())
            }
            ThreadAsm::Wait => {
                // request wait of an other another thread
                fn wait_callback<E: crate::vm::external::Engine>(
                    response: crate::vm::signal::SignalResult<E>,
                    stack: &mut crate::vm::allocator::stack::Stack,
                ) -> Result<(), RuntimeError> {
                    match response {
                        SignalResult::Ok(SignalAction::Wait { caller }) => {}
                        SignalResult::Error => return Err(RuntimeError::SignalError),
                        SignalResult::Ok(_) => {} // Unreachable
                    }
                    Ok(())
                }
                let _ = signal_handler.notify(
                    crate::vm::signal::Signal::Wait,
                    stack,
                    engine,
                    context.tid.clone(),
                    wait_callback::<E>,
                )?;
                scheduler.next();
                Ok(())
            }
            ThreadAsm::Wake => {
                // request wake of an other another thread
                fn wake_callback<E: crate::vm::external::Engine>(
                    response: crate::vm::signal::SignalResult<E>,
                    stack: &mut crate::vm::allocator::stack::Stack,
                ) -> Result<(), RuntimeError> {
                    match response {
                        SignalResult::Ok(SignalAction::Wake { .. }) => {
                            stack.push_with(&OK_SLICE)?;
                        }
                        SignalResult::Error => {
                            stack.push_with(&ERROR_SLICE)?;
                        }
                        SignalResult::Ok(_) => {} // Unreachable
                    }
                    Ok(())
                }
                let target = E::TID::from_u64(OpPrimitive::pop_num::<u64>(stack)?);
                let _ = signal_handler.notify(
                    crate::vm::signal::Signal::Wake(target),
                    stack,
                    engine,
                    context.tid.clone(),
                    wake_callback::<E>,
                )?;
                scheduler.next();
                Ok(())
            }
            ThreadAsm::Sleep => {
                // request sleep of an other another thread
                fn sleep_callback<E: crate::vm::external::Engine>(
                    response: crate::vm::signal::SignalResult<E>,
                    stack: &mut crate::vm::allocator::stack::Stack,
                ) -> Result<(), RuntimeError> {
                    match response {
                        SignalResult::Ok(SignalAction::Sleep { .. }) => {}
                        SignalResult::Error => return Err(RuntimeError::SignalError),
                        SignalResult::Ok(_) => {} // Unreachable
                    }
                    Ok(())
                }
                let time = OpPrimitive::pop_num::<u64>(stack)? as usize;
                let _ = signal_handler.notify(
                    crate::vm::signal::Signal::Sleep { time },
                    stack,
                    engine,
                    context.tid.clone(),
                    sleep_callback::<E>,
                )?;
                scheduler.next();
                Ok(())
            }
            ThreadAsm::Join => {
                // request join of an other another thread
                fn join_callback<E: crate::vm::external::Engine>(
                    response: crate::vm::signal::SignalResult<E>,
                    stack: &mut crate::vm::allocator::stack::Stack,
                ) -> Result<(), RuntimeError> {
                    match response {
                        SignalResult::Ok(SignalAction::Join { .. }) => {
                            stack.push_with(&OK_SLICE)?;
                        }
                        SignalResult::Error => {
                            stack.push_with(&ERROR_SLICE)?;
                        }
                        SignalResult::Ok(_) => {} // Unreachable
                    }
                    Ok(())
                }

                let target = E::TID::from_u64(OpPrimitive::pop_num::<u64>(stack)?);
                let _ = signal_handler.notify(
                    crate::vm::signal::Signal::Join(target),
                    stack,
                    engine,
                    context.tid.clone(),
                    join_callback::<E>,
                )?;
                scheduler.next();
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use crate::{
    //     vm::vm::{NoopEngine, StdoutTestEngine, ThreadTestEngine, MAX_THREAD_COUNT},
    //     Ciphel,
    // };

    use crate::{
        ast::statements::parse_statements,
        semantic::Resolve,
        test_extract_variable,
        vm::{
            allocator::heap::Heap,
            external::test::DefaultThreadID,
            runtime::{Runtime, Thread, ThreadContext, ThreadState},
            scheduler::QueuePolicy,
            stdio::StdIO,
            GenerateCode,
        },
    };

    pub fn compile_for<E: crate::vm::external::Engine>(
        input: &str,
        tid: &E::TID,
        runtime: &mut Runtime<E, QueuePolicy>,
    ) {
        let mut statements =
            parse_statements(input.into(), 0).expect("Parsing should have succeeded");

        let ThreadContext {
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

    #[test]
    fn valid_spawn() {
        let mut engine = crate::vm::external::test::ThreadTestEngine {
            id_auto_increment: 0,
        };

        let mut heap = Heap::new();
        let mut stdio = StdIO::default();
        let mut runtime = Runtime::default();

        let tid_1 = runtime
            .spawn(&mut engine)
            .expect("Spawning should have succeeded");

        compile_for(
            r##"
        let (tid,err) = spawn();
        "##,
            &tid_1,
            &mut runtime,
        );

        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");

        assert_eq!(runtime.snapshot().states.len(), 2);

        let tid_2 = {
            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_1)
                .expect("Thread should have been found");

            let id = test_extract_variable::<u64>("tid", scope_manager, stack, &heap)
                .expect("Variable should have been found");
            DefaultThreadID(id)
        };

        compile_for(
            r##"
        let x = 1;
        "##,
            &tid_1,
            &mut runtime,
        );
        compile_for(
            r##"
        let x = 2;
        "##,
            &tid_2,
            &mut runtime,
        );
        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        {
            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_1)
                .expect("Thread should have been found");

            let x_t1 = test_extract_variable::<i64>("x", scope_manager, stack, &heap)
                .expect("Variable should have been found");

            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_2)
                .expect("Thread should have been found");
            let x_t2 = test_extract_variable::<i64>("x", scope_manager, stack, &heap)
                .expect("Variable should have been found");

            assert_eq!(x_t1, 1);
            assert_eq!(x_t2, 2);
        }
    }

    #[test]
    fn valid_exit() {
        let mut engine = crate::vm::external::test::ThreadTestEngine {
            id_auto_increment: 0,
        };

        let mut heap = Heap::new();
        let mut stdio = StdIO::default();
        let mut runtime = Runtime::default();

        let tid_1 = runtime
            .spawn(&mut engine)
            .expect("Spawning should have succeeded");

        compile_for(
            r##"
        exit();
        "##,
            &tid_1,
            &mut runtime,
        );

        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        assert_eq!(runtime.snapshot().states.len(), 0);
    }

    #[test]
    fn valid_close() {
        let mut engine = crate::vm::external::test::ThreadTestEngine {
            id_auto_increment: 0,
        };

        let mut heap = Heap::new();
        let mut stdio = StdIO::default();
        let mut runtime = Runtime::default();

        let tid_1 = runtime
            .spawn(&mut engine)
            .expect("Spawning should have succeeded");

        compile_for(
            r##"
        let (tid,err) = spawn();
        "##,
            &tid_1,
            &mut runtime,
        );

        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");

        assert_eq!(runtime.snapshot().states.len(), 2);

        let tid_2 = {
            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_1)
                .expect("Thread should have been found");

            let id = test_extract_variable::<u64>("tid", scope_manager, stack, &heap)
                .expect("Variable should have been found");
            DefaultThreadID(id)
        };

        compile_for(
            &format!(
                r##"
        let err = close({});
        "##,
                tid_2.0
            ),
            &tid_1,
            &mut runtime,
        );
        compile_for(
            r##"
        let x = 2;
        "##,
            &tid_2,
            &mut runtime,
        );
        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        assert_eq!(runtime.snapshot().states.len(), 1);
    }

    #[test]
    fn valid_sleep() {
        let mut engine = crate::vm::external::test::ThreadTestEngine {
            id_auto_increment: 0,
        };

        let mut heap = Heap::new();
        let mut stdio = StdIO::default();
        let mut runtime = Runtime::default();

        let tid_1 = runtime
            .spawn(&mut engine)
            .expect("Spawning should have succeeded");

        compile_for(
            r##"
        sleep(4);
        let x = 2;
        "##,
            &tid_1,
            &mut runtime,
        );

        {
            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_1)
                .expect("Thread should have been found");

            let x = test_extract_variable::<u64>("x", scope_manager, stack, &heap)
                .expect("Variable should have been found");
            assert_ne!(x, 2);
        }

        for i in (1..=4).rev() {
            let _ = runtime
                .run(&mut heap, &mut stdio, &mut engine)
                .expect("Execution should have succeeded");
            if ThreadState::SLEEPING(i)
                != *runtime
                    .snapshot()
                    .states
                    .get(&tid_1)
                    .expect("Thread should exist")
            {
                panic!("Thread should have been sleeping {}", i);
            }
        }
        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");

        if ThreadState::IDLE
            != *runtime
                .snapshot()
                .states
                .get(&tid_1)
                .expect("Thread should exist")
        {
            panic!("Thread should have been idle");
        }
        {
            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_1)
                .expect("Thread should have been found");

            let x = test_extract_variable::<u64>("x", scope_manager, stack, &heap)
                .expect("Variable should have been found");
            assert_eq!(x, 2);
        }
    }

    #[test]
    fn valid_join() {
        let mut engine = crate::vm::external::test::ThreadTestEngine {
            id_auto_increment: 0,
        };

        let mut heap = Heap::new();
        let mut stdio = StdIO::default();
        let mut runtime = Runtime::default();

        let tid_1 = runtime
            .spawn(&mut engine)
            .expect("Spawning should have succeeded");

        compile_for(
            r##"
        let (tid,err) = spawn();
        "##,
            &tid_1,
            &mut runtime,
        );

        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");

        assert_eq!(runtime.snapshot().states.len(), 2);

        let tid_2 = {
            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_1)
                .expect("Thread should have been found");

            let id = test_extract_variable::<u64>("tid", scope_manager, stack, &heap)
                .expect("Variable should have been found");
            DefaultThreadID(id)
        };

        compile_for(
            &format!(
                r##"
        let err = join({});
        "##,
                tid_2.0
            ),
            &tid_1,
            &mut runtime,
        );
        compile_for(
            r##"
        let x = 2;
        "##,
            &tid_2,
            &mut runtime,
        );
        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        if (ThreadState::JOINING {
            target: tid_2.clone(),
        }) != *runtime
            .snapshot()
            .states
            .get(&tid_1)
            .expect("Thread should exist")
        {
            panic!("Thread should have been sleeping");
        }
        {
            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_2)
                .expect("Thread should have been found");
            let x_t2 = test_extract_variable::<i64>("x", scope_manager, stack, &heap)
                .expect("Variable should have been found");

            assert_eq!(x_t2, 2);
        }
        compile_for(
            r##"
        let x = 1;
        "##,
            &tid_1,
            &mut runtime,
        );
        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        {
            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_1)
                .expect("Thread should have been found");
            let x_t1 = test_extract_variable::<i64>("x", scope_manager, stack, &heap)
                .expect("Variable should have been found");

            assert_eq!(x_t1, 1);
        }
    }

    #[test]
    fn valid_wait_wake() {
        let mut engine = crate::vm::external::test::ThreadTestEngine {
            id_auto_increment: 0,
        };

        let mut heap = Heap::new();
        let mut stdio = StdIO::default();
        let mut runtime = Runtime::default();

        let tid_1 = runtime
            .spawn(&mut engine)
            .expect("Spawning should have succeeded");

        compile_for(
            r##"
        let (tid,err) = spawn();
        "##,
            &tid_1,
            &mut runtime,
        );

        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");

        assert_eq!(runtime.snapshot().states.len(), 2);

        let tid_2 = {
            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_1)
                .expect("Thread should have been found");

            let id = test_extract_variable::<u64>("tid", scope_manager, stack, &heap)
                .expect("Variable should have been found");
            DefaultThreadID(id)
        };

        compile_for(
            r##"
        wait();
        "##,
            &tid_1,
            &mut runtime,
        );
        compile_for(
            r##"
        let x = 2;
        "##,
            &tid_2,
            &mut runtime,
        );
        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");

        if ThreadState::WAITING
            != *runtime
                .snapshot()
                .states
                .get(&tid_1)
                .expect("Thread should exist")
        {
            panic!("Thread should have been waiting");
        }
        {
            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_2)
                .expect("Thread should have been found");
            let x_t2 = test_extract_variable::<i64>("x", scope_manager, stack, &heap)
                .expect("Variable should have been found");

            assert_eq!(x_t2, 2);
        }
        compile_for(
            r##"
        let x = 1;
        "##,
            &tid_1,
            &mut runtime,
        );
        compile_for(
            r##"
        x = 3;
        "##,
            &tid_2,
            &mut runtime,
        );
        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        if ThreadState::WAITING
            != *runtime
                .snapshot()
                .states
                .get(&tid_1)
                .expect("Thread should exist")
        {
            panic!("Thread should have been waiting");
        }
        {
            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_2)
                .expect("Thread should have been found");
            let x_t2 = test_extract_variable::<i64>("x", scope_manager, stack, &heap)
                .expect("Variable should have been found");

            assert_eq!(x_t2, 3);
        }
        compile_for(
            &format!(
                r##"
        let err = wake({});
        "##,
                tid_1.0
            ),
            &tid_2,
            &mut runtime,
        );
        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");

        let _ = runtime
            .run(&mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        {
            let (Thread { stack, .. }, ThreadContext { scope_manager, .. }) = runtime
                .thread_with_context_of(&tid_1)
                .expect("Thread should have been found");
            let x_t1 = test_extract_variable::<i64>("x", scope_manager, stack, &heap)
                .expect("Variable should have been found");

            assert_eq!(x_t1, 1);
        }
    }

    // #[test]
    // fn valid_spawn() {
    //     let mut engine = ThreadTestEngine {
    //         closed_thread: 0,
    //         spawned_thread: 0,
    //     };
    //     let mut ciphel = Ciphel::new();
    //     let tid = ciphel
    //         .start_arena(&mut engine)
    //         .expect("starting should not fail");

    //     let src = r##"

    //     fn main() -> Unit {
    //         print("Hello World");
    //     }

    //     let (child_tid,err) = spawn();
    //     assert(err);
    //     main();

    //     "##;

    //     ciphel
    //         .compile::<ThreadTestEngine>(crate::vm::vm::Player::P1, tid, src)
    //         .expect("Compilation should have succeeded");

    //     ciphel.run(&mut engine).expect("no error should arise");

    //     let child_tid = engine.spawned_thread;
    //     assert_eq!(child_tid, 1)
    // }

    // #[test]
    // fn valid_close() {
    //     let mut engine = ThreadTestEngine {
    //         closed_thread: 0,
    //         spawned_thread: 0,
    //     };
    //     let mut ciphel = Ciphel::new();
    //     let tid = ciphel
    //         .start_arena(&mut engine)
    //         .expect("starting should not fail");

    //     let src = r##"

    //     fn main() -> Unit {
    //         print("Hello World");
    //     }

    //     let (child_tid,err) = spawn();
    //     assert(err);
    //     main();

    //     "##;

    //     ciphel
    //         .compile::<ThreadTestEngine>(crate::vm::vm::Player::P1, tid, src)
    //         .expect("Compilation should have succeeded");

    //     ciphel.run(&mut engine).expect("no error should arise");

    //     let child_tid = engine.spawned_thread;
    //     assert_eq!(child_tid, 1);

    //     let child_src = r##"

    //     fn add(x:u64,y:u64) -> u64 {
    //         return x+y;
    //     }
    //     let res = add(10,10);

    //     print(f"result = {res}");

    //     "##;

    //     ciphel
    //         .compile::<ThreadTestEngine>(crate::vm::vm::Player::P1, child_tid, child_src)
    //         .expect("Compilation should have succeeded");

    //     ciphel.run(&mut engine).expect("no error should arise");

    //     let src = r##"

    //         let err = close(child_tid);
    //         assert(err);

    //     "##;

    //     ciphel
    //         .compile::<ThreadTestEngine>(crate::vm::vm::Player::P1, tid, src)
    //         .expect("Compilation should have succeeded");
    //     ciphel.run(&mut engine).expect("no error should arise");

    //     let closed_tid = engine.closed_thread;
    //     assert_eq!(closed_tid, 1);
    //     let src = r##"

    //         main();

    //     "##;

    //     ciphel
    //         .compile::<ThreadTestEngine>(crate::vm::vm::Player::P1, tid, src)
    //         .expect("Compilation should have succeeded");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     let tids = ciphel.available_tids(crate::vm::vm::Player::P1);
    //     assert_eq!(tids.len(), MAX_THREAD_COUNT - 1);
    // }

    // #[test]
    // fn valid_sleep() {
    //     let mut engine = NoopEngine {};
    //     let mut ciphel = Ciphel::new();
    //     let tid = ciphel
    //         .start_arena(&mut engine)
    //         .expect("starting should not fail");

    //     let src = r##"

    //     fn main() -> Unit {
    //         print("Hello World");
    //     }
    //     sleep(5);
    //     main();

    //     "##;

    //     ciphel
    //         .compile::<ThreadTestEngine>(crate::vm::vm::Player::P1, tid, src)
    //         .expect("Compilation should have succeeded");

    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    // }

    // #[test]
    // fn valid_wait_wake() {
    //     let mut engine = ThreadTestEngine {
    //         closed_thread: 0,
    //         spawned_thread: 0,
    //     };
    //     let mut ciphel = Ciphel::new();
    //     let tid = ciphel
    //         .start_arena(&mut engine)
    //         .expect("starting should not fail");

    //     let src = r##"

    //     fn main() -> Unit {
    //         print("Hello World");
    //     }
    //     let (child_tid,err) = spawn();
    //     assert(err);
    //     wait();
    //     main();

    //     "##;

    //     ciphel
    //         .compile::<ThreadTestEngine>(crate::vm::vm::Player::P1, tid, src)
    //         .expect("Compilation should have succeeded");
    //     ciphel.run(&mut engine).expect("no error should arise");

    //     let child_tid = engine.spawned_thread;

    //     let child_src = r##"

    //     sleep(3);
    //     wake(1);
    //     "##;

    //     ciphel
    //         .compile::<ThreadTestEngine>(crate::vm::vm::Player::P1, child_tid, child_src)
    //         .expect("Compilation should have succeeded");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    // }

    // #[test]
    // fn valid_join() {
    //     let mut engine = ThreadTestEngine {
    //         closed_thread: 0,
    //         spawned_thread: 0,
    //     };
    //     let mut ciphel = Ciphel::new();
    //     let tid = ciphel
    //         .start_arena(&mut engine)
    //         .expect("starting should not fail");

    //     let src = r##"

    //     fn main() -> Unit {
    //         print("Hello World");
    //     }
    //     let (child_tid,err) = spawn();
    //     assert(err);
    //     join(child_tid);
    //     main();

    //     "##;

    //     ciphel
    //         .compile::<ThreadTestEngine>(crate::vm::vm::Player::P1, tid, src)
    //         .expect("Compilation should have succeeded");
    //     ciphel.run(&mut engine).expect("no error should arise");

    //     let child_tid = engine.spawned_thread;

    //     let child_src = r##"
    //     sleep(5);
    //     "##;

    //     ciphel
    //         .compile::<ThreadTestEngine>(crate::vm::vm::Player::P1, child_tid, child_src)
    //         .expect("Compilation should have succeeded");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    // }

    // #[test]
    // fn valid_exit() {
    //     let mut engine = StdoutTestEngine { out: String::new() };
    //     let mut ciphel = Ciphel::new();
    //     let tid = ciphel
    //         .start_arena(&mut engine)
    //         .expect("starting should not fail");

    //     let src = r##"

    //     fn main() -> Unit {
    //         print("Hello World");
    //     }
    //     exit();
    //     main();

    //     "##;

    //     ciphel
    //         .compile::<ThreadTestEngine>(crate::vm::vm::Player::P1, tid, src)
    //         .expect("Compilation should have succeeded");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     let tids = ciphel.available_tids(crate::vm::vm::Player::P1);
    //     let output = engine.out;
    //     assert!(tids.len() == MAX_THREAD_COUNT);
    //     assert!(output.is_empty())
    // }

    // #[test]
    // fn valid_join_exit() {
    //     let mut engine = ThreadTestEngine {
    //         closed_thread: 0,
    //         spawned_thread: 0,
    //     };
    //     let mut ciphel = Ciphel::new();
    //     let tid = ciphel
    //         .start_arena(&mut engine)
    //         .expect("starting should not fail");

    //     let src = r##"

    //     fn main() -> Unit {
    //         print("Hello World");
    //     }
    //     let (child_tid,err) = spawn();
    //     assert(err);
    //     join(child_tid);
    //     main();

    //     "##;

    //     ciphel
    //         .compile::<ThreadTestEngine>(crate::vm::vm::Player::P1, tid, src)
    //         .expect("Compilation should have succeeded");
    //     ciphel.run(&mut engine).expect("no error should arise");

    //     let child_tid = engine.spawned_thread;
    //     let child_src = r##"
    //     sleep(3);
    //     exit();
    //     let x = 4+5;
    //     "##;

    //     ciphel
    //         .compile::<ThreadTestEngine>(crate::vm::vm::Player::P1, child_tid, child_src)
    //         .expect("Compilation should have succeeded");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     ciphel.run(&mut engine).expect("no error should arise");
    //     let tids = ciphel.available_tids(crate::vm::vm::Player::P1);
    //     assert_eq!(tids.len(), 3);
    // }
}
