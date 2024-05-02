use std::cell::Ref;

use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::static_types::{NumberType, PrimitiveType, StaticType, TupleType};
use crate::semantic::{Either, TypeOf};
use crate::vm::allocator::stack::Stack;
use crate::vm::casm::operation::OpPrimitive;
use crate::vm::casm::Casm;
use crate::vm::platform::utils::lexem;
use crate::vm::platform::LibCasm;
use crate::vm::vm::{CasmMetadata, Executable, RuntimeError, Signal, Tid, MAX_THREAD_COUNT};
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
    Wait,
    Wake,
    Sleep,
    Join,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThreadCasm {
    Spawn,
    Close,
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
            _ => None,
        }
    }
}
fn expect_one_u64(params: &Vec<Expression>, scope: &MutRc<Scope>) -> Result<(), SemanticError> {
    if params.len() != 1 {
        return Err(SemanticError::IncorrectArguments);
    }

    let size = &params[0];

    let _ = size.resolve(scope, &Some(p_num!(U64)), &())?;
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
            ThreadFn::Close => expect_one_u64(extra, scope),
            ThreadFn::Wait => expect_one_u64(extra, scope),
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
            ThreadFn::Sleep => Ok(e_static!(StaticType::Error)),
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
            ThreadCasm::Close => {
                let tid = OpPrimitive::get_num8::<u64>(stack)?;
                Err(RuntimeError::Signal(Signal::CLOSE(tid as usize)))
            }
            ThreadCasm::Wait => {
                let tid = OpPrimitive::get_num8::<u64>(stack)?;
                Err(RuntimeError::Signal(Signal::WAIT(tid as usize)))
            }
            ThreadCasm::Wake => {
                let tid = OpPrimitive::get_num8::<u64>(stack)?;
                Err(RuntimeError::Signal(Signal::WAKE(tid as usize)))
            }
            ThreadCasm::Sleep => {
                let tid = OpPrimitive::get_num8::<u64>(stack)?;
                Err(RuntimeError::Signal(Signal::SLEEP(tid as usize)))
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
        assert!(tids.len() == 1);

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
}
