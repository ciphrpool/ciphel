use std::cell::Ref;
use std::collections::HashSet;

use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::static_types::{NumberType, PrimitiveType, StaticType, TupleType};
use crate::semantic::{Either, TypeOf};
use crate::vm::allocator::stack::Stack;
use crate::vm::casm::operation::OpPrimitive;
use crate::vm::casm::Casm;
use crate::vm::platform::utils::lexem;
use crate::vm::platform::LibCasm;
use crate::vm::vm::{
    CasmMetadata, Executable, RuntimeError, Signal, ThreadState, Tid, MAX_THREAD_COUNT,
};
use crate::{
    ast::expressions::Expression,
    semantic::{EType, MutRc, Resolve, SemanticError},
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

impl CasmMetadata for ThreadCasm {
    fn name(&self, stdio: &mut crate::vm::stdio::StdIO, program: &CasmProgram) {
        match self {
            ThreadCasm::Spawn => stdio.push_casm_lib("spawn"),
            ThreadCasm::Close => stdio.push_casm_lib("close"),
            ThreadCasm::Exit => stdio.push_casm_lib("exit"),
            ThreadCasm::Wait => stdio.push_casm_lib("wait"),
            ThreadCasm::Wake => stdio.push_casm_lib("wake"),
            ThreadCasm::Sleep => stdio.push_casm_lib("sleep"),
            ThreadCasm::Join => stdio.push_casm_lib("join"),
        }
    }
}
impl ThreadFn {
    pub fn from(suffixe: &Option<String>, id: &String) -> Option<Self> {
        match suffixe {
            Some(suffixe) => {
                if suffixe != lexem::CORE {
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
fn expect_one_u64(params: &Vec<Expression>, scope: &MutRc<Scope>) -> Result<(), SemanticError> {
    if params.len() != 1 {
        return Err(SemanticError::IncorrectArguments);
    }

    let size = &params[0];

    let _ = size.resolve(scope, &Some(p_num!(U64)), &None)?;
    let size_type = size.type_of(&scope.borrow())?;
    match &size_type {
        Either::Static(value) => match value.as_ref() {
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
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        _context: &Self::Context,
        extra: &Self::Extra,
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
            ThreadFn::Close => expect_one_u64(extra, scope),
            ThreadFn::Wait => {
                if extra.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                Ok(())
            }
            ThreadFn::Wake => expect_one_u64(extra, scope),
            ThreadFn::Sleep => expect_one_u64(extra, scope),
            ThreadFn::Join => expect_one_u64(extra, scope),
        }
    }
}
impl TypeOf for ThreadFn {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
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
        _scope: &MutRc<Scope>,
        instructions: &CasmProgram,
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
    requested_spawn_count: &mut usize,
    availaible_tids: &mut Vec<Tid>,
    spawned_tid: &mut Vec<Tid>,
    program: &CasmProgram,
    stack: &mut Stack,
) -> Result<(), RuntimeError> {
    let tid = *requested_spawn_count + 1;
    if availaible_tids.len() < MAX_THREAD_COUNT {
        *requested_spawn_count += 1;
        availaible_tids.push(tid);
        spawned_tid.push(tid);
        let _ = stack
            .push_with(&(tid as u64).to_le_bytes())
            .map_err(|e| e.into())?;

        // TODO : NO_ERROR value
        let _ = stack.push_with(&[0u8]).map_err(|e| e.into())?;
        program.incr();
        Ok(())
    } else {
        // Error TooManyThread
        let _ = stack
            .push_with(&(0u64).to_le_bytes())
            .map_err(|e| e.into())?;

        // TODO : ERROR value
        let _ = stack.push_with(&[1u8]).map_err(|e| e.into())?;
        program.incr();
        Err(RuntimeError::TooManyThread)
    }
}

pub fn sig_close(
    tid: &Tid,
    availaible_tids: &mut Vec<Tid>,
    closed_tid: &mut Vec<Tid>,
    program: &CasmProgram,
    stack: &mut Stack,
) -> Result<(), RuntimeError> {
    if availaible_tids.contains(&tid) {
        let idx = availaible_tids.iter().position(|i| *i == *tid).unwrap();
        availaible_tids.remove(idx);
        closed_tid.push(*tid);

        // TODO : NO_ERROR value
        let _ = stack.push_with(&[0u8]).map_err(|e| e.into())?;
        program.incr();
        Ok(())
    } else {
        // Error InvalidTID
        let _ = stack
            .push_with(&(0u64).to_le_bytes())
            .map_err(|e| e.into())?;

        // TODO : ERROR value
        let _ = stack.push_with(&[1u8]).map_err(|e| e.into())?;
        program.incr();
        Err(RuntimeError::InvalidTID(*tid))
    }
}

pub fn sig_wait(state: &mut ThreadState, program: &CasmProgram) -> Result<(), RuntimeError> {
    let _ = state.to(ThreadState::WAITING)?;
    program.incr();
    Err(RuntimeError::Signal(Signal::WAIT))
}

pub fn sig_wake(
    tid: &Tid,
    availaible_tids: &mut Vec<Tid>,
    wakingup_tid: &mut HashSet<Tid>,
    program: &CasmProgram,
    stack: &mut Stack,
) -> Result<(), RuntimeError> {
    if availaible_tids.contains(tid) {
        wakingup_tid.insert(*tid);
        // TODO : NO_ERROR value
        let _ = stack.push_with(&[0u8]).map_err(|e| e.into())?;
        program.incr();
        Ok(())
    } else {
        // TODO : ERROR value
        let _ = stack.push_with(&[1u8]).map_err(|e| e.into())?;
        program.incr();
        Err(RuntimeError::InvalidTID(*tid))
    }
}

pub fn sig_sleep(
    nb_maf: &usize,
    state: &mut ThreadState,
    program: &CasmProgram,
) -> Result<(), RuntimeError> {
    let _ = state.to(ThreadState::SLEEPING(*nb_maf))?;
    program.incr();
    Err(RuntimeError::Signal(Signal::SLEEP(*nb_maf)))
}

pub fn sig_join(
    own_tid: Tid,
    join_tid: Tid,
    state: &mut ThreadState,
    join_waiting_list: &mut Vec<(Tid, Tid)>,
    availaible_tids: &mut Vec<Tid>,
    program: &CasmProgram,
    stack: &mut Stack,
) -> Result<(), RuntimeError> {
    dbg!((own_tid, join_tid));
    if availaible_tids.contains(&join_tid) {
        let _ = state.to(ThreadState::WAITING)?;
        join_waiting_list.push((own_tid, join_tid));
        let _ = stack.push_with(&[true as u8]).map_err(|e| e.into())?;

        // TODO : NO_ERROR value
        let _ = stack.push_with(&[0u8]).map_err(|e| e.into())?;
        program.incr();
        Err(RuntimeError::Signal(Signal::JOIN(join_tid)))
    } else {
        // TODO : ERROR value
        let _ = stack.push_with(&[1u8]).map_err(|e| e.into())?;
        program.incr();
        Err(RuntimeError::InvalidTID(join_tid))
    }
}
pub fn sig_exit(
    tid: Tid,
    state: &mut ThreadState,
    closed_tid: &mut Vec<Tid>,
    join_waiting_list: &mut Vec<(Tid, Tid)>,
    wakingup_tid: &mut HashSet<Tid>,
) -> Result<(), RuntimeError> {
    let _ = state.to(ThreadState::COMPLETED);
    closed_tid.push(tid);
    let idx_tid_list: Vec<(usize, usize)> = join_waiting_list
        .iter()
        .enumerate()
        .filter(|(_, (_, otid))| *otid == tid)
        .map(|(idx, (tid, _))| (idx, *tid))
        .collect();

    for (idx, tid) in idx_tid_list {
        join_waiting_list.remove(idx);
        wakingup_tid.insert(tid);
    }

    Err(RuntimeError::Signal(Signal::EXIT))
}

impl Executable for ThreadCasm {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
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
    use crate::Ciphel;

    use super::*;

    #[test]
    fn valid_spawn() {
        let mut ciphel = Ciphel::new();
        let tid = ciphel.start().expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }

        let (child_tid,err) = spawn();
        main();
        
        "##;

        ciphel
            .compile(tid, src)
            .expect("Compilation should have succeeded");

        ciphel.run().expect("no error should arise");
        let tids = ciphel.available_tids();
        assert!(tids.len() > 1);
    }

    #[test]
    fn valid_close() {
        let mut ciphel = Ciphel::new();
        let tid = ciphel.start().expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }

        let (child_tid,err) = spawn();
        main();
        
        "##;

        ciphel
            .compile(tid, src)
            .expect("Compilation should have succeeded");

        ciphel.run().expect("no error should arise");
        let tids = ciphel.available_tids();
        assert!(tids.len() > 1);
        let child_tid = tids
            .into_iter()
            .find(|id| *id != tid)
            .expect("Child should have an id");

        let child_src = r##"
        
        fn add(x:u64,y:u64) -> u64 {
            return x+y;
        }
        let res = add(10,10);

        print(f"result = {res}");
            
        "##;

        ciphel
            .compile(child_tid, child_src)
            .expect("Compilation should have succeeded");

        ciphel.run().expect("no error should arise");

        let src = r##"
        
            let err = close(child_tid);
                
        "##;

        ciphel
            .compile(tid, src)
            .expect("Compilation should have succeeded");
        ciphel.run().expect("no error should arise");
        let tids = ciphel.available_tids();

        let src = r##"
            
            main();
                
        "##;

        ciphel
            .compile(tid, src)
            .expect("Compilation should have succeeded");
        ciphel.run().expect("no error should arise");
        let tids = ciphel.available_tids();
        assert!(tids.len() == 1);
    }

    #[test]
    fn valid_sleep() {
        let mut ciphel = Ciphel::new();
        let tid = ciphel.start().expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }
        sleep(5);
        main();
        
        "##;

        ciphel
            .compile(tid, src)
            .expect("Compilation should have succeeded");

        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
    }

    #[test]
    fn valid_wait_wake() {
        let mut ciphel = Ciphel::new();
        let tid = ciphel.start().expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }
        let (child_tid,err) = spawn();
        wait();
        main();
        
        "##;

        ciphel
            .compile(tid, src)
            .expect("Compilation should have succeeded");
        ciphel.run().expect("no error should arise");

        let tids = ciphel.available_tids();
        let child_tid = tids
            .into_iter()
            .find(|id| *id != tid)
            .expect("Child should have an id");

        let child_src = r##"
        
        sleep(3);
        wake(1);
        "##;

        ciphel
            .compile(child_tid, child_src)
            .expect("Compilation should have succeeded");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
    }

    #[test]
    fn valid_join() {
        let mut ciphel = Ciphel::new();
        let tid = ciphel.start().expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }
        let (child_tid,err) = spawn();
        join(child_tid);
        main();
        
        "##;

        ciphel
            .compile(tid, src)
            .expect("Compilation should have succeeded");
        ciphel.run().expect("no error should arise");

        let tids = ciphel.available_tids();
        let child_tid = tids
            .into_iter()
            .find(|id| *id != tid)
            .expect("Child should have an id");

        let child_src = r##"
        sleep(5);
        "##;

        ciphel
            .compile(child_tid, child_src)
            .expect("Compilation should have succeeded");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
    }

    #[test]
    fn valid_exit() {
        let mut ciphel = Ciphel::new();
        let tid = ciphel.start().expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }
        exit();
        main();
        
        "##;

        ciphel
            .compile(tid, src)
            .expect("Compilation should have succeeded");
        ciphel.run().expect("no error should arise");
        let tids = ciphel.available_tids();
        assert!(tids.len() == 0);
    }

    #[test]
    fn valid_join_exit() {
        let mut ciphel = Ciphel::new();
        let tid = ciphel.start().expect("starting should not fail");

        let src = r##"
        
        fn main() -> Unit {
            print("Hello World");
        }
        let (child_tid,err) = spawn();
        join(child_tid);
        main();
        
        "##;

        ciphel
            .compile(tid, src)
            .expect("Compilation should have succeeded");
        ciphel.run().expect("no error should arise");

        let tids = ciphel.available_tids();
        let child_tid = tids
            .into_iter()
            .find(|id| *id != tid)
            .expect("Child should have an id");

        let child_src = r##"
        sleep(3);
        exit();
        let x = 4+5;
        "##;

        ciphel
            .compile(child_tid, child_src)
            .expect("Compilation should have succeeded");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        ciphel.run().expect("no error should arise");
        let tids = ciphel.available_tids();
        assert!(tids.len() == 1);
    }
}
