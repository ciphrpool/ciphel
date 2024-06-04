use crate::{
    semantic::ArcMutex,
    vm::{casm::Casm, vm::RuntimeError},
};

#[derive(Debug, Clone, Copy)]
pub enum VTableError {
    AllocError,
    FreeError,
    AccessError,
    Default,
}

#[derive(Debug, Clone)]
pub struct VTable {
    functions: ArcMutex<Vec<Option<Vec<Casm>>>>,
}
impl Into<RuntimeError> for VTableError {
    fn into(self) -> RuntimeError {
        RuntimeError::VTableError(self)
    }
}

impl Default for VTable {
    fn default() -> Self {
        Self {
            functions: Default::default(),
        }
    }
}

impl VTable {
    pub fn alloc(&self, function: Vec<Casm>) -> Result<u64, VTableError> {
        let mut borrowed = self.functions.as_ref().borrow_mut();
        let idx = borrowed.len();
        borrowed.push(Some(function));
        Ok(idx as u64)
    }

    pub fn read(&self, offset: u64) -> Result<Vec<Casm>, VTableError> {
        let borrowed: std::cell::Ref<'_, Vec<Option<Vec<Casm>>>> = self.functions.as_ref().borrow();
        match (&borrowed).get(offset as usize) {
            Some(Some(f)) => Ok(f.clone()),
            _ => Err(VTableError::AccessError),
        }
    }

    pub fn free(&self, offset: u64) -> Result<(), VTableError> {
        let mut borrowed = self.functions.as_ref().borrow_mut();
        match borrowed.get_mut(offset as usize) {
            Some(function) => {
                function.as_mut().map(|f| f.clear());
                *function = None;
                Ok(())
            }
            None => Err(VTableError::FreeError),
        }
    }
}
