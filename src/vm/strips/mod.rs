use super::{
    allocator::Memory,
    vm::{self, Executable},
};

pub mod access;
pub mod assign;
pub mod call;
pub mod declare;
pub mod if_strip;
pub mod loop_strip;
pub mod match_strip;
pub mod operation;
pub mod scope_strip;
pub mod serialize;
pub mod try_strip;

#[derive(Debug, Clone)]
pub enum Strip {
    Operation(operation::Operation),
    Call(call::Call),
    Serialize(serialize::Serialize),
    Access(access::Access),
    If(if_strip::IfSrip),
    Match(match_strip::MatchStrip),
    Try(try_strip::TryStrip),
    Assign(assign::Assign),
    Declare(declare::Declare),
    Loop(loop_strip::LoopStrip),
    Scope(scope_strip::ScopeStrip),
}

impl Executable for Strip {
    fn execute(&self, memory: &Memory) -> Result<(), vm::RuntimeError> {
        todo!()
    }
}
