use crate::vm::{
    allocator::Memory,
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub enum Operation {
    HighOrdMath(HighOrdMath),
    LowOrdMath(LowOrdMath),
    Shift(Shift),
    BitwiseAnd(BitwiseAnd),
    BitwiseXOR(BitwiseXOR),
    BitwiseOR(BitwiseOR),
    Cast(Cast),
    Comparaison(Comparaison),
    Equation(Equation),
    Inclusion(Inclusion),
    LogicalAnd(LogicalAnd),
    LogicalOr(LogicalOr),
    Minus(Minus),
    Not(Not),
}

impl Executable for Operation {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match self {
            Operation::HighOrdMath(value) => value.execute(memory),
            Operation::LowOrdMath(value) => value.execute(memory),
            Operation::Shift(value) => value.execute(memory),
            Operation::BitwiseAnd(value) => value.execute(memory),
            Operation::BitwiseXOR(value) => value.execute(memory),
            Operation::BitwiseOR(value) => value.execute(memory),
            Operation::Cast(value) => value.execute(memory),
            Operation::Comparaison(value) => value.execute(memory),
            Operation::Equation(value) => value.execute(memory),
            Operation::Inclusion(value) => value.execute(memory),
            Operation::LogicalAnd(value) => value.execute(memory),
            Operation::LogicalOr(value) => value.execute(memory),
            Operation::Minus(value) => value.execute(memory),
            Operation::Not(value) => value.execute(memory),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HighOrdMath {}

impl Executable for HighOrdMath {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum LowOrdMath {}

impl Executable for LowOrdMath {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Shift {}

impl Executable for Shift {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum BitwiseAnd {}

impl Executable for BitwiseAnd {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum BitwiseXOR {}

impl Executable for BitwiseXOR {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum BitwiseOR {}

impl Executable for BitwiseOR {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Cast {}

impl Executable for Cast {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Comparaison {}

impl Executable for Comparaison {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Equation {}

impl Executable for Equation {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Inclusion {}

impl Executable for Inclusion {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum LogicalAnd {}

impl Executable for LogicalAnd {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum LogicalOr {}

impl Executable for LogicalOr {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Minus {}

impl Executable for Minus {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Not {}

impl Executable for Not {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}
