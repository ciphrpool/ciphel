use std::usize;

use ulid::Ulid;

use crate::semantic::scope::static_types::StaticType;
use crate::semantic::{EType, SemanticError};
use crate::{ast::utils::strings::ID, semantic::scope::scope::ScopeManager};
use crate::{e_static, semantic};

use super::program::Program;
use super::{
    allocator::{
        heap::{Heap, HeapError},
        stack::{Stack, StackError},
    },
    stdio::StdIO,
};
use thiserror::Error;

pub type Tid = usize;

#[derive(Debug, Clone)]
pub enum Signal {
    SPAWN,
    EXIT,
    CLOSE(Tid),
    WAIT,
    WAIT_STDIN,
    WAKE(Tid),
    SLEEP(usize),
    JOIN(Tid),
}
