use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Sub};

use nom::AsBytes;
use num_traits::{FromBytes, ToBytes, Zero};

use crate::{
    ast::expressions::data::data_typeof,
    semantic::{
        scope::{
            static_types::{NumberType, PrimitiveType, SliceType, StaticType},
            user_type_impl::UserType,
        },
        EitherType, SizeOf,
    },
    vm::{
        allocator::{Memory, MemoryAddress},
        vm::{Executable, RuntimeError},
    },
};

use super::math_operation::{
    comparaison_operator, comparaison_operator_float_left, comparaison_operator_float_right,
    math_operator, math_operator_float_left, math_operator_float_right, ComparaisonOperator,
    MathOperator,
};

#[derive(Debug, Clone)]
pub struct Operation {
    kind: OperationKind,
    result: OpPrimitive,
}

impl Executable for Operation {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        self.kind.execute(memory)
    }
}

#[derive(Debug, Clone)]
pub enum OperationKind {
    Mult(Mult),
    Div(Division),
    Mod(Mod),
    Addition(Addition),
    Substraction(Substraction),
    ShiftLeft(ShiftLeft),
    ShiftRight(ShiftRight),
    BitwiseAnd(BitwiseAnd),
    BitwiseXOR(BitwiseXOR),
    BitwiseOR(BitwiseOR),
    // Cast(Cast),
    Less(Less),
    LessEqual(LessEqual),
    Greater(Greater),
    GreaterEqual(GreaterEqual),
    Equal(Equal),
    NotEqual(NotEqual),
    Inclusion(Inclusion),
    LogicalAnd(LogicalAnd),
    LogicalOr(LogicalOr),
    Minus(Minus),
    Not(Not),
}

#[derive(Debug, Clone, Copy)]
pub enum OpPrimitive {
    Number(NumberType),
    Float,
    Bool,
    Char,
    String(usize),
}

impl OpPrimitive {
    pub fn get_float(memory: &Memory) -> Result<f64, RuntimeError> {
        let data = memory
            .stack
            .pop(PrimitiveType::Float.size_of())
            .map_err(|e| e.into())?;

        let data =
            TryInto::<&[u8; 8]>::try_into(data.as_slice()).map_err(|_| RuntimeError::Default)?;
        Ok(f64::from_le_bytes(*data))
    }

    pub fn get_num16<N: FromBytes<Bytes = [u8; 16]>>(memory: &Memory) -> Result<N, RuntimeError> {
        let data = memory.stack.pop(16).map_err(|e| e.into())?;
        let data =
            TryInto::<&[u8; 16]>::try_into(data.as_slice()).map_err(|_| RuntimeError::Default)?;
        Ok(N::from_le_bytes(data))
    }
    pub fn get_num8<N: FromBytes<Bytes = [u8; 8]>>(memory: &Memory) -> Result<N, RuntimeError> {
        let data = memory.stack.pop(8).map_err(|e| e.into())?;
        let data =
            TryInto::<&[u8; 8]>::try_into(data.as_slice()).map_err(|_| RuntimeError::Default)?;
        Ok(N::from_le_bytes(data))
    }
    pub fn get_num4<N: FromBytes<Bytes = [u8; 4]>>(memory: &Memory) -> Result<N, RuntimeError> {
        let data = memory.stack.pop(4).map_err(|e| e.into())?;
        let data =
            TryInto::<&[u8; 4]>::try_into(data.as_slice()).map_err(|_| RuntimeError::Default)?;
        Ok(N::from_le_bytes(data))
    }
    pub fn get_num2<N: FromBytes<Bytes = [u8; 2]>>(memory: &Memory) -> Result<N, RuntimeError> {
        let data = memory.stack.pop(2).map_err(|e| e.into())?;
        let data =
            TryInto::<&[u8; 2]>::try_into(data.as_slice()).map_err(|_| RuntimeError::Default)?;
        Ok(N::from_le_bytes(data))
    }
    pub fn get_num1<N: FromBytes<Bytes = [u8; 1]>>(memory: &Memory) -> Result<N, RuntimeError> {
        let data = memory.stack.pop(1).map_err(|e| e.into())?;
        let data =
            TryInto::<&[u8; 1]>::try_into(data.as_slice()).map_err(|_| RuntimeError::Default)?;
        Ok(N::from_le_bytes(data))
    }

    pub fn get_bool(memory: &Memory) -> Result<bool, RuntimeError> {
        let data = memory
            .stack
            .pop(PrimitiveType::Bool.size_of())
            .map_err(|e| e.into())?;

        Ok(data.first().map_or(false, |byte| *byte != 0))
    }
    pub fn get_char(memory: &Memory) -> Result<char, RuntimeError> {
        let data = memory
            .stack
            .pop(PrimitiveType::Char.size_of())
            .map_err(|e| e.into())?;
        let data = data.first().map(|byte| *byte as char);
        let Some(data) = data else {
            return Err(RuntimeError::Default);
        };
        Ok(data)
    }
    pub fn get_string(size: usize, memory: &Memory) -> Result<String, RuntimeError> {
        let data = memory.stack.pop(size).map_err(|e| e.into())?;
        let data = std::str::from_utf8(&data).map_err(|_| RuntimeError::Default)?;
        Ok(data.to_string())
    }
}

impl Executable for OperationKind {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match self {
            OperationKind::Mult(value) => value.execute(memory),
            OperationKind::Div(value) => value.execute(memory),
            OperationKind::Mod(value) => value.execute(memory),
            OperationKind::Addition(value) => value.execute(memory),
            OperationKind::Substraction(value) => value.execute(memory),
            OperationKind::ShiftLeft(value) => value.execute(memory),
            OperationKind::ShiftRight(value) => value.execute(memory),
            OperationKind::BitwiseAnd(value) => value.execute(memory),
            OperationKind::BitwiseXOR(value) => value.execute(memory),
            OperationKind::BitwiseOR(value) => value.execute(memory),
            // OperatKindion::Cast(value) => value.execute(memory),
            OperationKind::Less(value) => value.execute(memory),
            OperationKind::LessEqual(value) => value.execute(memory),
            OperationKind::Greater(value) => value.execute(memory),
            OperationKind::GreaterEqual(value) => value.execute(memory),
            OperationKind::Equal(value) => value.execute(memory),
            OperationKind::NotEqual(value) => value.execute(memory),
            OperationKind::Inclusion(value) => value.execute(memory),
            OperationKind::LogicalAnd(value) => value.execute(memory),
            OperationKind::LogicalOr(value) => value.execute(memory),
            OperationKind::Minus(value) => value.execute(memory),
            OperationKind::Not(value) => value.execute(memory),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mult {
    left: OpPrimitive,
    right: OpPrimitive,
}
#[derive(Debug, Clone)]
pub struct Division {
    left: OpPrimitive,
    right: OpPrimitive,
}
#[derive(Debug, Clone)]
pub struct Mod {
    left: OpPrimitive,
    right: OpPrimitive,
}

impl Executable for Mult {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Mult, memory)
            }
            (OpPrimitive::Number(left), OpPrimitive::Float) => {
                let right = OpPrimitive::get_float(memory)?;
                math_operator_float_left(&left, right, MathOperator::Mult, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Number(right)) => {
                let left = OpPrimitive::get_float(memory)?;
                math_operator_float_right(left, &right, MathOperator::Mult, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                memory
                    .stack
                    .push_with(&(left * right).to_le_bytes())
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for Division {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Div, memory)
            }
            (OpPrimitive::Number(left), OpPrimitive::Float) => {
                let right = OpPrimitive::get_float(memory)?;
                math_operator_float_left(&left, right, MathOperator::Div, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Number(right)) => {
                let left = OpPrimitive::get_float(memory)?;
                math_operator_float_right(left, &right, MathOperator::Div, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                if right.is_zero() {
                    return Err(RuntimeError::MathError);
                }
                memory
                    .stack
                    .push_with(&(left / right).to_le_bytes())
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for Mod {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Mod, memory)
            }
            (OpPrimitive::Number(left), OpPrimitive::Float) => {
                let right = OpPrimitive::get_float(memory)?;
                math_operator_float_left(&left, right, MathOperator::Mod, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Number(right)) => {
                let left = OpPrimitive::get_float(memory)?;
                math_operator_float_right(left, &right, MathOperator::Mod, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                if right.is_zero() {
                    return Err(RuntimeError::MathError);
                }
                memory
                    .stack
                    .push_with(&(left % right).to_le_bytes())
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Addition {
    left: OpPrimitive,
    right: OpPrimitive,
}

#[derive(Debug, Clone)]
pub struct Substraction {
    left: OpPrimitive,
    right: OpPrimitive,
}

impl Executable for Addition {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Add, memory)
            }
            (OpPrimitive::Number(left), OpPrimitive::Float) => {
                let right = OpPrimitive::get_float(memory)?;
                math_operator_float_left(&left, right, MathOperator::Add, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Number(right)) => {
                let left = OpPrimitive::get_float(memory)?;
                math_operator_float_right(left, &right, MathOperator::Add, memory)
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let left = OpPrimitive::get_string(left_size, memory)?;
                let right = OpPrimitive::get_string(right_size, memory)?;
                memory
                    .stack
                    .push_with(&(left + &right).as_bytes())
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                memory
                    .stack
                    .push_with(&(left + right).to_le_bytes())
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for Substraction {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Sub, memory)
            }
            (OpPrimitive::Number(left), OpPrimitive::Float) => {
                let right = OpPrimitive::get_float(memory)?;
                math_operator_float_left(&left, right, MathOperator::Sub, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Number(right)) => {
                let left = OpPrimitive::get_float(memory)?;
                math_operator_float_right(left, &right, MathOperator::Sub, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                memory
                    .stack
                    .push_with(&(left - right).to_le_bytes())
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShiftLeft {
    left: OpPrimitive,
    right: OpPrimitive,
}
#[derive(Debug, Clone)]
pub struct ShiftRight {
    left: OpPrimitive,
    right: OpPrimitive,
}

impl Executable for ShiftLeft {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::ShiftLeft, memory)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for ShiftRight {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::ShiftRight, memory)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BitwiseAnd {
    left: OpPrimitive,
    right: OpPrimitive,
}

impl Executable for BitwiseAnd {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::BitAnd, memory)
            }
            (OpPrimitive::Number(left), OpPrimitive::Float) => {
                let right = OpPrimitive::get_float(memory)?;
                math_operator_float_left(&left, right, MathOperator::BitAnd, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Number(right)) => {
                let left = OpPrimitive::get_float(memory)?;
                math_operator_float_right(left, &right, MathOperator::BitAnd, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                memory
                    .stack
                    .push_with(&(left - right).to_le_bytes())
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BitwiseXOR {
    left: OpPrimitive,
    right: OpPrimitive,
}

impl Executable for BitwiseXOR {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::BitXor, memory)
            }
            (OpPrimitive::Number(left), OpPrimitive::Float) => {
                let right = OpPrimitive::get_float(memory)?;
                math_operator_float_left(&left, right, MathOperator::BitXor, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Number(right)) => {
                let left = OpPrimitive::get_float(memory)?;
                math_operator_float_right(left, &right, MathOperator::BitXor, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                memory
                    .stack
                    .push_with(&(left - right).to_le_bytes())
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BitwiseOR {
    left: OpPrimitive,
    right: OpPrimitive,
}

impl Executable for BitwiseOR {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::BitOr, memory)
            }
            (OpPrimitive::Number(left), OpPrimitive::Float) => {
                let right = OpPrimitive::get_float(memory)?;
                math_operator_float_left(&left, right, MathOperator::BitOr, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Number(right)) => {
                let left = OpPrimitive::get_float(memory)?;
                math_operator_float_right(left, &right, MathOperator::BitOr, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                memory
                    .stack
                    .push_with(&(left - right).to_le_bytes())
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Less {
    left: OpPrimitive,
    right: OpPrimitive,
}
#[derive(Debug, Clone)]
pub struct LessEqual {
    left: OpPrimitive,
    right: OpPrimitive,
}
#[derive(Debug, Clone)]
pub struct Greater {
    left: OpPrimitive,
    right: OpPrimitive,
}
#[derive(Debug, Clone)]
pub struct GreaterEqual {
    left: OpPrimitive,
    right: OpPrimitive,
}

impl Executable for Less {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::Less, memory)
            }
            (OpPrimitive::Number(left), OpPrimitive::Float) => {
                let right = OpPrimitive::get_float(memory)?;
                comparaison_operator_float_left(&left, right, ComparaisonOperator::Less, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Number(right)) => {
                let left = OpPrimitive::get_float(memory)?;
                comparaison_operator_float_right(left, &right, ComparaisonOperator::Less, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let left = OpPrimitive::get_bool(memory)?;
                let right = OpPrimitive::get_bool(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let left = OpPrimitive::get_char(memory)?;
                let right = OpPrimitive::get_char(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let left = OpPrimitive::get_string(left_size, memory)?;
                let right = OpPrimitive::get_string(right_size, memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for LessEqual {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::LessEqual, memory)
            }
            (OpPrimitive::Number(left), OpPrimitive::Float) => {
                let right = OpPrimitive::get_float(memory)?;
                comparaison_operator_float_left(
                    &left,
                    right,
                    ComparaisonOperator::LessEqual,
                    memory,
                )
            }
            (OpPrimitive::Float, OpPrimitive::Number(right)) => {
                let left = OpPrimitive::get_float(memory)?;
                comparaison_operator_float_right(
                    left,
                    &right,
                    ComparaisonOperator::LessEqual,
                    memory,
                )
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let left = OpPrimitive::get_bool(memory)?;
                let right = OpPrimitive::get_bool(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let left = OpPrimitive::get_char(memory)?;
                let right = OpPrimitive::get_char(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let left = OpPrimitive::get_string(left_size, memory)?;
                let right = OpPrimitive::get_string(right_size, memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for Greater {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::Greater, memory)
            }
            (OpPrimitive::Number(left), OpPrimitive::Float) => {
                let right = OpPrimitive::get_float(memory)?;
                comparaison_operator_float_left(&left, right, ComparaisonOperator::Greater, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Number(right)) => {
                let left = OpPrimitive::get_float(memory)?;
                comparaison_operator_float_right(left, &right, ComparaisonOperator::Greater, memory)
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let left = OpPrimitive::get_bool(memory)?;
                let right = OpPrimitive::get_bool(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let left = OpPrimitive::get_char(memory)?;
                let right = OpPrimitive::get_char(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let left = OpPrimitive::get_string(left_size, memory)?;
                let right = OpPrimitive::get_string(right_size, memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for GreaterEqual {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::GreaterEqual, memory)
            }
            (OpPrimitive::Number(left), OpPrimitive::Float) => {
                let right = OpPrimitive::get_float(memory)?;
                comparaison_operator_float_left(
                    &left,
                    right,
                    ComparaisonOperator::GreaterEqual,
                    memory,
                )
            }
            (OpPrimitive::Float, OpPrimitive::Number(right)) => {
                let left = OpPrimitive::get_float(memory)?;
                comparaison_operator_float_right(
                    left,
                    &right,
                    ComparaisonOperator::GreaterEqual,
                    memory,
                )
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let left = OpPrimitive::get_bool(memory)?;
                let right = OpPrimitive::get_bool(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let left = OpPrimitive::get_char(memory)?;
                let right = OpPrimitive::get_char(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let left = OpPrimitive::get_string(left_size, memory)?;
                let right = OpPrimitive::get_string(right_size, memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Equal {
    left: usize,
    right: usize,
}
#[derive(Debug, Clone)]
pub struct NotEqual {
    left: usize,
    right: usize,
}

impl Executable for Equal {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let data = {
            let left_data = memory.stack.pop(self.left).map_err(|e| e.into())?;

            let right_data = memory.stack.pop(self.right).map_err(|e| e.into())?;

            [(left_data == right_data) as u8]
        };
        memory.stack.push_with(&data).map_err(|e| e.into())
    }
}

impl Executable for NotEqual {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let data = {
            let left_data = memory.stack.pop(self.left).map_err(|e| e.into())?;

            let right_data = memory.stack.pop(self.right).map_err(|e| e.into())?;

            [(left_data != right_data) as u8]
        };
        memory.stack.push_with(&data).map_err(|e| e.into())
    }
}
#[derive(Debug, Clone)]
pub struct Inclusion {
    left: usize,
    iterator_addr: MemoryAddress,
    item_size: usize,
}

impl Executable for Inclusion {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let left_data = memory.stack.pop(self.left).map_err(|e| e.into())?;

        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct LogicalAnd();

impl Executable for LogicalAnd {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let left_data = OpPrimitive::get_bool(memory)?;
        let right_data = OpPrimitive::get_bool(memory)?;
        let data = [(left_data && right_data) as u8];
        memory.stack.push_with(&data).map_err(|e| e.into())
    }
}

#[derive(Debug, Clone)]
pub struct LogicalOr();

impl Executable for LogicalOr {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let left_data = OpPrimitive::get_bool(memory)?;
        let right_data = OpPrimitive::get_bool(memory)?;
        let data = [(left_data || right_data) as u8];
        memory.stack.push_with(&data).map_err(|e| e.into())
    }
}

#[derive(Debug, Clone)]
pub struct Minus {
    data_type: OpPrimitive,
}

impl Executable for Minus {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match &self.data_type {
            OpPrimitive::Float => {
                let data = OpPrimitive::get_float(memory)?;
                memory
                    .stack
                    .push_with(&data.to_le_bytes())
                    .map_err(|e| e.into())
            }
            OpPrimitive::Number(number) => match number {
                NumberType::U8 => {
                    let data = OpPrimitive::get_num1::<u8>(memory)? as i16;
                    memory
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let data = OpPrimitive::get_num2::<u16>(memory)? as i32;
                    memory
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let data = OpPrimitive::get_num4::<u32>(memory)? as i64;
                    memory
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let data = OpPrimitive::get_num8::<u64>(memory)? as i128;
                    memory
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let data = OpPrimitive::get_num16::<u128>(memory)? as i128;
                    memory
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let data = OpPrimitive::get_num1::<i8>(memory)?;
                    memory
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let data = OpPrimitive::get_num2::<i16>(memory)?;
                    memory
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let data = OpPrimitive::get_num4::<i32>(memory)?;
                    memory
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let data = OpPrimitive::get_num8::<i64>(memory)?;
                    memory
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let data = OpPrimitive::get_num16::<i128>(memory)?;
                    memory
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
            },
            OpPrimitive::Char => Err(RuntimeError::UnsupportedOperation),
            OpPrimitive::Bool => Err(RuntimeError::UnsupportedOperation),
            OpPrimitive::String(_) => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Not();

impl Executable for Not {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let value_data = OpPrimitive::get_bool(memory)?;

        let data = vec![(!value_data) as u8];
        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_float(num: f64, memory: &Memory) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_num1<T: num_traits::ToBytes<Bytes = [u8; 1]>>(
        num: T,
        memory: &Memory,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_num2<T: num_traits::ToBytes<Bytes = [u8; 2]>>(
        num: T,
        memory: &Memory,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_num4<T: num_traits::ToBytes<Bytes = [u8; 4]>>(
        num: T,
        memory: &Memory,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_num8<T: num_traits::ToBytes<Bytes = [u8; 8]>>(
        num: T,
        memory: &Memory,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_num16<T: num_traits::ToBytes<Bytes = [u8; 16]>>(
        num: T,
        memory: &Memory,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }
    fn init_char(memory: &Memory) -> Result<(), RuntimeError> {
        let data = vec!['a' as u8];
        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_bool(state: bool, memory: &Memory) -> Result<(), RuntimeError> {
        let data = vec![state as u8];
        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_string(text: &str, memory: &Memory) -> Result<(), RuntimeError> {
        let data = text.as_bytes().to_vec();
        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }

    fn to_number(data: Vec<u8>) -> Result<i64, ()> {
        if data.len() != 8 {
            return Err(());
        }
        let data = TryInto::<&[u8; 8]>::try_into(data.as_bytes()).map_err(|_| ())?;
        Ok(i64::from_le_bytes(*data))
    }
    fn to_float(data: Vec<u8>) -> Result<f64, ()> {
        if data.len() != 8 {
            return Err(());
        }
        let data = TryInto::<&[u8; 8]>::try_into(data.as_bytes()).map_err(|_| ())?;
        Ok(f64::from_le_bytes(*data))
    }

    #[test]
    fn valid_product() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(20u32, &memory).expect("init should have succeeded");
        Mult {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(&memory).expect("result should be of valid type");
        assert_eq!(10 * 20, res);

        let memory = Memory::new();
        init_float(10., &memory).expect("init should have succeeded");
        init_float(20., &memory).expect("init should have succeeded");
        Mult {
            left: OpPrimitive::Float,
            right: OpPrimitive::Float,
        }
        .execute(&memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_float(&memory).expect("result should be of valid type");
        assert_eq!(10. * 20., res);
    }
}
