use super::CasmProgram;
use crate::{
    semantic::{
        scope::static_types::{
            st_deserialize::extract_u64, NumberType, PrimitiveType, StaticType, StrSliceType,
        },
        EType, Either, SizeOf,
    },
    vm::{
        allocator::{Memory, MemoryAddress},
        scheduler::Thread,
        vm::{CodeGenerationError, Executable, Runtime, RuntimeError},
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        let _ = self.kind.execute(thread)?;
        thread.env.program.incr();
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
    String(usize),
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
                StaticType::StrSlice(StrSliceType { size }) => Ok(OpPrimitive::String(*size)),
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
    pub fn get_str_slice(size: usize, memory: &Memory) -> Result<String, RuntimeError> {
        let data = memory.stack.pop(size).map_err(|e| e.into())?;
        let data = std::str::from_utf8(&data).map_err(|_| RuntimeError::Deserialization)?;
        Ok(data.to_string())
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match self {
            OperationKind::Mult(value) => value.execute(thread),
            OperationKind::Div(value) => value.execute(thread),
            OperationKind::Mod(value) => value.execute(thread),
            OperationKind::Addition(value) => value.execute(thread),
            OperationKind::Substraction(value) => value.execute(thread),
            OperationKind::ShiftLeft(value) => value.execute(thread),
            OperationKind::ShiftRight(value) => value.execute(thread),
            OperationKind::BitwiseAnd(value) => value.execute(thread),
            OperationKind::BitwiseXOR(value) => value.execute(thread),
            OperationKind::BitwiseOR(value) => value.execute(thread),
            OperationKind::Cast(value) => value.execute(thread),
            OperationKind::Less(value) => value.execute(thread),
            OperationKind::LessEqual(value) => value.execute(thread),
            OperationKind::Greater(value) => value.execute(thread),
            OperationKind::GreaterEqual(value) => value.execute(thread),
            OperationKind::Equal(value) => value.execute(thread),
            OperationKind::NotEqual(value) => value.execute(thread),
            OperationKind::Inclusion(value) => value.execute(thread),
            OperationKind::LogicalAnd(value) => value.execute(thread),
            OperationKind::LogicalOr(value) => value.execute(thread),
            OperationKind::Minus(value) => value.execute(thread),
            OperationKind::Not(value) => value.execute(thread),
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Mult, &thread.memory())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for Division {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Div, &thread.memory())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for Mod {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Mod, &thread.memory())
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Add, &thread.memory())
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let right = OpPrimitive::get_str_slice(left_size, &thread.memory())?;
                let left = OpPrimitive::get_str_slice(right_size, &thread.memory())?;
                thread
                    .env
                    .stack
                    .push_with(&(left + &right).as_bytes())
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for Substraction {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Sub, &thread.memory())
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::ShiftLeft, &thread.memory())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for ShiftRight {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::ShiftRight, &thread.memory())
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::BitAnd, &thread.memory())
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::BitXor, &thread.memory())
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::BitOr, &thread.memory())
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::Less, &thread.memory())
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::get_bool(&thread.memory())?;
                let left = OpPrimitive::get_bool(&thread.memory())?;
                thread
                    .env
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::get_char(&thread.memory())?;
                let left = OpPrimitive::get_char(&thread.memory())?;
                thread
                    .env
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let right = OpPrimitive::get_str_slice(left_size, &thread.memory())?;
                let left = OpPrimitive::get_str_slice(right_size, &thread.memory())?;
                thread
                    .env
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for LessEqual {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => comparaison_operator(
                &left,
                &right,
                ComparaisonOperator::LessEqual,
                &thread.memory(),
            ),
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::get_bool(&thread.memory())?;
                let left = OpPrimitive::get_bool(&thread.memory())?;
                thread
                    .env
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::get_char(&thread.memory())?;
                let left = OpPrimitive::get_char(&thread.memory())?;
                thread
                    .env
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let right = OpPrimitive::get_str_slice(left_size, &thread.memory())?;
                let left = OpPrimitive::get_str_slice(right_size, &thread.memory())?;
                thread
                    .env
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for Greater {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => comparaison_operator(
                &left,
                &right,
                ComparaisonOperator::Greater,
                &thread.memory(),
            ),
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::get_bool(&thread.memory())?;
                let left = OpPrimitive::get_bool(&thread.memory())?;
                thread
                    .env
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::get_char(&thread.memory())?;
                let left = OpPrimitive::get_char(&thread.memory())?;
                thread
                    .env
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let right = OpPrimitive::get_str_slice(left_size, &thread.memory())?;
                let left = OpPrimitive::get_str_slice(right_size, &thread.memory())?;
                thread
                    .env
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl Executable for GreaterEqual {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => comparaison_operator(
                &left,
                &right,
                ComparaisonOperator::GreaterEqual,
                &thread.memory(),
            ),
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::get_bool(&thread.memory())?;
                let left = OpPrimitive::get_bool(&thread.memory())?;
                thread
                    .env
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::get_char(&thread.memory())?;
                let left = OpPrimitive::get_char(&thread.memory())?;
                thread
                    .env
                    .stack
                    .push_with(&[(left < right) as u8])
                    .map_err(|e| e.into())
            }
            (OpPrimitive::String(left_size), OpPrimitive::String(right_size)) => {
                let right = OpPrimitive::get_str_slice(left_size, &thread.memory())?;
                let left = OpPrimitive::get_str_slice(right_size, &thread.memory())?;
                thread
                    .env
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        let data = {
            let right_data = thread.env.stack.pop(self.right).map_err(|e| e.into())?;

            let left_data = thread.env.stack.pop(self.left).map_err(|e| e.into())?;

            [(left_data == right_data) as u8]
        };
        thread.env.stack.push_with(&data).map_err(|e| e.into())
    }
}

impl Executable for NotEqual {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        let data = {
            let right_data = thread.env.stack.pop(self.right).map_err(|e| e.into())?;

            let left_data = thread.env.stack.pop(self.left).map_err(|e| e.into())?;

            [(left_data != right_data) as u8]
        };
        thread.env.stack.push_with(&data).map_err(|e| e.into())
    }
}
#[derive(Debug, Clone)]
pub struct Inclusion {
    left: usize,
    iterator_addr: MemoryAddress,
    item_size: usize,
}

impl Executable for Inclusion {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        let _left_data = thread.env.stack.pop(self.left).map_err(|e| e.into())?;

        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct LogicalAnd();

impl Executable for LogicalAnd {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        let right_data = OpPrimitive::get_bool(&thread.memory())?;
        let left_data = OpPrimitive::get_bool(&thread.memory())?;
        let data = [(left_data && right_data) as u8];
        thread.env.stack.push_with(&data).map_err(|e| e.into())
    }
}

#[derive(Debug, Clone)]
pub struct LogicalOr();

impl Executable for LogicalOr {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        let right_data = OpPrimitive::get_bool(&thread.memory())?;
        let left_data = OpPrimitive::get_bool(&thread.memory())?;
        let data = [(left_data || right_data) as u8];
        thread.env.stack.push_with(&data).map_err(|e| e.into())
    }
}

#[derive(Debug, Clone)]
pub struct Minus {
    pub data_type: OpPrimitive,
}

impl Executable for Minus {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match &self.data_type {
            OpPrimitive::Number(number) => match number {
                NumberType::U8 => {
                    let data = OpPrimitive::get_num1::<u8>(&thread.memory())? as i16;
                    thread
                        .env
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let data = OpPrimitive::get_num2::<u16>(&thread.memory())? as i32;
                    thread
                        .env
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let data = OpPrimitive::get_num4::<u32>(&thread.memory())? as i64;
                    thread
                        .env
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let data = OpPrimitive::get_num8::<u64>(&thread.memory())? as i128;
                    thread
                        .env
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let data = OpPrimitive::get_num16::<u128>(&thread.memory())? as i128;
                    thread
                        .env
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let data = OpPrimitive::get_num1::<i8>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let data = OpPrimitive::get_num2::<i16>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let data = OpPrimitive::get_num4::<i32>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let data = OpPrimitive::get_num8::<i64>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let data = OpPrimitive::get_num16::<i128>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&(-data).to_le_bytes())
                        .map_err(|e| e.into())
                }
                NumberType::F64 => {
                    let data = OpPrimitive::get_num8::<f64>(&thread.memory())?;
                    thread
                        .env
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        let data = OpPrimitive::get_bool(&thread.memory())?;
        let data = [(!data) as u8];
        thread.env.stack.push_with(&data).map_err(|e| e.into())
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match (self.from, self.to) {
            (OpPrimitive::Number(number), OpPrimitive::Number(to)) => match number {
                NumberType::U8 => {
                    let data = OpPrimitive::get_num1::<u8>(&thread.memory())?;
                    push_data_as_type!(data, to, thread.memory())
                }
                NumberType::U16 => {
                    let data = OpPrimitive::get_num2::<u16>(&thread.memory())? as f64;
                    push_data_as_type!(data, to, thread.memory())
                }
                NumberType::U32 => {
                    let data = OpPrimitive::get_num4::<u32>(&thread.memory())? as f64;
                    push_data_as_type!(data, to, thread.memory())
                }
                NumberType::U64 => {
                    let data = OpPrimitive::get_num8::<u64>(&thread.memory())? as f64;
                    push_data_as_type!(data, to, thread.memory())
                }
                NumberType::U128 => {
                    let data = OpPrimitive::get_num16::<u128>(&thread.memory())? as f64;
                    push_data_as_type!(data, to, thread.memory())
                }
                NumberType::I8 => {
                    let data = OpPrimitive::get_num1::<i8>(&thread.memory())? as f64;
                    push_data_as_type!(data, to, thread.memory())
                }
                NumberType::I16 => {
                    let data = OpPrimitive::get_num2::<i16>(&thread.memory())? as f64;
                    push_data_as_type!(data, to, thread.memory())
                }
                NumberType::I32 => {
                    let data = OpPrimitive::get_num4::<i32>(&thread.memory())? as f64;
                    push_data_as_type!(data, to, thread.memory())
                }
                NumberType::I64 => {
                    let data = OpPrimitive::get_num8::<i64>(&thread.memory())? as f64;
                    push_data_as_type!(data, to, thread.memory())
                }
                NumberType::I128 => {
                    let data = OpPrimitive::get_num16::<i128>(&thread.memory())? as f64;
                    push_data_as_type!(data, to, thread.memory())
                }
                NumberType::F64 => {
                    let data = OpPrimitive::get_num8::<f64>(&thread.memory())? as f64;
                    push_data_as_type!(data, to, thread.memory())
                }
            },
            (OpPrimitive::Number(number), OpPrimitive::Bool) => match number {
                NumberType::U8 => {
                    let data = OpPrimitive::get_num1::<u8>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let data = OpPrimitive::get_num2::<u16>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let data = OpPrimitive::get_num4::<u32>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let data = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let data = OpPrimitive::get_num16::<u128>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let data = OpPrimitive::get_num1::<i8>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let data = OpPrimitive::get_num2::<i16>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let data = OpPrimitive::get_num4::<i32>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let data = OpPrimitive::get_num8::<i64>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let data = OpPrimitive::get_num16::<i128>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&[(data == 0) as u8])
                        .map_err(|e| e.into())
                }
                NumberType::F64 => {
                    let data = OpPrimitive::get_num8::<f64>(&thread.memory())?;
                    thread
                        .env
                        .stack
                        .push_with(&[(data == 0.0) as u8])
                        .map_err(|e| e.into())
                }
            },
            (OpPrimitive::Number(NumberType::U8), OpPrimitive::Char) => Ok(()),
            (OpPrimitive::Number(_), OpPrimitive::Char) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Number(_), OpPrimitive::String(_)) => {
                Err(RuntimeError::UnsupportedOperation)
            }
            (OpPrimitive::Bool, OpPrimitive::Number(number)) => {
                let data = OpPrimitive::get_char(&thread.memory())? as u8;
                match number {
                    NumberType::U8 => thread
                        .env
                        .stack
                        .push_with(&data.to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U16 => thread
                        .env
                        .stack
                        .push_with(&(data as u16).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U32 => thread
                        .env
                        .stack
                        .push_with(&(data as u32).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U64 => thread
                        .env
                        .stack
                        .push_with(&(data as u64).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U128 => thread
                        .env
                        .stack
                        .push_with(&(data as u128).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::I8 => thread
                        .env
                        .stack
                        .push_with(&(data as i8).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::I16 => thread
                        .env
                        .stack
                        .push_with(&(data as i16).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::I32 => thread
                        .env
                        .stack
                        .push_with(&(data as i32).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::I64 => thread
                        .env
                        .stack
                        .push_with(&(data as i64).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::I128 => thread
                        .env
                        .stack
                        .push_with(&(data as i128).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::F64 => thread
                        .env
                        .stack
                        .push_with(&(data as f64).to_le_bytes())
                        .map_err(|e| e.into()),
                }
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => Ok(()),
            (OpPrimitive::Bool, OpPrimitive::Char) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Bool, OpPrimitive::String(_)) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Char, OpPrimitive::Number(number)) => {
                let data = OpPrimitive::get_char(&thread.memory())? as u8;
                match number {
                    NumberType::U8 => thread
                        .env
                        .stack
                        .push_with(&data.to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U16 => thread
                        .env
                        .stack
                        .push_with(&(data as u16).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U32 => thread
                        .env
                        .stack
                        .push_with(&(data as u32).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U64 => thread
                        .env
                        .stack
                        .push_with(&(data as u64).to_le_bytes())
                        .map_err(|e| e.into()),
                    NumberType::U128 => thread
                        .env
                        .stack
                        .push_with(&(data as u128).to_le_bytes())
                        .map_err(|e| e.into()),
                    _ => Err(RuntimeError::UnsupportedOperation),
                }
            }
            (OpPrimitive::Char, OpPrimitive::Bool) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Char, OpPrimitive::Char) => Ok(()),
            (OpPrimitive::Char, OpPrimitive::String(_)) => Ok(()),
            (OpPrimitive::String(_), OpPrimitive::Number(_)) => {
                Err(RuntimeError::UnsupportedOperation)
            }
            (OpPrimitive::String(_), OpPrimitive::Bool) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::String(_), OpPrimitive::Char) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::String(_), OpPrimitive::String(_)) => Ok(()),
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
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");

        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(20u32, &thread.memory()).expect("init should have succeeded");
        Mult {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res =
            OpPrimitive::get_num4::<u32>(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 * 20, res);
    }

    #[test]
    fn valid_div() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(2u32, &thread.memory()).expect("init should have succeeded");
        Division {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res =
            OpPrimitive::get_num4::<u32>(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 / 2, res);
    }

    #[test]
    fn valid_mod() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(2u32, &thread.memory()).expect("init should have succeeded");
        Mod {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res =
            OpPrimitive::get_num4::<u32>(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 % 2, res);
    }

    #[test]
    fn valid_add() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(20u32, &thread.memory()).expect("init should have succeeded");
        Addition {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res =
            OpPrimitive::get_num4::<u32>(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 + 20, res);
    }

    #[test]
    fn valid_sub() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(5u32, &thread.memory()).expect("init should have succeeded");
        Substraction {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res =
            OpPrimitive::get_num4::<u32>(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 - 5, res);
    }

    #[test]
    fn valid_sl() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(5u32, &thread.memory()).expect("init should have succeeded");
        ShiftLeft {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res =
            OpPrimitive::get_num4::<u32>(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 << 5, res);
    }

    #[test]
    fn valid_sr() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(2u32, &thread.memory()).expect("init should have succeeded");
        ShiftRight {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res =
            OpPrimitive::get_num4::<u32>(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 >> 2, res);
    }

    #[test]
    fn valid_bitand() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(5u32, &thread.memory()).expect("init should have succeeded");
        BitwiseAnd {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res =
            OpPrimitive::get_num4::<u32>(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 & 5, res);
    }

    #[test]
    fn valid_bitxor() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(5u32, &thread.memory()).expect("init should have succeeded");
        BitwiseXOR {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res =
            OpPrimitive::get_num4::<u32>(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 ^ 5, res);
    }

    #[test]
    fn valid_bitor() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(5u32, &thread.memory()).expect("init should have succeeded");
        BitwiseOR {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res =
            OpPrimitive::get_num4::<u32>(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 | 5, res);
    }

    #[test]
    fn valid_less() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(5u32, &thread.memory()).expect("init should have succeeded");
        Less {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 < 5, res);
    }

    #[test]
    fn valid_less_equal() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(5u32, &thread.memory()).expect("init should have succeeded");
        LessEqual {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 <= 5, res);
    }

    #[test]
    fn valid_greater() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(5u32, &thread.memory()).expect("init should have succeeded");
        Greater {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 > 5, res);
    }

    #[test]
    fn valid_greater_equal() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(5u32, &thread.memory()).expect("init should have succeeded");
        GreaterEqual {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 >= 5, res);
    }

    #[test]
    fn valid_equal() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        GreaterEqual {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 == 10, res);
    }

    #[test]
    fn valid_not_equal() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        init_num4(5u32, &thread.memory()).expect("init should have succeeded");
        GreaterEqual {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&thread.memory()).expect("result should be of valid type");
        assert_eq!(10 != 5, res);
    }

    #[test]
    fn valid_logical_and() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_bool(true, &thread.memory()).expect("init should have succeeded");
        init_bool(true, &thread.memory()).expect("init should have succeeded");
        LogicalAnd()
            .execute(&thread)
            .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&thread.memory()).expect("result should be of valid type");
        assert_eq!(true && true, res);
    }

    #[test]
    fn valid_logical_or() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_bool(true, &thread.memory()).expect("init should have succeeded");
        init_bool(true, &thread.memory()).expect("init should have succeeded");
        LogicalOr()
            .execute(&thread)
            .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&thread.memory()).expect("result should be of valid type");
        assert_eq!(true || true, res);
    }

    #[test]
    fn valid_minus() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_num4(10u32, &thread.memory()).expect("init should have succeeded");
        Minus {
            data_type: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&thread)
        .expect("execution should have succeeded");

        let res =
            OpPrimitive::get_num8::<i64>(&thread.memory()).expect("result should be of valid type");
        assert_eq!(-10i64, res);
    }

    #[test]
    fn valid_not() {
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        init_bool(true, &thread.memory()).expect("init should have succeeded");
        Not()
            .execute(&thread)
            .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(&thread.memory()).expect("result should be of valid type");
        assert_eq!(false, res);
    }
}
