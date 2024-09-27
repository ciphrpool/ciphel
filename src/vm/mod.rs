use thiserror::Error;
use ulid::Ulid;

pub mod allocator;
pub mod asm;
pub mod core;
pub mod error_handler;
pub mod external;
pub mod program;
pub mod runtime;
pub mod scheduler;
pub mod scheduler_v2;
pub mod stdio;
pub mod vm;

#[derive(Debug, Clone)]
pub struct CodeGenerationContext {
    pub return_label: Option<Ulid>,
    pub break_label: Option<Ulid>,
    pub continue_label: Option<Ulid>,
}

impl Default for CodeGenerationContext {
    fn default() -> Self {
        Self {
            return_label: Default::default(),
            break_label: Default::default(),
            continue_label: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum CodeGenerationError {
    #[error("Unresolved Error")]
    UnresolvedError,
    #[error("unlocatale expression")]
    Unlocatable,
    #[error("unaccessible expression")]
    Unaccessible,
    #[error("unexpected error")]
    Default,
}

pub trait GenerateCode {
    fn gencode<E: external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Weight {
    #[default]
    ZERO,
    MAX,
    CUSTOM(usize),

    LOW,
    MEDIUM,
    HIGH,
    EXTREME,
}

impl Weight {
    pub fn mult(&self, c: usize) -> usize {
        match self {
            Weight::ZERO => todo!(),
            Weight::MAX => todo!(),
            Weight::CUSTOM(_) => todo!(),
            Weight::LOW => todo!(),
            Weight::MEDIUM => todo!(),
            Weight::HIGH => todo!(),
            Weight::EXTREME => todo!(),
        }
    }

    pub fn get(&self) -> usize {
        match self {
            Weight::ZERO => 0,
            Weight::MAX => todo!(), //super::scheduler::INSTRUCTION_MAX_COUNT,
            Weight::CUSTOM(w) => *w,
            Weight::LOW => 1,
            Weight::MEDIUM => 2,
            Weight::HIGH => 4,
            Weight::EXTREME => 8,
        }
    }
}

pub trait AsmName<E: crate::vm::external::Engine> {
    fn name(&self, stdio: &mut stdio::StdIO, program: &program::Program<E>, engine: &mut E);
}

pub trait AsmWeight {
    fn weight(&self) -> Weight {
        Weight::LOW
    }
}
