use super::CasmProgram;
use crate::{
    semantic::{
        scope::static_types::{st_deserialize::extract_u64, NumberType, PrimitiveType, StaticType},
        EType, Either, SizeOf,
    },
    vm::{
        allocator::{Memory, MemoryAddress},
        vm::{CodeGenerationError, Executable, RuntimeError},
    },
};
use nom::AsBytes;
use num_traits::{FromBytes, ToBytes, Zero};
use std::cell::Cell;

use super::math_operation::{
    comparaison_operator, math_operator, ComparaisonOperator, MathOperator,
};

#[derive(Debug, Clone)]
pub struct Operation {
    pub kind: OperationKind,
    // pub result: OpPrimitive,
}

impl Executable for Operation {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let _ = self.kind.execute(program, memory)?;
        program.cursor.set(program.cursor.get() + 1);
        Ok(())
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
    Cast(Cast),
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
    Bool,
    Char,
    String,
}

impl TryInto<OpPrimitive> for EType {
    type Error = CodeGenerationError;

    fn try_into(self) -> Result<OpPrimitive, Self::Error> {
        match self {
            Either::Static(value) => match value.as_ref() {
                StaticType::Primitive(value) => match value {
                    PrimitiveType::Number(value) => Ok(OpPrimitive::Number(*value)),
                    PrimitiveType::Char => Ok(OpPrimitive::Char),
                    PrimitiveType::Bool => Ok(OpPrimitive::Bool),
                },
                StaticType::String(_) => Ok(OpPrimitive::String),
                _ => Err(CodeGenerationError::UnresolvedError),
            },
            Either::User(_) => Err(CodeGenerationError::UnresolvedError),
        }
    }
}

impl OpPrimitive {
    // pub fn get_float(memory: &Memory) -> Result<f64, RuntimeError> {
    //     let data = memory
    //         .stack
    //         .pop(PrimitiveType::Float.size_of())
    //         .map_err(|e| e.into())?;

    //     let data = TryInto::<&[u8; 8]>::try_into(data.as_slice())
    //         .map_err(|_| RuntimeError::Deserialization)?;
    //     Ok(f64::from_le_bytes(*data))
    // }

    pub fn get_num16<N: FromBytes<Bytes = [u8; 16]>>(memory: &Memory) -> Result<N, RuntimeError> {
        let data = memory.stack.pop(16).map_err(|e| e.into())?;
        let data = TryInto::<&[u8; 16]>::try_into(data.as_slice())
            .map_err(|_| RuntimeError::Deserialization)?;
        Ok(N::from_le_bytes(data))
    }
    pub fn get_num8<N: FromBytes<Bytes = [u8; 8]>>(memory: &Memory) -> Result<N, RuntimeError> {
        let data = memory.stack.pop(8).map_err(|e| e.into())?;
        let data = TryInto::<&[u8; 8]>::try_into(data.as_slice())
            .map_err(|_| RuntimeError::Deserialization)?;
        Ok(N::from_le_bytes(data))
    }
    pub fn get_num4<N: FromBytes<Bytes = [u8; 4]>>(memory: &Memory) -> Result<N, RuntimeError> {
        let data = memory.stack.pop(4).map_err(|e| e.into())?;
        let data = TryInto::<&[u8; 4]>::try_into(data.as_slice())
            .map_err(|_| RuntimeError::Deserialization)?;
        Ok(N::from_le_bytes(data))
    }
    pub fn get_num2<N: FromBytes<Bytes = [u8; 2]>>(memory: &Memory) -> Result<N, RuntimeError> {
        let data = memory.stack.pop(2).map_err(|e| e.into())?;
        let data = TryInto::<&[u8; 2]>::try_into(data.as_slice())
            .map_err(|_| RuntimeError::Deserialization)?;
        Ok(N::from_le_bytes(data))
    }
    pub fn get_num1<N: FromBytes<Bytes = [u8; 1]>>(memory: &Memory) -> Result<N, RuntimeError> {
        let data = memory.stack.pop(1).map_err(|e| e.into())?;
        let data = TryInto::<&[u8; 1]>::try_into(data.as_slice())
            .map_err(|_| RuntimeError::Deserialization)?;
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
            return Err(RuntimeError::Deserialization);
        };
        Ok(data)
    }
    pub fn get_string(memory: &Memory) -> Result<String, RuntimeError> {
        let heap_address = OpPrimitive::get_num8::<u64>(memory)?;
        let data = memory
            .heap
            .read(heap_address as usize, 16)
            .expect("Heap Read should have succeeded");
        let (length, rest) = extract_u64(&data)?;
        let (capacity, rest) = extract_u64(rest)?;
        let data = memory
            .heap
            .read(heap_address as usize + 16, length as usize)
            .expect("Heap Read should have succeeded");
        let data = std::str::from_utf8(&data).map_err(|_| RuntimeError::Deserialization)?;
        Ok(data.to_string())
    }
}

impl Executable for OperationKind {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match self {
            OperationKind::Mult(value) => value.execute(program, memory),
            OperationKind::Div(value) => value.execute(program, memory),
            OperationKind::Mod(value) => value.execute(program, memory),
            OperationKind::Addition(value) => value.execute(program, memory),
            OperationKind::Substraction(value) => value.execute(program, memory),
            OperationKind::ShiftLeft(value) => value.execute(program, memory),
            OperationKind::ShiftRight(value) => value.execute(program, memory),
            OperationKind::BitwiseAnd(value) => value.execute(program, memory),
            OperationKind::BitwiseXOR(value) => value.execute(program, memory),
            OperationKind::BitwiseOR(value) => value.execute(program, memory),
            OperationKind::Cast(value) => value.execute(program, memory),
            OperationKind::Less(value) => value.execute(program, memory),
            OperationKind::LessEqual(value) => value.execute(program, memory),
            OperationKind::Greater(value) => value.execute(program, memory),
            OperationKind::GreaterEqual(value) => value.execute(program, memory),
            OperationKind::Equal(value) => value.execute(program, memory),
            OperationKind::NotEqual(value) => value.execute(program, memory),
            OperationKind::Inclusion(value) => value.execute(program, memory),
            OperationKind::LogicalAnd(value) => value.execute(program, memory),
            OperationKind::LogicalOr(value) => value.execute(program, memory),
            OperationKind::Minus(value) => value.execute(program, memory),
            OperationKind::Not(value) => value.execute(program, memory),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mult {
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}
#[derive(Debug, Clone)]
pub struct Division {
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}
#[derive(Debug, Clone)]
pub struct Mod {
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}

impl Executable for Mult {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Mult, memory)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for Division {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Div, memory)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for Mod {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Mod, memory)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Addition {
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}

#[derive(Debug, Clone)]
pub struct Substraction {
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}

impl Executable for Addition {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Add, memory)
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right = OpPrimitive::get_string(memory)?;
                let left = OpPrimitive::get_string(memory)?;
                memory
                    .stack
                    .push_with(&(left + &right).as_bytes())
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for Substraction {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Sub, memory)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShiftLeft {
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}
#[derive(Debug, Clone)]
pub struct ShiftRight {
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}

impl Executable for ShiftLeft {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::ShiftLeft, memory)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for ShiftRight {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
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
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}

impl Executable for BitwiseAnd {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::BitAnd, memory)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BitwiseXOR {
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}

impl Executable for BitwiseXOR {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::BitXor, memory)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BitwiseOR {
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}

impl Executable for BitwiseOR {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::BitOr, memory)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Less {
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}
#[derive(Debug, Clone)]
pub struct LessEqual {
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}
#[derive(Debug, Clone)]
pub struct Greater {
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}
#[derive(Debug, Clone)]
pub struct GreaterEqual {
    pub left: OpPrimitive,
    pub right: OpPrimitive,
}

impl Executable for Less {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::Less, memory)
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::get_bool(memory)?;
                let left = OpPrimitive::get_bool(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::get_char(memory)?;
                let left = OpPrimitive::get_char(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right = OpPrimitive::get_string(memory)?;
                let left = OpPrimitive::get_string(memory)?;
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
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::LessEqual, memory)
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::get_bool(memory)?;
                let left = OpPrimitive::get_bool(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::get_char(memory)?;
                let left = OpPrimitive::get_char(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right = OpPrimitive::get_string(memory)?;
                let left = OpPrimitive::get_string(memory)?;
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
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::Greater, memory)
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::get_bool(memory)?;
                let left = OpPrimitive::get_bool(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::get_char(memory)?;
                let left = OpPrimitive::get_char(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right = OpPrimitive::get_string(memory)?;
                let left = OpPrimitive::get_string(memory)?;
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
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::GreaterEqual, memory)
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::get_bool(memory)?;
                let left = OpPrimitive::get_bool(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::get_char(memory)?;
                let left = OpPrimitive::get_char(memory)?;
                memory
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right = OpPrimitive::get_string(memory)?;
                let left = OpPrimitive::get_string(memory)?;
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
    pub left: usize,
    pub right: usize,
}
#[derive(Debug, Clone)]
pub struct NotEqual {
    pub left: usize,
    pub right: usize,
}

impl Executable for Equal {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let data = {
            let right_data = memory.stack.pop(self.right).map_err(|e| e.into())?;

            let left_data = memory.stack.pop(self.left).map_err(|e| e.into())?;

            [(left_data == right_data) as u8]
        };
        memory.stack.push_with(&data).map_err(|e| e.into())
    }
}

impl Executable for NotEqual {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let data = {
            let right_data = memory.stack.pop(self.right).map_err(|e| e.into())?;

            let left_data = memory.stack.pop(self.left).map_err(|e| e.into())?;

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
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let _left_data = memory.stack.pop(self.left).map_err(|e| e.into())?;

        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct LogicalAnd();

impl Executable for LogicalAnd {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let right_data = OpPrimitive::get_bool(memory)?;
        let left_data = OpPrimitive::get_bool(memory)?;
        let data = [(left_data && right_data) as u8];
        memory.stack.push_with(&data).map_err(|e| e.into())
    }
}

#[derive(Debug, Clone)]
pub struct LogicalOr();

impl Executable for LogicalOr {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let right_data = OpPrimitive::get_bool(memory)?;
        let left_data = OpPrimitive::get_bool(memory)?;
        let data = [(left_data || right_data) as u8];
        memory.stack.push_with(&data).map_err(|e| e.into())
    }
}

#[derive(Debug, Clone)]
pub struct Minus {
    pub data_type: OpPrimitive,
}

impl Executable for Minus {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match &self.data_type {
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
                NumberType::F64 => {
                    let data = OpPrimitive::get_num8::<f64>(memory)?;
                    memory
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
            },
            OpPrimitive::Char => Err(RuntimeError::UnsupportedOperation),
            OpPrimitive::Bool => Err(RuntimeError::UnsupportedOperation),
            OpPrimitive::String => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Not();

impl Executable for Not {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let data = OpPrimitive::get_bool(memory)?;
        let data = [(!data) as u8];
        memory.stack.push_with(&data).map_err(|e| e.into())
    }
}

#[derive(Debug, Clone)]
pub struct Cast {
    pub from: OpPrimitive,
    pub to: OpPrimitive,
}

macro_rules! push_data_as_type {
    ($data:expr, $num_type:expr, $memory:expr) => {
        match $num_type {
            NumberType::U8 => $memory
                .stack
                .push_with(&($data as u8).to_le_bytes())
                .map_err(|e| e.into()),
            NumberType::U16 => $memory
                .stack
                .push_with(&($data as u16).to_le_bytes())
                .map_err(|e| e.into()),
            NumberType::U32 => $memory
                .stack
                .push_with(&($data as u32).to_le_bytes())
                .map_err(|e| e.into()),
            NumberType::U64 => $memory
                .stack
                .push_with(&($data as u64).to_le_bytes())
                .map_err(|e| e.into()),
            NumberType::U128 => $memory
                .stack
                .push_with(&($data as u128).to_le_bytes())
                .map_err(|e| e.into()),
            NumberType::I8 => $memory
                .stack
                .push_with(&($data as i8).to_le_bytes())
                .map_err(|e| e.into()),
            NumberType::I16 => $memory
                .stack
                .push_with(&($data as i16).to_le_bytes())
                .map_err(|e| e.into()),
            NumberType::I32 => $memory
                .stack
                .push_with(&($data as i32).to_le_bytes())
                .map_err(|e| e.into()),
            NumberType::I64 => $memory
                .stack
                .push_with(&($data as i64).to_le_bytes())
                .map_err(|e| e.into()),
            NumberType::I128 => $memory
                .stack
                .push_with(&($data as i128).to_le_bytes())
                .map_err(|e| e.into()),
            NumberType::F64 => $memory
                .stack
                .push_with(&($data as f64).to_le_bytes())
                .map_err(|e| e.into()),
        }
    };
}

impl Executable for Cast {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match (self.from, self.to) {
            (OpPrimitive::Number(number), OpPrimitive::Number(to)) => match number {
                NumberType::U8 => {
                    let data = OpPrimitive::get_num1::<u8>(memory)?;
                    push_data_as_type!(data, to, memory)
                }
                NumberType::U16 => {
                    let data = OpPrimitive::get_num2::<u16>(memory)? as f64;
                    push_data_as_type!(data, to, memory)
                }
                NumberType::U32 => {
                    let data = OpPrimitive::get_num4::<u32>(memory)? as f64;
                    push_data_as_type!(data, to, memory)
                }
                NumberType::U64 => {
                    let data = OpPrimitive::get_num8::<u64>(memory)? as f64;
                    push_data_as_type!(data, to, memory)
                }
                NumberType::U128 => {
                    let data = OpPrimitive::get_num16::<u128>(memory)? as f64;
                    push_data_as_type!(data, to, memory)
                }
                NumberType::I8 => {
                    let data = OpPrimitive::get_num1::<i8>(memory)? as f64;
                    push_data_as_type!(data, to, memory)
                }
                NumberType::I16 => {
                    let data = OpPrimitive::get_num2::<i16>(memory)? as f64;
                    push_data_as_type!(data, to, memory)
                }
                NumberType::I32 => {
                    let data = OpPrimitive::get_num4::<i32>(memory)? as f64;
                    push_data_as_type!(data, to, memory)
                }
                NumberType::I64 => {
                    let data = OpPrimitive::get_num8::<i64>(memory)? as f64;
                    push_data_as_type!(data, to, memory)
                }
                NumberType::I128 => {
                    let data = OpPrimitive::get_num16::<i128>(memory)? as f64;
                    push_data_as_type!(data, to, memory)
                }
                NumberType::F64 => {
                    let data = OpPrimitive::get_num8::<f64>(memory)? as f64;
                    push_data_as_type!(data, to, memory)
                }
            },
            (OpPrimitive::Number(number), OpPrimitive::Bool) => match number {
                NumberType::U8 => {
                    let data = OpPrimitive::get_num1::<u8>(memory)?;
                    memory
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let data = OpPrimitive::get_num2::<u16>(memory)?;
                    memory
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let data = OpPrimitive::get_num4::<u32>(memory)?;
                    memory
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let data = OpPrimitive::get_num8::<u64>(memory)?;
                    memory
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let data = OpPrimitive::get_num16::<u128>(memory)?;
                    memory
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let data = OpPrimitive::get_num1::<i8>(memory)?;
                    memory
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let data = OpPrimitive::get_num2::<i16>(memory)?;
                    memory
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let data = OpPrimitive::get_num4::<i32>(memory)?;
                    memory
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let data = OpPrimitive::get_num8::<i64>(memory)?;
                    memory
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let data = OpPrimitive::get_num16::<i128>(memory)?;
                    memory
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::F64 => {
                    let data = OpPrimitive::get_num8::<f64>(memory)?;
                    memory
                        .stack
                        .push_with(&[(data == 0.0) as u8])
                        .map_err(|e| e.into())
                }
            },
            (OpPrimitive::Number(NumberType::U8), OpPrimitive::Char) => Ok(()),
            (OpPrimitive::Number(_), OpPrimitive::Char) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Number(_), OpPrimitive::String) => {
                Err(RuntimeError::UnsupportedOperation)
            }
            (OpPrimitive::Bool, OpPrimitive::Number(number)) => {
                let data = OpPrimitive::get_char(memory)? as u8;
                match number {
                    NumberType::U8 => memory
                        .stack
                        .push_with(&data.to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U16 => memory
                        .stack
                        .push_with(&(data as u16).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U32 => memory
                        .stack
                        .push_with(&(data as u32).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U64 => memory
                        .stack
                        .push_with(&(data as u64).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U128 => memory
                        .stack
                        .push_with(&(data as u128).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::I8 => memory
                        .stack
                        .push_with(&(data as i8).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::I16 => memory
                        .stack
                        .push_with(&(data as i16).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::I32 => memory
                        .stack
                        .push_with(&(data as i32).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::I64 => memory
                        .stack
                        .push_with(&(data as i64).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::I128 => memory
                        .stack
                        .push_with(&(data as i128).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::F64 => memory
                        .stack
                        .push_with(&(data as f64).to_le_bytes())
                        .map_err(|e| e.into()),
                }
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => Ok(()),
            (OpPrimitive::Bool, OpPrimitive::Char) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Bool, OpPrimitive::String) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Char, OpPrimitive::Number(number)) => {
                let data = OpPrimitive::get_char(memory)? as u8;
                match number {
                    NumberType::U8 => memory
                        .stack
                        .push_with(&data.to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U16 => memory
                        .stack
                        .push_with(&(data as u16).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U32 => memory
                        .stack
                        .push_with(&(data as u32).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U64 => memory
                        .stack
                        .push_with(&(data as u64).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U128 => memory
                        .stack
                        .push_with(&(data as u128).to_le_bytes())
                        .map_err(|e| e.into()),
                    _ => Err(RuntimeError::UnsupportedOperation),
                }
            }
            (OpPrimitive::Char, OpPrimitive::Bool) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Char, OpPrimitive::Char) => Ok(()),
            (OpPrimitive::Char, OpPrimitive::String) => Ok(()),
            (OpPrimitive::String, OpPrimitive::Number(_)) => {
                Err(RuntimeError::UnsupportedOperation)
            }
            (OpPrimitive::String, OpPrimitive::Bool) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::String, OpPrimitive::Char) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::String, OpPrimitive::String) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_float(num: f64, memory: &Memory) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_num1<T: num_traits::ToBytes<Bytes = [u8; 1]>>(
        num: T,
        memory: &Memory,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_num2<T: num_traits::ToBytes<Bytes = [u8; 2]>>(
        num: T,
        memory: &Memory,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_num4<T: num_traits::ToBytes<Bytes = [u8; 4]>>(
        num: T,
        memory: &Memory,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_num8<T: num_traits::ToBytes<Bytes = [u8; 8]>>(
        num: T,
        memory: &Memory,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_num16<T: num_traits::ToBytes<Bytes = [u8; 16]>>(
        num: T,
        memory: &Memory,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
        Ok(())
    }
    fn init_char(memory: &Memory) -> Result<(), RuntimeError> {
        let data = vec!['a' as u8];
        let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_bool(state: bool, memory: &Memory) -> Result<(), RuntimeError> {
        let data = vec![state as u8];
        let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
        Ok(())
    }

    fn init_string(text: &str, memory: &Memory) -> Result<(), RuntimeError> {
        let data = text.as_bytes().to_vec();
        let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
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
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(&memory).expect("result should be of valid type");
        assert_eq!(10 * 20, res);
    }

    #[test]
    fn valid_div() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(2u32, &memory).expect("init should have succeeded");
        Division {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(&memory).expect("result should be of valid type");
        assert_eq!(10 / 2, res);
    }

    #[test]
    fn valid_mod() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(2u32, &memory).expect("init should have succeeded");
        Mod {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(&memory).expect("result should be of valid type");
        assert_eq!(10 % 2, res);
    }

    #[test]
    fn valid_add() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(20u32, &memory).expect("init should have succeeded");
        Addition {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(&memory).expect("result should be of valid type");
        assert_eq!(10 + 20, res);
    }

    #[test]
    fn valid_sub() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(5u32, &memory).expect("init should have succeeded");
        Substraction {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(&memory).expect("result should be of valid type");
        assert_eq!(10 - 5, res);
    }

    #[test]
    fn valid_sl() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(5u32, &memory).expect("init should have succeeded");
        ShiftLeft {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(&memory).expect("result should be of valid type");
        assert_eq!(10 << 5, res);
    }

    #[test]
    fn valid_sr() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(2u32, &memory).expect("init should have succeeded");
        ShiftRight {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(&memory).expect("result should be of valid type");
        assert_eq!(10 >> 2, res);
    }

    #[test]
    fn valid_bitand() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(5u32, &memory).expect("init should have succeeded");
        BitwiseAnd {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(&memory).expect("result should be of valid type");
        assert_eq!(10 & 5, res);
    }

    #[test]
    fn valid_bitxor() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(5u32, &memory).expect("init should have succeeded");
        BitwiseXOR {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(&memory).expect("result should be of valid type");
        assert_eq!(10 ^ 5, res);
    }

    #[test]
    fn valid_bitor() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(5u32, &memory).expect("init should have succeeded");
        BitwiseOR {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(&memory).expect("result should be of valid type");
        assert_eq!(10 | 5, res);
    }

    #[test]
    fn valid_less() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(5u32, &memory).expect("init should have succeeded");
        Less {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&memory).expect("result should be of valid type");
        assert_eq!(10 < 5, res);
    }

    #[test]
    fn valid_less_equal() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(5u32, &memory).expect("init should have succeeded");
        LessEqual {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&memory).expect("result should be of valid type");
        assert_eq!(10 <= 5, res);
    }

    #[test]
    fn valid_greater() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(5u32, &memory).expect("init should have succeeded");
        Greater {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&memory).expect("result should be of valid type");
        assert_eq!(10 > 5, res);
    }

    #[test]
    fn valid_greater_equal() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(5u32, &memory).expect("init should have succeeded");
        GreaterEqual {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&memory).expect("result should be of valid type");
        assert_eq!(10 >= 5, res);
    }

    #[test]
    fn valid_equal() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(10u32, &memory).expect("init should have succeeded");
        GreaterEqual {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&memory).expect("result should be of valid type");
        assert_eq!(10 == 10, res);
    }

    #[test]
    fn valid_not_equal() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        init_num4(5u32, &memory).expect("init should have succeeded");
        GreaterEqual {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&memory).expect("result should be of valid type");
        assert_eq!(10 != 5, res);
    }

    #[test]
    fn valid_logical_and() {
        let memory = Memory::new();
        init_bool(true, &memory).expect("init should have succeeded");
        init_bool(true, &memory).expect("init should have succeeded");
        LogicalAnd()
            .execute(&CasmProgram::default(), &memory)
            .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&memory).expect("result should be of valid type");
        assert_eq!(true && true, res);
    }

    #[test]
    fn valid_logical_or() {
        let memory = Memory::new();
        init_bool(true, &memory).expect("init should have succeeded");
        init_bool(true, &memory).expect("init should have succeeded");
        LogicalOr()
            .execute(&CasmProgram::default(), &memory)
            .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&memory).expect("result should be of valid type");
        assert_eq!(true || true, res);
    }

    #[test]
    fn valid_minus() {
        let memory = Memory::new();
        init_num4(10u32, &memory).expect("init should have succeeded");
        Minus {
            data_type: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&CasmProgram::default(), &memory)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num8::<i64>(&memory).expect("result should be of valid type");
        assert_eq!(-10i64, res);
    }

    #[test]
    fn valid_not() {
        let memory = Memory::new();
        init_bool(true, &memory).expect("init should have succeeded");
        Not()
            .execute(&CasmProgram::default(), &memory)
            .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&memory).expect("result should be of valid type");
        assert_eq!(false, res);
    }
}
