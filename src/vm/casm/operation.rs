use crate::{
    semantic::{
        scope::static_types::{st_deserialize::extract_u64, NumberType, PrimitiveType, StaticType},
        EType, Either, SizeOf,
    },
    vm::{
        allocator::{align, heap::Heap, stack::Stack},
        stdio::StdIO,
        vm::{CasmMetadata, CodeGenerationError, Executable, RuntimeError},
    },
};
use nom::AsBytes;
use num_traits::{FromBytes, ToBytes};

use super::{
    math_operation::{comparaison_operator, math_operator, ComparaisonOperator, MathOperator},
    CasmProgram,
};

#[derive(Debug, Clone)]
pub struct Operation {
    pub kind: OperationKind,
    // pub result: OpPrimitive,
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Operation {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        let _ = self.kind.execute(program, stack, heap, stdio, engine)?;
        program.incr();
        Ok(())
    }
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Operation {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self.kind {
            OperationKind::Align => stdio.push_casm(engine, "align"),
            OperationKind::CastCharToUTF8 => stdio.push_casm(engine, "char_to_utf8"),
            OperationKind::Mult(Mult { left, right }) => {
                stdio.push_casm(engine, &format!("mult_{}_{}", left.name(), right.name()))
            }
            OperationKind::Div(Division { left, right }) => {
                stdio.push_casm(engine, &format!("div_{}_{}", left.name(), right.name()))
            }
            OperationKind::Mod(Mod { left, right }) => {
                stdio.push_casm(engine, &format!("mod_{}_{}", left.name(), right.name()))
            }
            OperationKind::Addition(Addition { left, right }) => {
                stdio.push_casm(engine, &format!("add_{}_{}", left.name(), right.name()))
            }
            OperationKind::Substraction(Substraction { left, right }) => {
                stdio.push_casm(engine, &format!("sub_{}_{}", left.name(), right.name()))
            }
            OperationKind::ShiftLeft(ShiftLeft { left, right }) => {
                stdio.push_casm(engine, &format!("shl_{}_{}", left.name(), right.name()))
            }
            OperationKind::ShiftRight(ShiftRight { left, right }) => {
                stdio.push_casm(engine, &format!("shr_{}_{}", left.name(), right.name()))
            }
            OperationKind::BitwiseAnd(BitwiseAnd { left, right }) => {
                stdio.push_casm(engine, &format!("band_{}_{}", left.name(), right.name()))
            }
            OperationKind::BitwiseXOR(BitwiseXOR { left, right }) => {
                stdio.push_casm(engine, &format!("bxor_{}_{}", left.name(), right.name()))
            }
            OperationKind::BitwiseOR(BitwiseOR { left, right }) => {
                stdio.push_casm(engine, &format!("bor_{}_{}", left.name(), right.name()))
            }
            OperationKind::Cast(Cast { from, to }) => {
                stdio.push_casm(engine, &format!("cast_{}_{}", from.name(), to.name()))
            }
            OperationKind::Less(Less { left, right }) => {
                stdio.push_casm(engine, &format!("le_{}_{}", left.name(), right.name()))
            }
            OperationKind::LessEqual(LessEqual { left, right }) => {
                stdio.push_casm(engine, &format!("leq_{}_{}", left.name(), right.name()))
            }
            OperationKind::Greater(Greater { left, right }) => {
                stdio.push_casm(engine, &format!("ge_{}_{}", left.name(), right.name()))
            }
            OperationKind::GreaterEqual(GreaterEqual { left, right }) => {
                stdio.push_casm(engine, &format!("geq_{}_{}", left.name(), right.name()))
            }
            OperationKind::Equal(Equal { left, right }) => {
                stdio.push_casm(engine, &format!("eq {}B", left))
            }
            OperationKind::NotEqual(NotEqual { left, right }) => {
                stdio.push_casm(engine, &format!("neq {}B", left))
            }
            OperationKind::LogicalAnd(LogicalAnd()) => stdio.push_casm(engine, &format!("and")),
            OperationKind::LogicalOr(LogicalOr()) => stdio.push_casm(engine, &format!("or")),
            OperationKind::Minus(Minus { data_type }) => {
                stdio.push_casm(engine, &format!("neg_{}", data_type.name()))
            }
            OperationKind::Not(Not()) => stdio.push_casm(engine, "not"),
        }
    }
}
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
    NotEqual(NotEqual),
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
                StaticType::StrSlice(StrSliceType) => Ok(OpPrimitive::String),
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
    //         ?;

    //     let data = TryInto::<&[u8; 8]>::try_into(data.as_slice())
    //         .map_err(|_| RuntimeError::Deserialization)?;
    //     Ok(f64::from_le_bytes(*data))
    // }

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

    pub fn get_num16<N: FromBytes<Bytes = [u8; 16]>>(
        memory: &mut Stack,
    ) -> Result<N, RuntimeError> {
        let data = memory.pop(16)?;
        let data =
            TryInto::<&[u8; 16]>::try_into(data).map_err(|_| RuntimeError::Deserialization)?;
        Ok(N::from_le_bytes(data))
    }
    pub fn get_num8<N: FromBytes<Bytes = [u8; 8]>>(memory: &mut Stack) -> Result<N, RuntimeError> {
        let data = memory.pop(8)?;
        let data =
            TryInto::<&[u8; 8]>::try_into(data).map_err(|_| RuntimeError::Deserialization)?;
        Ok(N::from_le_bytes(data))
    }
    pub fn get_num4<N: FromBytes<Bytes = [u8; 4]>>(memory: &mut Stack) -> Result<N, RuntimeError> {
        let data = memory.pop(4)?;
        let data =
            TryInto::<&[u8; 4]>::try_into(data).map_err(|_| RuntimeError::Deserialization)?;
        Ok(N::from_le_bytes(data))
    }
    pub fn get_num2<N: FromBytes<Bytes = [u8; 2]>>(memory: &mut Stack) -> Result<N, RuntimeError> {
        let data = memory.pop(2)?;
        let data =
            TryInto::<&[u8; 2]>::try_into(data).map_err(|_| RuntimeError::Deserialization)?;
        Ok(N::from_le_bytes(data))
    }
    pub fn get_num1<N: FromBytes<Bytes = [u8; 1]>>(memory: &mut Stack) -> Result<N, RuntimeError> {
        let data = memory.pop(1)?;
        let data =
            TryInto::<&[u8; 1]>::try_into(data).map_err(|_| RuntimeError::Deserialization)?;
        Ok(N::from_le_bytes(data))
    }

    pub fn get_bool(memory: &mut Stack) -> Result<bool, RuntimeError> {
        let data = memory
            .pop(PrimitiveType::Bool.size_of())
            ?;

        Ok(data.first().map_or(false, |byte| *byte != 0))
    }
    pub fn get_char(memory: &mut Stack) -> Result<char, RuntimeError> {
        let data = memory
            .pop(PrimitiveType::Char.size_of())
            ?;
        let data =
            TryInto::<&[u8; 4]>::try_into(data).map_err(|_| RuntimeError::Deserialization)?;

        let chara = std::str::from_utf8(data.as_slice())
            .map_err(|_| RuntimeError::Deserialization)?
            .chars()
            .next()
            .ok_or(RuntimeError::Deserialization)?;
        Ok(chara)
    }
    pub fn get_str_slice(memory: &mut Stack) -> Result<String, RuntimeError> {
        let len = OpPrimitive::get_num8::<u64>(memory)? as usize;
        let data = memory.pop(len)?;
        let data = std::str::from_utf8(&data).map_err(|_| RuntimeError::Deserialization)?;
        Ok(data.to_string())
    }
    pub fn get_string(stack: &mut Stack, heap: &mut Heap) -> Result<String, RuntimeError> {
        let heap_address = OpPrimitive::get_num8::<u64>(stack)?;
        let data = heap
            .read(heap_address as usize, 16)
            .expect("Heap Read should have succeeded");
        let (length, rest) = extract_u64(&data)?;
        let (_capacity, _rest) = extract_u64(rest)?;
        let data = heap
            .read(heap_address as usize + 16, length as usize)
            .expect("Heap Read should have succeeded");
        let data = std::str::from_utf8(&data).map_err(|_| RuntimeError::Deserialization)?;
        Ok(data.to_string())
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for OperationKind {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match self {
            OperationKind::Mult(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::Div(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::Mod(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::Addition(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::Substraction(value) => {
                value.execute(program, stack, heap, stdio, engine)
            }
            OperationKind::ShiftLeft(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::ShiftRight(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::BitwiseAnd(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::BitwiseXOR(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::BitwiseOR(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::Cast(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::Less(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::LessEqual(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::Greater(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::GreaterEqual(value) => {
                value.execute(program, stack, heap, stdio, engine)
            }
            OperationKind::Equal(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::NotEqual(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::LogicalAnd(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::LogicalOr(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::Minus(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::Not(value) => value.execute(program, stack, heap, stdio, engine),
            OperationKind::Align => {
                let num = OpPrimitive::get_num8::<u64>(stack)?;
                let aligned_num = align(num as usize) as u64;
                Ok(stack
                    .push_with(&aligned_num.to_le_bytes())?)
                    
            }
            OperationKind::CastCharToUTF8 => {
                let chara = OpPrimitive::get_char(stack)?;
                let chara = chara.to_string();
                let chara = chara.as_bytes();
                let _ = stack.push_with(chara)?;
                Ok(stack
                    .push_with(&(chara.len() as u64).to_le_bytes())?)
                    
            }
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

impl<G: crate::GameEngineStaticFn> Executable<G> for Mult {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Mult, stack)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Division {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Div, stack)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Mod {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
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

impl<G: crate::GameEngineStaticFn> Executable<G> for Addition {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::Add, stack)
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right = OpPrimitive::get_str_slice(stack)?;
                let left = OpPrimitive::get_str_slice(stack)?;

                let str_bytes: Vec<u8> = (left.to_owned() + &right).into_bytes();

                stack
                    .push_with(str_bytes.as_slice())
                    ?;

                Ok(stack
                    .push_with(&(str_bytes.len() as u64).to_le_bytes())?)
                    
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Substraction {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
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

impl<G: crate::GameEngineStaticFn> Executable<G> for ShiftLeft {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                math_operator(&left, &right, MathOperator::ShiftLeft, stack)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for ShiftRight {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
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

impl<G: crate::GameEngineStaticFn> Executable<G> for BitwiseAnd {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
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

impl<G: crate::GameEngineStaticFn> Executable<G> for BitwiseXOR {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
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

impl<G: crate::GameEngineStaticFn> Executable<G> for BitwiseOR {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
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

impl<G: crate::GameEngineStaticFn> Executable<G> for Less {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::Less, stack)
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::get_bool(stack)?;
                let left = OpPrimitive::get_bool(stack)?;
                Ok(stack
                    .push_with(&[(left < right) as u8])?)
                    
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::get_char(stack)?;
                let left = OpPrimitive::get_char(stack)?;
                Ok(stack
                    .push_with(&[(left < right) as u8])?)
                    
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right = OpPrimitive::get_str_slice(stack)?;
                let left = OpPrimitive::get_str_slice(stack)?;
                Ok(stack
                    .push_with(&[(left < right) as u8])?)
                    
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for LessEqual {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::LessEqual, stack)
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::get_bool(stack)?;
                let left = OpPrimitive::get_bool(stack)?;
                Ok(stack
                    .push_with(&[(left < right) as u8])?)
                    
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::get_char(stack)?;
                let left = OpPrimitive::get_char(stack)?;
                Ok(stack
                    .push_with(&[(left < right) as u8])?)
                    
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right = OpPrimitive::get_str_slice(stack)?;
                let left = OpPrimitive::get_str_slice(stack)?;
                Ok(stack
                    .push_with(&[(left < right) as u8])?)
                    
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Greater {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::Greater, stack)
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::get_bool(stack)?;
                let left = OpPrimitive::get_bool(stack)?;
                Ok(stack
                    .push_with(&[(left < right) as u8])?)
                    
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::get_char(stack)?;
                let left = OpPrimitive::get_char(stack)?;
                Ok(stack
                    .push_with(&[(left < right) as u8])?)
                    
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right = OpPrimitive::get_str_slice(stack)?;
                let left = OpPrimitive::get_str_slice(stack)?;
                Ok(stack
                    .push_with(&[(left < right) as u8])?)
                    
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for GreaterEqual {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match (self.left, self.right) {
            (OpPrimitive::Number(left), OpPrimitive::Number(right)) => {
                comparaison_operator(&left, &right, ComparaisonOperator::GreaterEqual, stack)
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => {
                let right = OpPrimitive::get_bool(stack)?;
                let left = OpPrimitive::get_bool(stack)?;
                Ok(stack
                    .push_with(&[(left < right) as u8])?)
                    
            }
            (OpPrimitive::Char, OpPrimitive::Char) => {
                let right = OpPrimitive::get_char(stack)?;
                let left = OpPrimitive::get_char(stack)?;
                Ok(stack
                    .push_with(&[(left < right) as u8])?)
                    
            }
            (OpPrimitive::String, OpPrimitive::String) => {
                let right = OpPrimitive::get_str_slice(stack)?;
                let left = OpPrimitive::get_str_slice(stack)?;
                Ok(stack
                    .push_with(&[(left < right) as u8])?)
                    
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

impl<G: crate::GameEngineStaticFn> Executable<G> for Equal {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        let data = {
            let right_data = stack.pop(self.right)?.to_owned();

            let left_data = stack.pop(self.left)?;

            [(left_data == right_data) as u8]
        };
        Ok(stack.push_with(&data)?)
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for NotEqual {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        let data = {
            let right_data = stack.pop(self.right)?.to_owned();

            let left_data = stack.pop(self.left)?.to_owned();

            [(left_data != right_data) as u8]
        };
        Ok(stack.push_with(&data)?)
    }
}

#[derive(Debug, Clone)]
pub struct LogicalAnd();

impl<G: crate::GameEngineStaticFn> Executable<G> for LogicalAnd {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        let right_data = OpPrimitive::get_bool(stack)?;
        let left_data = OpPrimitive::get_bool(stack)?;
        let data = [(left_data && right_data) as u8];
        Ok(stack.push_with(&data)?)
    }
}

#[derive(Debug, Clone)]
pub struct LogicalOr();

impl<G: crate::GameEngineStaticFn> Executable<G> for LogicalOr {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        let right_data = OpPrimitive::get_bool(stack)?;
        let left_data = OpPrimitive::get_bool(stack)?;
        let data = [(left_data || right_data) as u8];
        Ok(stack.push_with(&data)?)
    }
}

#[derive(Debug, Clone)]
pub struct Minus {
    pub data_type: OpPrimitive,
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Minus {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match &self.data_type {
            OpPrimitive::Number(number) => match number {
                NumberType::U8 => {
                    let data = OpPrimitive::get_num1::<u8>(stack)? as i16;
                    Ok(stack
                        .push_with(&(-data).to_le_bytes())?)
                        
                }
                NumberType::U16 => {
                    let data = OpPrimitive::get_num2::<u16>(stack)? as i32;
                    Ok(stack
                        .push_with(&(-data).to_le_bytes())?)
                        
                }
                NumberType::U32 => {
                    let data = OpPrimitive::get_num4::<u32>(stack)? as i64;
                    Ok(stack
                        .push_with(&(-data).to_le_bytes())?)
                        
                }
                NumberType::U64 => {
                    let data = OpPrimitive::get_num8::<u64>(stack)? as i128;
                    Ok(stack
                        .push_with(&(-data).to_le_bytes())?)
                        
                }
                NumberType::U128 => {
                    let data = OpPrimitive::get_num16::<u128>(stack)? as i128;
                    Ok(stack
                        .push_with(&(-data).to_le_bytes())?)
                        
                }
                NumberType::I8 => {
                    let data = OpPrimitive::get_num1::<i8>(stack)?;
                    Ok(stack
                        .push_with(&(-data).to_le_bytes())?)
                        
                }
                NumberType::I16 => {
                    let data = OpPrimitive::get_num2::<i16>(stack)?;
                    Ok(stack
                        .push_with(&(-data).to_le_bytes())?)
                        
                }
                NumberType::I32 => {
                    let data = OpPrimitive::get_num4::<i32>(stack)?;
                    Ok(stack
                        .push_with(&(-data).to_le_bytes())?)
                        
                }
                NumberType::I64 => {
                    let data = OpPrimitive::get_num8::<i64>(stack)?;
                    Ok(stack
                        .push_with(&(-data).to_le_bytes())?)
                        
                }
                NumberType::I128 => {
                    let data = OpPrimitive::get_num16::<i128>(stack)?;
                    Ok(stack
                        .push_with(&(-data).to_le_bytes())?)
                        
                }
                NumberType::F64 => {
                    let data = OpPrimitive::get_num8::<f64>(stack)?;
                    Ok(stack
                        .push_with(&(-data).to_le_bytes())?)
                        
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

impl<G: crate::GameEngineStaticFn> Executable<G> for Not {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        let data = OpPrimitive::get_bool(stack)?;
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
            NumberType::U8 => $memory
                .push_with(&($data as u8).to_le_bytes())
                ,
            NumberType::U16 => $memory
                .push_with(&($data as u16).to_le_bytes())
                ,
            NumberType::U32 => $memory
                .push_with(&($data as u32).to_le_bytes())
                ,
            NumberType::U64 => $memory
                .push_with(&($data as u64).to_le_bytes())
                ,
            NumberType::U128 => $memory
                .push_with(&($data as u128).to_le_bytes())
                ,
            NumberType::I8 => $memory
                .push_with(&($data as i8).to_le_bytes())
                ,
            NumberType::I16 => $memory
                .push_with(&($data as i16).to_le_bytes())
                ,
            NumberType::I32 => $memory
                .push_with(&($data as i32).to_le_bytes())
                ,
            NumberType::I64 => $memory
                .push_with(&($data as i64).to_le_bytes())
                ,
            NumberType::I128 => $memory
                .push_with(&($data as i128).to_le_bytes())
                ,
            NumberType::F64 => $memory
                .push_with(&($data as f64).to_le_bytes())
                ,
        }
    };
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Cast {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match (self.from, self.to) {
            (OpPrimitive::Number(number), OpPrimitive::Number(to)) => match number {
                NumberType::U8 => {
                    let data = OpPrimitive::get_num1::<u8>(stack)?;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::U16 => {
                    let data = OpPrimitive::get_num2::<u16>(stack)? as f64;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::U32 => {
                    let data = OpPrimitive::get_num4::<u32>(stack)? as f64;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::U64 => {
                    let data = OpPrimitive::get_num8::<u64>(stack)? as f64;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::U128 => {
                    let data = OpPrimitive::get_num16::<u128>(stack)? as f64;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::I8 => {
                    let data = OpPrimitive::get_num1::<i8>(stack)? as f64;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::I16 => {
                    let data = OpPrimitive::get_num2::<i16>(stack)? as f64;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::I32 => {
                    let data = OpPrimitive::get_num4::<i32>(stack)? as f64;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::I64 => {
                    let data = OpPrimitive::get_num8::<i64>(stack)? as f64;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::I128 => {
                    let data = OpPrimitive::get_num16::<i128>(stack)? as f64;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
                NumberType::F64 => {
                    let data = OpPrimitive::get_num8::<f64>(stack)? as f64;
                    Ok(push_data_as_type!(data, to, stack)?)
                }
            },
            (OpPrimitive::Number(number), OpPrimitive::Bool) => match number {
                NumberType::U8 => {
                    let data = OpPrimitive::get_num1::<u8>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::U16 => {
                    let data = OpPrimitive::get_num2::<u16>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::U32 => {
                    let data = OpPrimitive::get_num4::<u32>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::U64 => {
                    let data = OpPrimitive::get_num8::<u64>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::U128 => {
                    let data = OpPrimitive::get_num16::<u128>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::I8 => {
                    let data = OpPrimitive::get_num1::<i8>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::I16 => {
                    let data = OpPrimitive::get_num2::<i16>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::I32 => {
                    let data = OpPrimitive::get_num4::<i32>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::I64 => {
                    let data = OpPrimitive::get_num8::<i64>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::I128 => {
                    let data = OpPrimitive::get_num16::<i128>(stack)?;
                    Ok(stack.push_with(&[(data != 0) as u8])?)
                }
                NumberType::F64 => {
                    let data = OpPrimitive::get_num8::<f64>(stack)?;
                    Ok(stack
                        .push_with(&[(data == 0.0) as u8])?)
                        
                }
            },
            (OpPrimitive::Number(NumberType::U8), OpPrimitive::Char) => Ok(()),
            (OpPrimitive::Number(_), OpPrimitive::Char) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Number(_), OpPrimitive::String) => {
                Err(RuntimeError::UnsupportedOperation)
            }
            (OpPrimitive::Bool, OpPrimitive::Number(number)) => {
                let data = OpPrimitive::get_num1::<u8>(stack)? as u8;
                match number {
                    NumberType::U8 => Ok(stack.push_with(&data.to_le_bytes())?),
                    NumberType::U16 => Ok(stack
                        .push_with(&(data as u16).to_le_bytes())?)
                        ,
                    NumberType::U32 => Ok(stack
                        .push_with(&(data as u32).to_le_bytes())?)
                        ,
                    NumberType::U64 => Ok(stack
                        .push_with(&(data as u64).to_le_bytes())?)
                        ,
                    NumberType::U128 => Ok(stack
                        .push_with(&(data as u128).to_le_bytes())?)
                        ,
                    NumberType::I8 => Ok(stack
                        .push_with(&(data as i8).to_le_bytes())?)
                        ,
                    NumberType::I16 => Ok(stack
                        .push_with(&(data as i16).to_le_bytes())?)
                        ,
                    NumberType::I32 => Ok(stack
                        .push_with(&(data as i32).to_le_bytes())?)
                        ,
                    NumberType::I64 => Ok(stack
                        .push_with(&(data as i64).to_le_bytes())?)
                        ,
                    NumberType::I128 => Ok(stack
                        .push_with(&(data as i128).to_le_bytes())?)
                        ,
                    NumberType::F64 => Ok(stack
                        .push_with(&(data as f64).to_le_bytes())?)
                        ,
                }
            }
            (OpPrimitive::Bool, OpPrimitive::Bool) => Ok(()),
            (OpPrimitive::Bool, OpPrimitive::Char) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Bool, OpPrimitive::String) => Err(RuntimeError::UnsupportedOperation),
            (OpPrimitive::Char, OpPrimitive::Number(number)) => {
                let data = OpPrimitive::get_num4::<u32>(stack)?;
                match number {
                    NumberType::U8 => Ok(stack
                        .push_with(&(data as u8).to_le_bytes())?)
                        ,
                    NumberType::U16 => Ok(stack
                        .push_with(&(data as u16).to_le_bytes())?)
                        ,
                    NumberType::U32 => Ok(stack
                        .push_with(&(data as u32).to_le_bytes())?)
                        ,
                    NumberType::U64 => Ok(stack
                        .push_with(&(data as u64).to_le_bytes())?)
                        ,
                    NumberType::U128 => Ok(stack
                        .push_with(&(data as u128).to_le_bytes())?)
                        ,
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

#[cfg(test)]
mod tests {
    use crate::{semantic::scope::scope::Scope, vm::vm::Runtime};

    use super::*;

    fn init_float(num: f64, memory: &mut Stack) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let _ = memory.push_with(&data)?;
        Ok(())
    }

    fn init_num1<T: num_traits::ToBytes<Bytes = [u8; 1]>>(
        num: T,
        memory: &mut Stack,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let _ = memory.push_with(&data)?;
        Ok(())
    }

    fn init_num2<T: num_traits::ToBytes<Bytes = [u8; 2]>>(
        num: T,
        memory: &mut Stack,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let _ = memory.push_with(&data)?;
        Ok(())
    }

    fn init_num4<T: num_traits::ToBytes<Bytes = [u8; 4]>>(
        num: T,
        memory: &mut Stack,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let _ = memory.push_with(&data)?;
        Ok(())
    }

    fn init_num8<T: num_traits::ToBytes<Bytes = [u8; 8]>>(
        num: T,
        memory: &mut Stack,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let _ = memory.push_with(&data)?;
        Ok(())
    }

    fn init_num16<T: num_traits::ToBytes<Bytes = [u8; 16]>>(
        num: T,
        memory: &mut Stack,
    ) -> Result<(), RuntimeError> {
        let data = num.to_le_bytes().to_vec();
        let _ = memory.push_with(&data)?;
        Ok(())
    }
    fn init_char(memory: &mut Stack) -> Result<(), RuntimeError> {
        let data = vec!['a' as u8];
        let _ = memory.push_with(&data)?;
        Ok(())
    }

    fn init_bool(state: bool, memory: &mut Stack) -> Result<(), RuntimeError> {
        let data = vec![state as u8];
        let _ = memory.push_with(&data)?;
        Ok(())
    }

    fn init_string(text: &str, memory: &mut Stack) -> Result<(), RuntimeError> {
        let data = text.as_bytes().to_vec();
        let _ = memory.push_with(&data)?;
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
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");

        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(20u32, stack).expect("init should have succeeded");
        Mult {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(stack).expect("result should be of valid type");
        assert_eq!(10 * 20, res);
    }

    #[test]
    fn valid_div() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(2u32, stack).expect("init should have succeeded");
        Division {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(stack).expect("result should be of valid type");
        assert_eq!(10 / 2, res);
    }

    #[test]
    fn valid_mod() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(2u32, stack).expect("init should have succeeded");
        Mod {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(stack).expect("result should be of valid type");
        assert_eq!(10 % 2, res);
    }

    #[test]
    fn valid_add() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(20u32, stack).expect("init should have succeeded");
        Addition {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(stack).expect("result should be of valid type");
        assert_eq!(10 + 20, res);
    }

    #[test]
    fn valid_sub() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(5u32, stack).expect("init should have succeeded");
        Substraction {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(stack).expect("result should be of valid type");
        assert_eq!(10 - 5, res);
    }

    #[test]
    fn valid_sl() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(5u32, stack).expect("init should have succeeded");
        ShiftLeft {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(stack).expect("result should be of valid type");
        assert_eq!(10 << 5, res);
    }

    #[test]
    fn valid_sr() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(2u32, stack).expect("init should have succeeded");
        ShiftRight {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(stack).expect("result should be of valid type");
        assert_eq!(10 >> 2, res);
    }

    #[test]
    fn valid_bitand() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(5u32, stack).expect("init should have succeeded");
        BitwiseAnd {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(stack).expect("result should be of valid type");
        assert_eq!(10 & 5, res);
    }

    #[test]
    fn valid_bitxor() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(5u32, stack).expect("init should have succeeded");
        BitwiseXOR {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(stack).expect("result should be of valid type");
        assert_eq!(10 ^ 5, res);
    }

    #[test]
    fn valid_bitor() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(5u32, stack).expect("init should have succeeded");
        BitwiseOR {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num4::<u32>(stack).expect("result should be of valid type");
        assert_eq!(10 | 5, res);
    }

    #[test]
    fn valid_less() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(5u32, stack).expect("init should have succeeded");
        Less {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(stack).expect("result should be of valid type");
        assert_eq!(10 < 5, res);
    }

    #[test]
    fn valid_less_equal() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(5u32, stack).expect("init should have succeeded");
        LessEqual {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(stack).expect("result should be of valid type");
        assert_eq!(10 <= 5, res);
    }
    #[test]
    fn valid_cast() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num8(1u64, stack).expect("init should have succeeded");
        Cast {
            from: OpPrimitive::Number(NumberType::U64),
            to: OpPrimitive::Bool,
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(stack).expect("result should be of valid type");
        assert_eq!(true, res);
    }
    #[test]
    fn valid_greater() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(5u32, stack).expect("init should have succeeded");
        Greater {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(stack).expect("result should be of valid type");
        assert_eq!(10 > 5, res);
    }

    #[test]
    fn valid_greater_equal() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(5u32, stack).expect("init should have succeeded");
        GreaterEqual {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(stack).expect("result should be of valid type");
        assert_eq!(10 >= 5, res);
    }

    #[test]
    fn valid_equal() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(10u32, stack).expect("init should have succeeded");
        GreaterEqual {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(stack).expect("result should be of valid type");
        assert_eq!(10 == 10, res);
    }

    #[test]
    fn valid_not_equal() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        init_num4(5u32, stack).expect("init should have succeeded");
        GreaterEqual {
            left: OpPrimitive::Number(NumberType::U32),
            right: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(stack).expect("result should be of valid type");
        assert_eq!(10 != 5, res);
    }

    #[test]
    fn valid_logical_and() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_bool(true, stack).expect("init should have succeeded");
        init_bool(true, stack).expect("init should have succeeded");
        LogicalAnd()
            .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
            .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(stack).expect("result should be of valid type");
        assert_eq!(true && true, res);
    }

    #[test]
    fn valid_logical_or() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_bool(true, stack).expect("init should have succeeded");
        init_bool(true, stack).expect("init should have succeeded");
        LogicalOr()
            .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
            .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(stack).expect("result should be of valid type");
        assert_eq!(true || true, res);
    }

    #[test]
    fn valid_minus() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_num4(10u32, stack).expect("init should have succeeded");
        Minus {
            data_type: OpPrimitive::Number(NumberType::U32),
        }
        .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
        .expect("execution should have succeeded");

        let res = OpPrimitive::get_num8::<i64>(stack).expect("result should be of valid type");
        assert_eq!(-10i64, res);
    }

    #[test]
    fn valid_not() {
        let mut engine = crate::vm::vm::NoopGameEngine {};
        let (mut runtime, mut heap, mut stdio) = Runtime::new();
        let tid = runtime
            .spawn_with_scope(crate::vm::vm::Player::P1, Scope::new())
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, stack, mut program) = runtime
            .get_mut(crate::vm::vm::Player::P1, tid)
            .expect("Thread should exist");
        init_bool(true, stack).expect("init should have succeeded");
        Not()
            .execute(&mut program, stack, &mut heap, &mut stdio, &mut engine)
            .expect("execution should have succeeded");

        let res = OpPrimitive::get_bool(stack).expect("result should be of valid type");
        assert_eq!(false, res);
    }
}
