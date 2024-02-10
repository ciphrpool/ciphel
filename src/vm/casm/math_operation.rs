use num_traits::ToBytes;

use crate::{
    semantic::scope::static_types::NumberType,
    vm::{allocator::Memory, vm::RuntimeError},
};

use super::operation::OpPrimitive;

macro_rules! perform_operation {
    ($left:expr, $right:expr, $op:tt, $cast_left:ty, $cast_right:ty) => {
        match $op {
            MathOperator::Mult => ($left as $cast_left)
                .checked_mul($right as $cast_right)
                .ok_or(RuntimeError::MathError)?
                .to_le_bytes(),
            MathOperator::Div => {
                if $right == 0 {
                    return Err(RuntimeError::MathError);
                }

                ($left as $cast_left)
                    .checked_div(($right as $cast_right))
                    .ok_or(RuntimeError::MathError)?
                    .to_le_bytes()
            }
            MathOperator::Mod => ($left as $cast_left)
                .checked_rem($right as $cast_right)
                .ok_or(RuntimeError::MathError)?
                .to_le_bytes(),
            MathOperator::Add => ($left as $cast_left)
                .checked_add($right as $cast_right)
                .ok_or(RuntimeError::MathError)?
                .to_le_bytes(),
            MathOperator::Sub => ($left as $cast_left)
                .checked_sub($right as $cast_right)
                .ok_or(RuntimeError::MathError)?
                .to_le_bytes(),
            MathOperator::BitAnd => ($left as $cast_left & ($right as $cast_right)).to_le_bytes(),
            MathOperator::BitOr => ($left as $cast_left | ($right as $cast_right)).to_le_bytes(),
            MathOperator::BitXor => ($left as $cast_left ^ ($right as $cast_right)).to_le_bytes(),
            MathOperator::ShiftLeft => ($left as $cast_left << ($right as $cast_right)).to_le_bytes(),
            MathOperator::ShiftRight => ($left as $cast_left >> ($right as $cast_right)).to_le_bytes(),
        }
    };
}

macro_rules! perform_operation_f64 {
    ($left:expr, $right:expr, $op:tt) => {
        match $op {
            MathOperator::Mult => $left * $right,
            MathOperator::Div => {
                if $right == 0.0 {
                    return Err(RuntimeError::MathError);
                }
                $left / $right
            }
            MathOperator::Mod => $left % $right,
            MathOperator::Add => $left + $right,
            MathOperator::Sub => $left - $right,
            // Bitwise operations are not applicable for f64, can return an error or handle differently
            _ => return Err(RuntimeError::UnsupportedOperation),
        }
    };
}

pub enum MathOperator {
    Mult,
    Div,
    Mod,
    Add,
    Sub,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

pub fn math_operator(
    left: &NumberType,
    right: &NumberType,
    operator: MathOperator,
    memory: &Memory,
) -> Result<(), RuntimeError> {
    match left {
        NumberType::U8 => {
            let right_data = OpPrimitive::get_num1::<u8>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u8, u8);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u16, u16);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u32, u32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u64, u64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u128, u128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i16, i16);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i16, i16);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i32, i32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::U16 => {
            let right_data = OpPrimitive::get_num2::<u16>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u16, u16);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u16, u16);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u32, u32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u64, u64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u128, u128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)? as i32;
                    let result = perform_operation!(left_data, right_data, operator, i32, i32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)? as i32;
                    let result = perform_operation!(left_data, right_data, operator, i32, i32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i32, i32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::U32 => {
            let right_data = OpPrimitive::get_num4::<u32>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u32, u32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u32, u32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u32, u32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u64, u64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u128, u128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)? as i64;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)? as i64;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)? as i64;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::U64 => {
            let right_data = OpPrimitive::get_num8::<u64>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u64, u64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u64, u64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u64, u64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u64, u64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u128, u128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)? as i128;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)? as i128;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)? as i128;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)? as i128;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::U128 => {
            let right_data = OpPrimitive::get_num16::<u128>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u128, u128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u128, u128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u128, u128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u128, u128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, u128, u128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)? as i128;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)? as i128;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)? as i128;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)? as i128;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::I8 => {
            let right_data = OpPrimitive::get_num1::<i8>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i16, i16);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i32, i32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i8, i8);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i16, i16);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i32, i32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::I16 => {
            let right_data = OpPrimitive::get_num2::<i16>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i16, i16);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i32, i32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i16, i16);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i16, i16);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i32, i32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::I32 => {
            let right_data = OpPrimitive::get_num4::<i32>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i32, i32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i32, i32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i32, i32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i32, i32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i32, i32);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::I64 => {
            let right_data = OpPrimitive::get_num8::<i64>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i64, i64);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::I128 => {
            let right_data = OpPrimitive::get_num16::<i128>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result = perform_operation!(left_data, right_data, operator, i128, i128);
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
    }
}

pub fn math_operator_float_left(
    left: &NumberType,
    operator: MathOperator,
    memory: &Memory,
) -> Result<(), RuntimeError> {
    let right = OpPrimitive::get_float(memory)?;
    match left {
        NumberType::U8 => {
            let left_data = OpPrimitive::get_num1::<u8>(memory)? as f64;
            let result = perform_operation_f64!(left_data, right, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::U16 => {
            let left_data = OpPrimitive::get_num2::<u16>(memory)? as f64;
            let result = perform_operation_f64!(left_data, right, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::U32 => {
            let left_data = OpPrimitive::get_num4::<u32>(memory)? as f64;
            let right = OpPrimitive::get_float(memory)?;
            let result = perform_operation_f64!(left_data, right, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::U64 => {
            let left_data = OpPrimitive::get_num8::<u64>(memory)? as f64;
            let result = perform_operation_f64!(left_data, right, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::U128 => {
            let left_data = OpPrimitive::get_num16::<u128>(memory)? as f64;
            let result = perform_operation_f64!(left_data, right, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::I8 => {
            let left_data = OpPrimitive::get_num1::<i8>(memory)? as f64;
            let result = perform_operation_f64!(left_data, right, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::I16 => {
            let left_data = OpPrimitive::get_num2::<i16>(memory)? as f64;
            let result = perform_operation_f64!(left_data, right, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::I32 => {
            let left_data = OpPrimitive::get_num4::<i32>(memory)? as f64;
            let result = perform_operation_f64!(left_data, right, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::I64 => {
            let left_data = OpPrimitive::get_num8::<i64>(memory)? as f64;
            let result = perform_operation_f64!(left_data, right, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::I128 => {
            let left_data = OpPrimitive::get_num16::<i128>(memory)? as f64;
            let result = perform_operation_f64!(left_data, right, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
    }
}

pub fn math_operator_float_right(
    right: &NumberType,
    operator: MathOperator,
    memory: &Memory,
) -> Result<(), RuntimeError> {
    match right {
        NumberType::U8 => {
            let right_data = OpPrimitive::get_num1::<u8>(memory)? as f64;
            let left = OpPrimitive::get_float(memory)?;
            let result = perform_operation_f64!(left, right_data, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::U16 => {
            let right_data = OpPrimitive::get_num2::<u16>(memory)? as f64;
            let left = OpPrimitive::get_float(memory)?;
            let result = perform_operation_f64!(left, right_data, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::U32 => {
            let right_data = OpPrimitive::get_num4::<u32>(memory)? as f64;
            let left = OpPrimitive::get_float(memory)?;
            let result = perform_operation_f64!(left, right_data, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::U64 => {
            let right_data = OpPrimitive::get_num8::<u64>(memory)? as f64;
            let left = OpPrimitive::get_float(memory)?;
            let result = perform_operation_f64!(left, right_data, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::U128 => {
            let right_data = OpPrimitive::get_num16::<u128>(memory)? as f64;
            let left = OpPrimitive::get_float(memory)?;
            let result = perform_operation_f64!(left, right_data, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::I8 => {
            let right_data = OpPrimitive::get_num1::<i8>(memory)? as f64;
            let left = OpPrimitive::get_float(memory)?;
            let result = perform_operation_f64!(left, right_data, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::I16 => {
            let right_data = OpPrimitive::get_num2::<i16>(memory)? as f64;
            let left = OpPrimitive::get_float(memory)?;
            let result = perform_operation_f64!(left, right_data, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::I32 => {
            let right_data = OpPrimitive::get_num4::<i32>(memory)? as f64;
            let left = OpPrimitive::get_float(memory)?;
            let result = perform_operation_f64!(left, right_data, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::I64 => {
            let right_data = OpPrimitive::get_num8::<i64>(memory)? as f64;
            let left = OpPrimitive::get_float(memory)?;
            let result = perform_operation_f64!(left, right_data, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
        NumberType::I128 => {
            let right_data = OpPrimitive::get_num16::<i128>(memory)? as f64;
            let left = OpPrimitive::get_float(memory)?;
            let result = perform_operation_f64!(left, right_data, operator).to_le_bytes();
            memory.stack.push_with(&result).map_err(|e| e.into())
        }
    }
}

macro_rules! perform_comparaison {
    ($left:expr, $right:expr, $op:tt, $cast_left:ty, $cast_right:ty) => {
        match $op {
            ComparaisonOperator::Less => ($left as $cast_left) < ($right as $cast_right),
            ComparaisonOperator::LessEqual => ($left as $cast_left) <= ($right as $cast_right),
            ComparaisonOperator::Greater => ($left as $cast_left) > ($right as $cast_right),
            ComparaisonOperator::GreaterEqual => ($left as $cast_left) >= ($right as $cast_right),
            ComparaisonOperator::Equal => ($left as $cast_left) == ($right as $cast_right),
            ComparaisonOperator::NotEqual => ($left as $cast_left) != ($right as $cast_right),
        }
    };
}

macro_rules! perform_comparaison_default {
    ($left:expr, $right:expr, $op:tt) => {
        match $op {
            ComparaisonOperator::Less => $left < $right,
            ComparaisonOperator::LessEqual => $left <= $right,
            ComparaisonOperator::Greater => $left > $right,
            ComparaisonOperator::GreaterEqual => $left >= $right,
            ComparaisonOperator::Equal => $left == $right,
            ComparaisonOperator::NotEqual => $left != $right,
        }
    };
}

pub enum ComparaisonOperator {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
}

pub fn comparaison_operator(
    left: &NumberType,
    right: &NumberType,
    operator: ComparaisonOperator,
    memory: &Memory,
) -> Result<(), RuntimeError> {
    match left {
        NumberType::U8 => {
            let right_data = OpPrimitive::get_num1::<u8>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u8, u8) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u16, u16) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u32, u32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u64, u64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u128, u128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)? as i16;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i16, i16) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i16, i16) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i32, i32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::U16 => {
            let right_data = OpPrimitive::get_num2::<u16>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u16, u16) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u16, u16) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u32, u32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u64, u64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u128, u128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)? as i32;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i32, i32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)? as i32;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i32, i32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i32, i32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::U32 => {
            let right_data = OpPrimitive::get_num4::<u32>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u32, u32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u32, u32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u32, u32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u64, u64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u128, u128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)? as i64;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)? as i64;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)? as i64;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::U64 => {
            let right_data = OpPrimitive::get_num8::<u64>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u64, u64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u64, u64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u64, u64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u64, u64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u128, u128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)? as i128;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)? as i128;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)? as i128;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)? as i128;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::U128 => {
            let right_data = OpPrimitive::get_num16::<u128>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u128, u128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u128, u128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u128, u128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u128, u128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, u128, u128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)? as i128;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)? as i128;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)? as i128;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)? as i128;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::I8 => {
            let right_data = OpPrimitive::get_num1::<i8>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i16, i16) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i32, i32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i8, i8) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i16, i16) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i32, i32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::I16 => {
            let right_data = OpPrimitive::get_num2::<i16>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i16, i16) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i32, i32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i16, i16) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i16, i16) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i32, i32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::I32 => {
            let right_data = OpPrimitive::get_num4::<i32>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i32, i32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i32, i32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i32, i32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i32, i32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i32, i32) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::I64 => {
            let right_data = OpPrimitive::get_num8::<i64>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i64, i64) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
        NumberType::I128 => {
            let right_data = OpPrimitive::get_num16::<i128>(memory)?;
            match right {
                NumberType::U8 => {
                    let left_data = OpPrimitive::get_num1::<u8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U16 => {
                    let left_data = OpPrimitive::get_num2::<u16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U32 => {
                    let left_data = OpPrimitive::get_num4::<u32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U64 => {
                    let left_data = OpPrimitive::get_num8::<u64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::U128 => {
                    let left_data = OpPrimitive::get_num16::<u128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I8 => {
                    let left_data = OpPrimitive::get_num1::<i8>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I16 => {
                    let left_data = OpPrimitive::get_num2::<i16>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I32 => {
                    let left_data = OpPrimitive::get_num4::<i32>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I64 => {
                    let left_data = OpPrimitive::get_num8::<i64>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
                NumberType::I128 => {
                    let left_data = OpPrimitive::get_num16::<i128>(memory)?;
                    let result =
                        [perform_comparaison!(left_data, right_data, operator, i128, i128) as u8];
                    memory.stack.push_with(&result).map_err(|e| e.into())
                }
            }
        }
    }
}

macro_rules! perform_comparison_with_float_left {
    ($memory:expr, $left_type:ident, $operator:expr, $get_num_method:ident) => {{
        let right = OpPrimitive::get_float($memory)?;
        let left_data = OpPrimitive::$get_num_method::<$left_type>($memory)? as f64;
        let result = [perform_comparaison_default!(left_data, right, $operator) as u8];
        $memory.stack.push_with(&result).map_err(|e| e.into())
    }};
}

pub fn comparaison_operator_float_left(
    left: &NumberType,
    operator: ComparaisonOperator,
    memory: &Memory,
) -> Result<(), RuntimeError> {
    match left {
        NumberType::U8 => perform_comparison_with_float_left!(memory, u8, operator, get_num1),
        NumberType::U16 => perform_comparison_with_float_left!(memory, u16, operator, get_num2),
        NumberType::U32 => perform_comparison_with_float_left!(memory, u32, operator, get_num4),
        NumberType::U64 => perform_comparison_with_float_left!(memory, u64, operator, get_num8),
        NumberType::U128 => perform_comparison_with_float_left!(memory, u128, operator, get_num16),
        NumberType::I8 => perform_comparison_with_float_left!(memory, i8, operator, get_num1),
        NumberType::I16 => perform_comparison_with_float_left!(memory, i16, operator, get_num2),
        NumberType::I32 => perform_comparison_with_float_left!(memory, i32, operator, get_num4),
        NumberType::I64 => perform_comparison_with_float_left!(memory, i64, operator, get_num8),
        NumberType::I128 => perform_comparison_with_float_left!(memory, i128, operator, get_num16),
        // handle other cases as needed
    }
}

macro_rules! perform_comparison_with_float_right {
    ($memory:expr, $right_type:ident, $operator:expr, $get_num_method:ident) => {{
        let right_data = OpPrimitive::$get_num_method::<$right_type>($memory)? as f64;
        let left = OpPrimitive::get_float($memory)?;
        let result = [perform_comparaison_default!(left, right_data, $operator) as u8];
        $memory.stack.push_with(&result).map_err(|e| e.into())
    }};
}

pub fn comparaison_operator_float_right(
    right: &NumberType,
    operator: ComparaisonOperator,
    memory: &Memory,
) -> Result<(), RuntimeError> {
    match right {
        NumberType::U8 => perform_comparison_with_float_right!(memory, u8, operator, get_num1),
        NumberType::U16 => perform_comparison_with_float_right!(memory, u16, operator, get_num2),
        NumberType::U32 => perform_comparison_with_float_right!(memory, u32, operator, get_num4),
        NumberType::U64 => perform_comparison_with_float_right!(memory, u64, operator, get_num8),
        NumberType::U128 => perform_comparison_with_float_right!(memory, u128, operator, get_num16),
        NumberType::I8 => perform_comparison_with_float_right!(memory, i8, operator, get_num1),
        NumberType::I16 => perform_comparison_with_float_right!(memory, i16, operator, get_num2),
        NumberType::I32 => perform_comparison_with_float_right!(memory, i32, operator, get_num4),
        NumberType::I64 => perform_comparison_with_float_right!(memory, i64, operator, get_num8),
        NumberType::I128 => perform_comparison_with_float_right!(memory, i128, operator, get_num16),
    }
}
