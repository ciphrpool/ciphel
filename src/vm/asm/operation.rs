use std::fmt::Debug;

use crate::{
    semantic::{
        scope::static_types::{NumberType, PrimitiveType, StaticType},
        EType, SizeOf,
    },
    vm::{
        allocator::{align, heap::Heap, stack::Stack, MemoryAddress},
        runtime::RuntimeError,
        scheduler::Executable,
        stdio::StdIO,
        CodeGenerationError,
    },
};
use nom::AsBytes;
use num_traits::{FromBytes, PrimInt, ToBytes};

use super::math_operation::{
    comparaison_operator, math_operator, ComparaisonOperator, MathOperator,
};

#[derive(Debug, Clone)]
pub struct Operation {
    pub kind: OperationKind,
}

impl<E: crate::vm::external::Engine> Executable<E> for Operation {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        let _ = self.kind.execute(
            program,
            scheduler,
            signal_handler,
            stack,
            heap,
            stdio,
            engine,
            context,
        )?;
        scheduler.next();
        Ok(())
    }
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Operation {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E, pid : E::PID) {
        match self.kind {
            OperationKind::Align => stdio.push_asm(engine, pid, "align"),
            OperationKind::CastCharToUTF8 => stdio.push_asm(engine, pid, "char_to_utf8"),
            OperationKind::Mult(Mult { left, right }) => {
                stdio.push_asm(engine, pid, &format!("mult_{}_{}", left.name(), right.name()))
            }
            OperationKind::Div(Division { left, right }) => {
                stdio.push_asm(engine, pid, &format!("div_{}_{}", left.name(), right.name()))
            }
            OperationKind::Mod(Mod { left, right }) => {
                stdio.push_asm(engine, pid, &format!("mod_{}_{}", left.name(), right.name()))
            }
            OperationKind::Addition(Addition { left, right }) => {
                stdio.push_asm(engine, pid, &format!("add_{}_{}", left.name(), right.name()))
            }
            OperationKind::Substraction(Substraction { left, right }) => {
                stdio.push_asm(engine, pid, &format!("sub_{}_{}", left.name(), right.name()))
            }
            OperationKind::ShiftLeft(ShiftLeft { left, right }) => {
                stdio.push_asm(engine, pid, &format!("shl_{}_{}", left.name(), right.name()))
            }
            OperationKind::ShiftRight(ShiftRight { left, right }) => {
                stdio.push_asm(engine, pid, &format!("shr_{}_{}", left.name(), right.name()))
            }
            OperationKind::BitwiseAnd(BitwiseAnd { left, right }) => {
                stdio.push_asm(engine, pid, &format!("band_{}_{}", left.name(), right.name()))
            }
            OperationKind::BitwiseXOR(BitwiseXOR { left, right }) => {
                stdio.push_asm(engine, pid, &format!("bxor_{}_{}", left.name(), right.name()))
            }
            OperationKind::BitwiseOR(BitwiseOR { left, right }) => {
                stdio.push_asm(engine, pid, &format!("bor_{}_{}", left.name(), right.name()))
            }
            OperationKind::Cast(Cast { from, to }) => {
                stdio.push_asm(engine, pid, &format!("cast_{}_{}", from.name(), to.name()))
            }
            OperationKind::Less(Less { left, right }) => {
                stdio.push_asm(engine, pid, &format!("le_{}_{}", left.name(), right.name()))
            }
            OperationKind::LessEqual(LessEqual { left, right }) => {
                stdio.push_asm(engine, pid, &format!("leq_{}_{}", left.name(), right.name()))
            }
            OperationKind::Greater(Greater { left, right }) => {
                stdio.push_asm(engine, pid, &format!("ge_{}_{}", left.name(), right.name()))
            }
            OperationKind::GreaterEqual(GreaterEqual { left, right }) => {
                stdio.push_asm(engine, pid, &format!("geq_{}_{}", left.name(), right.name()))
            }
            OperationKind::Equal(Equal { left, right }) => {
                stdio.push_asm(engine, pid, &format!("eq {}B", left))
            }
            OperationKind::NotEqual(NotEqual { left, right }) => {
                stdio.push_asm(engine, pid, &format!("neq {}B", left))
            }
            OperationKind::LogicalAnd(LogicalAnd()) => stdio.push_asm(engine, pid, &format!("and")),
            OperationKind::LogicalOr(LogicalOr()) => stdio.push_asm(engine, pid, &format!("or")),
            OperationKind::Minus(Minus { data_type }) => {
                stdio.push_asm(engine, pid, &format!("neg_{}", data_type.name()))
            }
            OperationKind::Not(Not()) => stdio.push_asm(engine, pid, "not"),
            OperationKind::StrEqual(_) => stdio.push_asm(engine, pid, &format!("str_eq")),
            OperationKind::StrNotEqual(_) => stdio.push_asm(engine, pid, &format!("str_eq")),
        }
    }
}

impl crate::vm::AsmWeight for Operation {}

#[derive(Debug, Clone)]
pub enum OperationKind {
    Align,
    CastCharToUTF8,
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
    StrEqual(StrEqual),
    NotEqual(NotEqual),
    StrNotEqual(StrNotEqual),
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
            EType::Static(value) => match value {
                StaticType::Primitive(value) => match value {
                    PrimitiveType::Number(value) => Ok(OpPrimitive::Number(value)),
                    PrimitiveType::Char => Ok(OpPrimitive::Char),
                    PrimitiveType::Bool => Ok(OpPrimitive::Bool),
                },
                StaticType::StrSlice(_) => Ok(OpPrimitive::String),
                _ => Err(CodeGenerationError::UnresolvedError),
            },
            EType::User { .. } => Err(CodeGenerationError::UnresolvedError),
        }
    }
}

impl OpPrimitive {
    pub fn name(&self) -> &'static str {
        match self {
            OpPrimitive::Number(num) => match num {
                NumberType::U8 => "u8",
                NumberType::U16 => "u16",
                NumberType::U32 => "u32",
                NumberType::U64 => "u64",
                NumberType::U128 => "u128",
                NumberType::I8 => "i8",
                NumberType::I16 => "i16",
                NumberType::I32 => "i32",
                NumberType::I64 => "i64",
                NumberType::I128 => "i128",
                NumberType::F64 => "f64",
            },
            OpPrimitive::Bool => "bool",
            OpPrimitive::Char => "char",
            OpPrimitive::String => "str",
        }
    }

    pub fn pop_float(memory: &mut Stack) -> Result<f64, RuntimeError> {
        let data = memory.pop(8)?;
        let data =
            TryInto::<&[u8; 8]>::try_into(data).map_err(|_| RuntimeError::Deserialization)?;
        Ok(f64::from_le_bytes(*data))
    }

    pub fn get_float_from(
        address: MemoryAddress,
        stack: &Stack,
        heap: &Heap,
    ) -> Result<f64, RuntimeError> {
        let data = match address {
            MemoryAddress::Heap { .. } => heap.read_slice(address, 8)?,
            MemoryAddress::Stack { .. } => stack.read(address, 8)?,
            MemoryAddress::Global { .. } => stack.read_global(address, 8)?,
            MemoryAddress::Frame { .. } => stack.read_in_frame(address, 8)?,
        };
        let data =
            TryInto::<&[u8; 8]>::try_into(data).map_err(|_| RuntimeError::Deserialization)?;
        Ok(f64::from_le_bytes(*data))
    }

    pub fn pop_bool(memory: &mut Stack) -> Result<bool, RuntimeError> {
        let data = memory.pop(PrimitiveType::Bool.size_of())?;

        Ok(data.first().map_or(false, |byte| *byte != 0))
    }

    pub fn get_bool_from(
        address: MemoryAddress,
        stack: &Stack,
        heap: &Heap,
    ) -> Result<bool, RuntimeError> {
        let data = match address {
            MemoryAddress::Heap { .. } => heap.read_slice(address, 1)?,
            MemoryAddress::Stack { .. } => stack.read(address, 1)?,
            MemoryAddress::Global { .. } => stack.read_global(address, 1)?,
            MemoryAddress::Frame { .. } => stack.read_in_frame(address, 1)?,
        };

        Ok(data.first().map_or(false, |byte| *byte != 0))
    }

    pub fn pop_char(memory: &mut Stack) -> Result<char, RuntimeError> {
        let data = memory.pop(PrimitiveType::Char.size_of())?;
        let data =
            TryInto::<&[u8; 4]>::try_into(data).map_err(|_| RuntimeError::Deserialization)?;

        let chara = std::str::from_utf8(data.as_slice())
            .map_err(|_| RuntimeError::Deserialization)?
            .chars()
            .next()
            .ok_or(RuntimeError::Deserialization)?;
        Ok(chara)
    }

    pub fn get_char_from(
        address: MemoryAddress,
        stack: &Stack,
        heap: &Heap,
    ) -> Result<char, RuntimeError> {
        let data = match address {
            MemoryAddress::Heap { .. } => heap.read_slice(address, 4)?,
            MemoryAddress::Stack { .. } => stack.read(address, 4)?,
            MemoryAddress::Global { .. } => stack.read_global(address, 4)?,
            MemoryAddress::Frame { .. } => stack.read_in_frame(address, 4)?,
        };

        let data =
            TryInto::<&[u8; 4]>::try_into(data).map_err(|_| RuntimeError::Deserialization)?;

        let chara = std::str::from_utf8(data.as_slice())
            .map_err(|_| RuntimeError::Deserialization)?
            .chars()
            .next()
            .ok_or(RuntimeError::Deserialization)?;
        Ok(chara)
    }

    pub fn get_string_from(
        address: MemoryAddress,
        stack: &Stack,
        heap: &Heap,
    ) -> Result<String, RuntimeError> {
        let size = OpPrimitive::get_num_from::<u64>(address, stack, heap)? as usize;
        let data = match address {
            MemoryAddress::Heap { .. } => heap.read_slice(address.add(8), size)?,
            MemoryAddress::Stack { .. } => stack.read(address.add(8), size)?,
            MemoryAddress::Global { .. } => stack.read_global(address.add(8), size)?,
            MemoryAddress::Frame { .. } => stack.read_in_frame(address.add(8), size)?,
        };

        let data = std::str::from_utf8(&data).map_err(|_| RuntimeError::Deserialization)?;
        Ok(data.to_string())
    }
}

pub trait PopNum {
    fn pop_num<T: PrimInt + Debug>(
        stack: &mut crate::vm::allocator::stack::Stack,
    ) -> Result<T, RuntimeError>;
}

fn pop_data<const N: usize>(stack: &mut Stack) -> Result<[u8; N], RuntimeError> {
    let data = stack.pop(N)?;
    data.try_into().map_err(|_| RuntimeError::Deserialization)
}

impl PopNum for crate::vm::asm::operation::OpPrimitive {
    fn pop_num<T: PrimInt + Debug>(
        stack: &mut crate::vm::allocator::stack::Stack,
    ) -> Result<T, RuntimeError> {
        match std::mem::size_of::<T>() {
            1 => {
                let data: [u8; 1] = pop_data(stack)?;
                Ok(T::from(u8::from_le_bytes(data))
                    .or_else(|| T::from(i8::from_le_bytes(data)))
                    .ok_or(RuntimeError::Deserialization)?)
            }
            2 => {
                let data: [u8; 2] = pop_data(stack)?;
                Ok(T::from(u16::from_le_bytes(data))
                    .or_else(|| T::from(i16::from_le_bytes(data)))
                    .ok_or(RuntimeError::Deserialization)?)
            }
            4 => {
                let data: [u8; 4] = pop_data(stack)?;
                Ok(T::from(u32::from_le_bytes(data))
                    .or_else(|| T::from(i32::from_le_bytes(data)))
                    .ok_or(RuntimeError::Deserialization)?)
            }
            8 => {
                let data: [u8; 8] = pop_data(stack)?;
                Ok(T::from(u64::from_le_bytes(data))
                    .or_else(|| T::from(i64::from_le_bytes(data)))
                    .ok_or(RuntimeError::Deserialization)?)
            }
            16 => {
                let data: [u8; 16] = pop_data(stack)?;
                Ok(T::from(u128::from_le_bytes(data))
                    .or_else(|| T::from(i128::from_le_bytes(data)))
                    .ok_or(RuntimeError::Deserialization)?)
            }
            _ => Err(RuntimeError::Deserialization),
        }
    }
}

pub trait GetNumFrom {
    fn get_num_from<T: PrimInt>(
        address: crate::vm::allocator::MemoryAddress,
        stack: &crate::vm::allocator::stack::Stack,
        heap: &crate::vm::allocator::heap::Heap,
    ) -> Result<T, RuntimeError>;
}

fn read_data<const N: usize>(
    address: MemoryAddress,
    stack: &Stack,
    heap: &Heap,
) -> Result<[u8; N], RuntimeError> {
    let vec = match address {
        MemoryAddress::Heap { .. } => heap.read_slice(address, N)?,
        MemoryAddress::Stack { .. } => stack.read(address, N)?,
        MemoryAddress::Global { .. } => stack.read_global(address, N)?,
        MemoryAddress::Frame { .. } => stack.read_in_frame(address, N)?,
    };
    vec.try_into().map_err(|_| RuntimeError::Deserialization)
}

impl GetNumFrom for crate::vm::asm::operation::OpPrimitive {
    fn get_num_from<T: PrimInt>(
        address: crate::vm::allocator::MemoryAddress,
        stack: &crate::vm::allocator::stack::Stack,
        heap: &crate::vm::allocator::heap::Heap,
    ) -> Result<T, RuntimeError> {
        match std::mem::size_of::<T>() {
            1 => {
                let data: [u8; 1] = read_data(address, stack, heap)?;
                Ok(T::from(u8::from_le_bytes(data))
                    .or_else(|| T::from(i8::from_le_bytes(data)))
                    .ok_or(RuntimeError::Deserialization)?)
            }
            2 => {
                let data: [u8; 2] = read_data(address, stack, heap)?;
                Ok(T::from(u16::from_le_bytes(data))
                    .or_else(|| T::from(i16::from_le_bytes(data)))
                    .ok_or(RuntimeError::Deserialization)?)
            }
            4 => {
                let data: [u8; 4] = read_data(address, stack, heap)?;
                Ok(T::from(u32::from_le_bytes(data))
                    .or_else(|| T::from(i32::from_le_bytes(data)))
                    .ok_or(RuntimeError::Deserialization)?)
            }
            8 => {
                let data: [u8; 8] = read_data(address, stack, heap)?;
                Ok(T::from(u64::from_le_bytes(data))
                    .or_else(|| T::from(i64::from_le_bytes(data)))
                    .ok_or(RuntimeError::Deserialization)?)
            }
            16 => {
                let data: [u8; 16] = read_data(address, stack, heap)?;
                Ok(T::from(u128::from_le_bytes(data))
                    .or_else(|| T::from(i128::from_le_bytes(data)))
                    .ok_or(RuntimeError::Deserialization)?)
            }
            _ => Err(RuntimeError::Deserialization),
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for OperationKind {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match self {
            OperationKind::Mult(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::Div(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::Mod(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::Addition(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::Substraction(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::ShiftLeft(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::ShiftRight(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::BitwiseAnd(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::BitwiseXOR(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::BitwiseOR(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::Cast(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::Less(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::LessEqual(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::Greater(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::GreaterEqual(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::Equal(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::NotEqual(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::LogicalAnd(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::LogicalOr(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::Minus(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::Not(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::Align => {
                let num = OpPrimitive::pop_num::<u64>(stack)?;
                let aligned_num = align(num as usize) as u64;
                Ok(stack.push_with(&aligned_num.to_le_bytes())?)
            }
            OperationKind::CastCharToUTF8 => {
                let chara = OpPrimitive::pop_char(stack)?;
                let chara = chara.to_string();
                let chara = chara.as_bytes();
                let _ = stack.push_with(chara)?;
                Ok(stack.push_with(&(chara.len() as u64).to_le_bytes())?)
            }
            OperationKind::StrEqual(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            OperationKind::StrNotEqual(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
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

impl<E: crate::vm::external::Engine> Executable<E> for Mult {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Mult, stack)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for Division {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Div, stack)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for Mod {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Mod, stack)
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

impl<E: crate::vm::external::Engine> Executable<E> for Addition {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Add, stack)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for Substraction {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Sub, stack)
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

impl<E: crate::vm::external::Engine> Executable<E> for ShiftLeft {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::ShiftLeft, stack)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for ShiftRight {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::ShiftRight, stack)
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

impl<E: crate::vm::external::Engine> Executable<E> for BitwiseAnd {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::BitAnd, stack)
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

impl<E: crate::vm::external::Engine> Executable<E> for BitwiseXOR {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::BitXor, stack)
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

impl<E: crate::vm::external::Engine> Executable<E> for BitwiseOR {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::BitOr, stack)
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

impl<E: crate::vm::external::Engine> Executable<E> for Less {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::Less, stack)
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::pop_bool(stack)?;
                let left = OpPrimitive::pop_bool(stack)?;
                Ok(stack.push_with(&[(left < right) as u8])?)
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::pop_char(stack)?;
                let left = OpPrimitive::pop_char(stack)?;
                Ok(stack.push_with(&[(left < right) as u8])?)
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right_address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let left_address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let right = OpPrimitive::get_string_from(right_address, stack, heap)?;
                let left = OpPrimitive::get_string_from(left_address, stack, heap)?;
                Ok(stack.push_with(&[(left < right) as u8])?)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for LessEqual {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::LessEqual, stack)
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::pop_bool(stack)?;
                let left = OpPrimitive::pop_bool(stack)?;
                Ok(stack.push_with(&[(left <= right) as u8])?)
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::pop_char(stack)?;
                let left = OpPrimitive::pop_char(stack)?;
                Ok(stack.push_with(&[(left <= right) as u8])?)
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right_address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let left_address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let right = OpPrimitive::get_string_from(right_address, stack, heap)?;
                let left = OpPrimitive::get_string_from(left_address, stack, heap)?;
                Ok(stack.push_with(&[(left <= right) as u8])?)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for Greater {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::Greater, stack)
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::pop_bool(stack)?;
                let left = OpPrimitive::pop_bool(stack)?;
                Ok(stack.push_with(&[(left < right) as u8])?)
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::pop_char(stack)?;
                let left = OpPrimitive::pop_char(stack)?;
                Ok(stack.push_with(&[(left < right) as u8])?)
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right_address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let left_address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let right = OpPrimitive::get_string_from(right_address, stack, heap)?;
                let left = OpPrimitive::get_string_from(left_address, stack, heap)?;
                Ok(stack.push_with(&[(left > right) as u8])?)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for GreaterEqual {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::GreaterEqual, stack)
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::pop_bool(stack)?;
                let left = OpPrimitive::pop_bool(stack)?;
                Ok(stack.push_with(&[(left < right) as u8])?)
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::pop_char(stack)?;
                let left = OpPrimitive::pop_char(stack)?;
                Ok(stack.push_with(&[(left < right) as u8])?)
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right_address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let left_address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let right = OpPrimitive::get_string_from(right_address, stack, heap)?;
                let left = OpPrimitive::get_string_from(left_address, stack, heap)?;
                Ok(stack.push_with(&[(left >= right) as u8])?)
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

#[derive(Debug, Clone)]
pub struct StrEqual;

#[derive(Debug, Clone)]
pub struct StrNotEqual;

impl<E: crate::vm::external::Engine> Executable<E> for Equal {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        let right_data = stack.pop(self.right)?.to_owned();

        let left_data = stack.pop(self.left)?;

        let data = [(left_data == right_data) as u8];
        Ok(stack.push_with(&data)?)
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for StrEqual {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        let right_address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
        let left_address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

        let left = OpPrimitive::get_string_from(left_address, stack, heap)?;
        let right = OpPrimitive::get_string_from(right_address, stack, heap)?;

        let data = [(left == right) as u8];

        Ok(stack.push_with(&data)?)
    }
}
impl<E: crate::vm::external::Engine> Executable<E> for NotEqual {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        let right_data = stack.pop(self.right)?.to_owned();

        let left_data = stack.pop(self.left)?.to_owned();

        let data = [(left_data != right_data) as u8];

        Ok(stack.push_with(&data)?)
    }
}
impl<E: crate::vm::external::Engine> Executable<E> for StrNotEqual {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        let left_address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
        let right_address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

        let left = OpPrimitive::get_string_from(left_address, stack, heap)?;
        let right = OpPrimitive::get_string_from(right_address, stack, heap)?;
        let data = [(left != right) as u8];

        Ok(stack.push_with(&data)?)
    }
}
#[derive(Debug, Clone)]
pub struct LogicalAnd();

impl<E: crate::vm::external::Engine> Executable<E> for LogicalAnd {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        let right_data = OpPrimitive::pop_bool(stack)?;
        let left_data = OpPrimitive::pop_bool(stack)?;
        let data = [(left_data && right_data) as u8];
        Ok(stack.push_with(&data)?)
    }
}

#[derive(Debug, Clone)]
pub struct LogicalOr();

impl<E: crate::vm::external::Engine> Executable<E> for LogicalOr {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        let right_data = OpPrimitive::pop_bool(stack)?;
        let left_data = OpPrimitive::pop_bool(stack)?;
        let data = [(left_data || right_data) as u8];
        Ok(stack.push_with(&data)?)
    }
}

#[derive(Debug, Clone)]
pub struct Minus {
    pub data_type: OpPrimitive,
}

impl<E: crate::vm::external::Engine> Executable<E> for Minus {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match &self.data_type {
            OpPrimitive::Number(number) => match number {
                NumberType::U8 => {
                    let data = OpPrimitive::pop_num::<u8>(stack)? as i16;
                    Ok(stack.push_with(&(-data).to_le_bytes())?)
                }
                NumberType::U16 => {
                    let data = OpPrimitive::pop_num::<u16>(stack)? as i32;
                    Ok(stack.push_with(&(-data).to_le_bytes())?)
                }
                NumberType::U32 => {
                    let data = OpPrimitive::pop_num::<u32>(stack)? as i64;
                    Ok(stack.push_with(&(-data).to_le_bytes())?)
                }
                NumberType::U64 => {
                    let data = OpPrimitive::pop_num::<u64>(stack)? as i128;
                    Ok(stack.push_with(&(-data).to_le_bytes())?)
                }
                NumberType::U128 => {
                    let data = OpPrimitive::pop_num::<u128>(stack)? as i128;
                    Ok(stack.push_with(&(-data).to_le_bytes())?)
                }
                NumberType::I8 => {
                    let data = OpPrimitive::pop_num::<i8>(stack)?;
                    Ok(stack.push_with(&(-data).to_le_bytes())?)
                }
                NumberType::I16 => {
                    let data = OpPrimitive::pop_num::<i16>(stack)?;
                    Ok(stack.push_with(&(-data).to_le_bytes())?)
                }
                NumberType::I32 => {
                    let data = OpPrimitive::pop_num::<i32>(stack)?;
                    Ok(stack.push_with(&(-data).to_le_bytes())?)
                }
                NumberType::I64 => {
                    let data = OpPrimitive::pop_num::<i64>(stack)?;
                    Ok(stack.push_with(&(-data).to_le_bytes())?)
                }
                NumberType::I128 => {
                    let data = OpPrimitive::pop_num::<i128>(stack)?;
                    Ok(stack.push_with(&(-data).to_le_bytes())?)
                }
                NumberType::F64 => {
                    let data = OpPrimitive::pop_float(stack)?;
                    Ok(stack.push_with(&(-data).to_le_bytes())?)
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

impl<E: crate::vm::external::Engine> Executable<E> for Not {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        let data = OpPrimitive::pop_bool(stack)?;
        let data = [(!data) as u8];
        Ok(stack.push_with(&data)?)
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
            NumberType::U8 => $memory.push_with(&($data as u8).to_le_bytes()),
            NumberType::U16 => $memory.push_with(&($data as u16).to_le_bytes()),
            NumberType::U32 => $memory.push_with(&($data as u32).to_le_bytes()),
            NumberType::U64 => $memory.push_with(&($data as u64).to_le_bytes()),
            NumberType::U128 => $memory.push_with(&($data as u128).to_le_bytes()),
            NumberType::I8 => $memory.push_with(&($data as i8).to_le_bytes()),
            NumberType::I16 => $memory.push_with(&($data as i16).to_le_bytes()),
            NumberType::I32 => $memory.push_with(&($data as i32).to_le_bytes()),
            NumberType::I64 => $memory.push_with(&($data as i64).to_le_bytes()),
            NumberType::I128 => $memory.push_with(&($data as i128).to_le_bytes()),
            NumberType::F64 => $memory.push_with(&($data as f64).to_le_bytes()),
        }
    };
}

impl<E: crate::vm::external::Engine> Executable<E> for Cast {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match (self.from, self.to) {
            (OpPrimitive::Number(number), OpPrimitive::Number(to)) => match number {
                NumberType::U8 => {
                    let data = OpPrimitive::pop_num::<u8>(stack)?;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::U16 => {
                    let data = OpPrimitive::pop_num::<u16>(stack)?;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::U32 => {
                    let data = OpPrimitive::pop_num::<u32>(stack)?;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::U64 => {
                    let data = OpPrimitive::pop_num::<u64>(stack)?;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::U128 => {
                    let data = OpPrimitive::pop_num::<u128>(stack)?;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::I8 => {
                    let data = OpPrimitive::pop_num::<i8>(stack)?;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::I16 => {
                    let data = OpPrimitive::pop_num::<i16>(stack)?;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::I32 => {
                    let data = OpPrimitive::pop_num::<i32>(stack)?;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::I64 => {
                    let data = OpPrimitive::pop_num::<i64>(stack)?;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::I128 => {
                    let data = OpPrimitive::pop_num::<i128>(stack)?;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::F64 => {
                    let data = OpPrimitive::pop_float(stack)?;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
            },
            (OpPrimitive::Number(number), OpPrimitive::Bool) => match number {
                NumberType::U8 => {
                    let data = OpPrimitive::pop_num::<u8>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::U16 => {
                    let data = OpPrimitive::pop_num::<u16>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::U32 => {
                    let data = OpPrimitive::pop_num::<u32>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::U64 => {
                    let data = OpPrimitive::pop_num::<u64>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::U128 => {
                    let data = OpPrimitive::pop_num::<u128>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::I8 => {
                    let data = OpPrimitive::pop_num::<i8>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::I16 => {
                    let data = OpPrimitive::pop_num::<i16>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::I32 => {
                    let data = OpPrimitive::pop_num::<i32>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::I64 => {
                    let data = OpPrimitive::pop_num::<i64>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::I128 => {
                    let data = OpPrimitive::pop_num::<i128>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::F64 => {
                    let data = OpPrimitive::pop_float(stack)?;
                    Ok(stack.push_with(&[(data == 0.0) as u8])?)
                }
            },
            (OpPrimitive::Number(NumberType::U8), OpPrimitive::Char) => {
                Err(RuntimeError::UnsupportedOperation)
            }
            (OpPrimitive::Number(_), OpPrimitive::Char) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Number(_), OpPrimitive::String) => {
                Err(RuntimeError::UnsupportedOperation)
            }
            (OpPrimitive::Bool, OpPrimitive::Number(number)) => {
                let data = OpPrimitive::pop_num::<u8>(stack)? as u8;
                match number {
                    NumberType::U8 => Ok(stack.push_with(&data.to_le_bytes())?),
                    NumberType::U16 => Ok(stack.push_with(&(data as u16).to_le_bytes())?),
                    NumberType::U32 => Ok(stack.push_with(&(data as u32).to_le_bytes())?),
                    NumberType::U64 => Ok(stack.push_with(&(data as u64).to_le_bytes())?),
                    NumberType::U128 => Ok(stack.push_with(&(data as u128).to_le_bytes())?),
                    NumberType::I8 => Ok(stack.push_with(&(data as i8).to_le_bytes())?),
                    NumberType::I16 => Ok(stack.push_with(&(data as i16).to_le_bytes())?),
                    NumberType::I32 => Ok(stack.push_with(&(data as i32).to_le_bytes())?),
                    NumberType::I64 => Ok(stack.push_with(&(data as i64).to_le_bytes())?),
                    NumberType::I128 => Ok(stack.push_with(&(data as i128).to_le_bytes())?),
                    NumberType::F64 => Ok(stack.push_with(&(data as f64).to_le_bytes())?),
                }
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => Ok(()),
            (OpPrimitive::Bool, OpPrimitive::Char) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Bool, OpPrimitive::String) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Char, OpPrimitive::Number(number)) => {
                let data = OpPrimitive::pop_num::<u32>(stack)?;
                match number {
                    NumberType::U8 => Ok(stack.push_with(&(data as u8).to_le_bytes())?),
                    NumberType::U16 => Ok(stack.push_with(&(data as u16).to_le_bytes())?),
                    NumberType::U32 => Ok(stack.push_with(&(data as u32).to_le_bytes())?),
                    NumberType::U64 => Ok(stack.push_with(&(data as u64).to_le_bytes())?),
                    NumberType::U128 => Ok(stack.push_with(&(data as u128).to_le_bytes())?),
                    _ => Err(RuntimeError::UnsupportedOperation),
                }
            }
            (OpPrimitive::Char, OpPrimitive::Bool) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Char, OpPrimitive::Char) => Err(RuntimeError::UnsupportedOperation),
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
