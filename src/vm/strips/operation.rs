use nom::AsBytes;
use num_traits::ToBytes;

use crate::{
    ast::expressions::data::data_typeof,
    semantic::{
        scope::{
            static_types::{PrimitiveType, SliceType, StaticType},
            user_type_impl::UserType,
        },
        EitherType, SizeOf,
    },
    vm::{
        allocator::{Memory, MemoryAddress},
        vm::{Executable, RuntimeError},
    },
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
    Div(Div),
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
    Number,
    Float,
    Bool,
    Char,
    String(usize),
}

impl OpPrimitive {
    fn get_float(memory: &Memory) -> Result<f64, RuntimeError> {
        let data = memory
            .stack
            .pop(PrimitiveType::Float.size_of())
            .map_err(|e| e.into())?;

        let data =
            TryInto::<&[u8; 8]>::try_into(data.as_bytes()).map_err(|_| RuntimeError::Default)?;
        Ok(f64::from_le_bytes(*data))
    }
    fn get_number(memory: &Memory) -> Result<i64, RuntimeError> {
        let data = memory
            .stack
            .pop(PrimitiveType::Float.size_of())
            .map_err(|e| e.into())?;

        let data =
            TryInto::<&[u8; 8]>::try_into(data.as_bytes()).map_err(|_| RuntimeError::Default)?;
        Ok(i64::from_le_bytes(*data))
    }
    fn get_bool(memory: &Memory) -> Result<bool, RuntimeError> {
        let data = memory
            .stack
            .pop(PrimitiveType::Bool.size_of())
            .map_err(|e| e.into())?;

        Ok(data.first().map_or(false, |byte| *byte != 0))
    }
    fn get_char(memory: &Memory) -> Result<char, RuntimeError> {
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
    fn get_string(size: usize, memory: &Memory) -> Result<String, RuntimeError> {
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
pub struct Div {
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
        let data = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                Some((left * right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Number, OpPrimitive::Float) => {
                let left = OpPrimitive::get_number(memory)? as f64;
                let right = OpPrimitive::get_float(memory)?;
                Some((left * right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Float, OpPrimitive::Number) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_number(memory)? as f64;
                Some((left * right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                Some((left * right).to_le_bytes().to_vec())
            }
            _ => None,
        };
        match data {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
        }
    }
}

impl Executable for Div {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let data = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                if right == 0 {
                    return Err(RuntimeError::MathError);
                }
                Some((left / right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Number, OpPrimitive::Float) => {
                let left = OpPrimitive::get_number(memory)? as f64;
                let right = OpPrimitive::get_float(memory)?;
                if right == 0. {
                    return Err(RuntimeError::MathError);
                }
                Some((left / right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Float, OpPrimitive::Number) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_number(memory)? as f64;
                if right == 0. {
                    return Err(RuntimeError::MathError);
                }
                Some((left / right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                if right == 0. {
                    return Err(RuntimeError::MathError);
                }
                Some((left / right).to_le_bytes().to_vec())
            }
            _ => None,
        };
        match data {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
        }
    }
}

impl Executable for Mod {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let data = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                if right == 0 {
                    return Err(RuntimeError::MathError);
                }
                Some((left % right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Number, OpPrimitive::Float) => {
                let left = OpPrimitive::get_number(memory)? as f64;
                let right = OpPrimitive::get_float(memory)?;
                if right == 0. {
                    return Err(RuntimeError::MathError);
                }
                Some((left % right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Float, OpPrimitive::Number) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_number(memory)? as f64;
                if right == 0. {
                    return Err(RuntimeError::MathError);
                }
                Some((left % right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                if right == 0. {
                    return Err(RuntimeError::MathError);
                }
                Some((left % right).to_le_bytes().to_vec())
            }
            _ => None,
        };
        match data {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
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
        let data = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                Some((left + right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Number, OpPrimitive::Float) => {
                let left = OpPrimitive::get_number(memory)? as f64;
                let right = OpPrimitive::get_float(memory)?;
                Some((left + right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Float, OpPrimitive::Number) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_number(memory)? as f64;
                Some((left + right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                Some((left + right).to_le_bytes().to_vec())
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let left = OpPrimitive::get_string(left_size, memory)?;
                let right = OpPrimitive::get_string(right_size, memory)?;
                Some((left + &right).as_bytes().to_vec())
            }
            _ => None,
        };
        match data {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
        }
    }
}

impl Executable for Substraction {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let result = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                Some((left - right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Number, OpPrimitive::Float) => {
                let left = OpPrimitive::get_number(memory)? as f64;
                let right = OpPrimitive::get_float(memory)?;
                Some((left - right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Float, OpPrimitive::Number) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_number(memory)? as f64;
                Some((left - right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                Some((left - right).to_le_bytes().to_vec())
            }
            _ => None,
        };

        match result {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
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
        let result = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                Some((left << right).to_le_bytes().to_vec())
            }
            _ => None,
        };

        match result {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
        }
    }
}

impl Executable for ShiftRight {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let result = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                Some((left >> right).to_le_bytes().to_vec())
            }
            _ => None,
        };

        match result {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
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
        let result = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                Some((left & right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let left = OpPrimitive::get_bool(memory)?;
                let right = OpPrimitive::get_bool(memory)?;
                Some(vec![(left & right) as u8])
            }
            _ => None,
        };

        match result {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
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
        let result = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                Some((left ^ right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let left = OpPrimitive::get_bool(memory)?;
                let right = OpPrimitive::get_bool(memory)?;
                Some(vec![(left ^ right) as u8])
            }
            _ => None,
        };

        match result {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
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
        let result = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                Some((left | right).to_le_bytes().to_vec())
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let left = OpPrimitive::get_bool(memory)?;
                let right = OpPrimitive::get_bool(memory)?;
                Some(vec![(left | right) as u8])
            }
            _ => None,
        };

        match result {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
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
        let result = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                Some(vec![(left < right) as u8])
            }
            (OpPrimitive::Number, OpPrimitive::Float) => {
                let left = OpPrimitive::get_number(memory)? as f64;
                let right = OpPrimitive::get_float(memory)?;
                Some(vec![(left < right) as u8])
            }
            (OpPrimitive::Float, OpPrimitive::Number) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_number(memory)? as f64;
                Some(vec![(left < right) as u8])
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                Some(vec![(left < right) as u8])
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let left = OpPrimitive::get_bool(memory)?;
                let right = OpPrimitive::get_bool(memory)?;
                Some(vec![(left < right) as u8])
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let left = OpPrimitive::get_char(memory)?;
                let right = OpPrimitive::get_char(memory)?;
                Some(vec![(left < right) as u8])
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let left = OpPrimitive::get_string(left_size, memory)?;
                let right = OpPrimitive::get_string(right_size, memory)?;
                Some(vec![(left < right) as u8])
            }
            _ => None,
        };

        match result {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
        }
    }
}

impl Executable for LessEqual {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let result = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                Some(vec![(left <= right) as u8])
            }
            (OpPrimitive::Number, OpPrimitive::Float) => {
                let left = OpPrimitive::get_number(memory)? as f64;
                let right = OpPrimitive::get_float(memory)?;
                Some(vec![(left <= right) as u8])
            }
            (OpPrimitive::Float, OpPrimitive::Number) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_number(memory)? as f64;
                Some(vec![(left <= right) as u8])
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                Some(vec![(left <= right) as u8])
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let left = OpPrimitive::get_bool(memory)?;
                let right = OpPrimitive::get_bool(memory)?;
                Some(vec![(left <= right) as u8])
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let left = OpPrimitive::get_char(memory)?;
                let right = OpPrimitive::get_char(memory)?;
                Some(vec![(left <= right) as u8])
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let left = OpPrimitive::get_string(left_size, memory)?;
                let right = OpPrimitive::get_string(right_size, memory)?;
                Some(vec![(left <= right) as u8])
            }
            _ => None,
        };

        match result {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
        }
    }
}

impl Executable for Greater {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let result = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                Some(vec![(left > right) as u8])
            }
            (OpPrimitive::Number, OpPrimitive::Float) => {
                let left = OpPrimitive::get_number(memory)? as f64;
                let right = OpPrimitive::get_float(memory)?;
                Some(vec![(left > right) as u8])
            }
            (OpPrimitive::Float, OpPrimitive::Number) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_number(memory)? as f64;
                Some(vec![(left > right) as u8])
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                Some(vec![(left > right) as u8])
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let left = OpPrimitive::get_bool(memory)?;
                let right = OpPrimitive::get_bool(memory)?;
                Some(vec![(left > right) as u8])
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let left = OpPrimitive::get_char(memory)?;
                let right = OpPrimitive::get_char(memory)?;
                Some(vec![(left > right) as u8])
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let left = OpPrimitive::get_string(left_size, memory)?;
                let right = OpPrimitive::get_string(right_size, memory)?;
                Some(vec![(left > right) as u8])
            }
            _ => None,
        };

        match result {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
        }
    }
}

impl Executable for GreaterEqual {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let result = match (self.left, self.right) {
            (OpPrimitive::Number, OpPrimitive::Number) => {
                let left = OpPrimitive::get_number(memory)?;
                let right = OpPrimitive::get_number(memory)?;
                Some(vec![(left >= right) as u8])
            }
            (OpPrimitive::Number, OpPrimitive::Float) => {
                let left = OpPrimitive::get_number(memory)? as f64;
                let right = OpPrimitive::get_float(memory)?;
                Some(vec![(left >= right) as u8])
            }
            (OpPrimitive::Float, OpPrimitive::Number) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_number(memory)? as f64;
                Some(vec![(left >= right) as u8])
            }
            (OpPrimitive::Float, OpPrimitive::Float) => {
                let left = OpPrimitive::get_float(memory)?;
                let right = OpPrimitive::get_float(memory)?;
                Some(vec![(left >= right) as u8])
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let left = OpPrimitive::get_bool(memory)?;
                let right = OpPrimitive::get_bool(memory)?;
                Some(vec![(left >= right) as u8])
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let left = OpPrimitive::get_char(memory)?;
                let right = OpPrimitive::get_char(memory)?;
                Some(vec![(left >= right) as u8])
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let left = OpPrimitive::get_string(left_size, memory)?;
                let right = OpPrimitive::get_string(right_size, memory)?;
                Some(vec![(left >= right) as u8])
            }
            _ => None,
        };

        match result {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
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

            vec![(left_data == right_data) as u8]
        };

        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }
}

impl Executable for NotEqual {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let data = {
            let left_data = memory.stack.pop(self.left).map_err(|e| e.into())?;

            let right_data = memory.stack.pop(self.right).map_err(|e| e.into())?;

            vec![(left_data != right_data) as u8]
        };

        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
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

        let data = vec![(left_data && right_data) as u8];
        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LogicalOr();

impl Executable for LogicalOr {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let left_data = OpPrimitive::get_bool(memory)?;

        let right_data = OpPrimitive::get_bool(memory)?;

        let data = vec![(left_data || right_data) as u8];
        let offset = memory.stack.top();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Minus {
    data_type: OpPrimitive,
}

impl Executable for Minus {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let result = match &self.data_type {
            OpPrimitive::Float => {
                let data = OpPrimitive::get_float(memory)?;
                Some((-data).to_le_bytes())
            }
            OpPrimitive::Number => {
                let data = OpPrimitive::get_number(memory)?;
                Some((-data).to_le_bytes())
            }
            OpPrimitive::Char => None,
            OpPrimitive::Bool => None,
            OpPrimitive::String(_) => None,
        };

        match result {
            Some(data) => {
                let offset = memory.stack.top();
                let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
                let _ = memory
                    .stack
                    .write(offset, &data.to_vec())
                    .map_err(|e| e.into())?;
                Ok(())
            }
            None => Err(RuntimeError::Default),
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

    fn init_number(num: i64, memory: &Memory) -> Result<(), RuntimeError> {
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
        init_number(10, &memory).expect("init should have succeeded");
        init_number(20, &memory).expect("init should have succeeded");
        Mult {
            left: OpPrimitive::Number,
            right: OpPrimitive::Number,
        }
        .execute(&memory)
        .expect("execution should have succeeded");

        let data = memory
            .stack
            .read_last(8)
            .expect("read should have succeeded");
        let res = to_number(data).expect("result should be of valid type");
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

        let data = memory
            .stack
            .read_last(8)
            .expect("read should have succeeded");
        let res = to_float(data).expect("result should be of valid type");
        assert_eq!(10. * 20., res);
    }
}
