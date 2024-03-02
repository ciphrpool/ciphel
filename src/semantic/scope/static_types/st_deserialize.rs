use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{
        expressions::{
            data::{Data, Number, Primitive, Slice, StrSlice, Tuple, Vector},
            Atomic, Expression,
        },
        utils::lexem,
    },
    semantic::{
        scope::{static_types::StaticType, user_type_impl::UserType, ScopeApi},
        AccessLevel, EType, Either, Info, Metadata, SizeOf,
    },
    vm::{
        allocator::{stack::Offset, MemoryAddress},
        casm::{
            alloc::Access,
            memcopy::MemCopy,
            operation::{Addition, OpPrimitive, Operation, OperationKind},
            serialize::Serialized,
            Casm,
        },
        platform::{
            stdlib::{
                io::{IOCasm, PrintCasm},
                StdCasm,
            },
            LibCasm,
        },
        vm::{CodeGenerationError, DeserializeFrom, Printer, RuntimeError},
    },
};

use super::{NumberType, PrimitiveType, SliceType, StrSliceType, StringType, TupleType, VecType};

impl<Scope: ScopeApi> DeserializeFrom<Scope> for StaticType {
    type Output = Data<Scope>;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        match self {
            StaticType::Primitive(value) => {
                Ok(Data::Primitive(<PrimitiveType as DeserializeFrom<
                    Scope,
                >>::deserialize_from(
                    value, bytes
                )?))
            }
            StaticType::Slice(value) => Ok(Data::Slice(value.deserialize_from(bytes)?)),
            StaticType::Vec(value) => Ok(Data::Vec(value.deserialize_from(bytes)?)),
            StaticType::Fn(_value) => unimplemented!(),
            StaticType::Chan(_value) => unimplemented!(),
            StaticType::Tuple(value) => Ok(Data::Tuple(value.deserialize_from(bytes)?)),
            StaticType::Unit => Ok(Data::Unit),
            StaticType::Any => Err(RuntimeError::Deserialization),
            StaticType::Error => Err(RuntimeError::Deserialization),
            StaticType::Address(_value) => todo!(),
            StaticType::Map(_value) => unimplemented!(),
            StaticType::String(value) => {
                Ok(Data::StrSlice(
                    <StringType as DeserializeFrom<Scope>>::deserialize_from(value, bytes)?,
                ))
            }
            StaticType::StrSlice(value) => {
                Ok(Data::StrSlice(
                    <StrSliceType as DeserializeFrom<Scope>>::deserialize_from(value, bytes)?,
                ))
            }
        }
    }
}

impl Printer for StaticType {
    fn build_printer(&self) -> Result<Vec<Casm>, CodeGenerationError> {
        match self {
            StaticType::Primitive(value) => value.build_printer(),
            StaticType::Slice(value) => value.build_printer(),
            StaticType::String(value) => value.build_printer(),
            StaticType::StrSlice(value) => value.build_printer(),
            StaticType::Vec(value) => value.build_printer(),
            StaticType::Fn(value) => todo!(),
            StaticType::Chan(value) => todo!(),
            StaticType::Tuple(value) => value.build_printer(),
            StaticType::Unit => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintID("unit".into())),
            )))]),
            StaticType::Any => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintID("any".into())),
            )))]),
            StaticType::Error => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintID("error".into())),
            )))]),
            StaticType::Address(value) => todo!(),
            StaticType::Map(value) => todo!(),
        }
    }
}

impl<Scope: ScopeApi> DeserializeFrom<Scope> for NumberType {
    type Output = Number;
    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        match self {
            NumberType::U8 => {
                let data = TryInto::<&[u8; 1]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Number::U8(u8::from_le_bytes(*data)))
            }
            NumberType::U16 => {
                let data = TryInto::<&[u8; 2]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Number::U16(u16::from_le_bytes(*data)))
            }
            NumberType::U32 => {
                let data = TryInto::<&[u8; 4]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Number::U32(u32::from_le_bytes(*data)))
            }
            NumberType::U64 => {
                let data = TryInto::<&[u8; 8]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Number::U64(u64::from_le_bytes(*data)))
            }
            NumberType::U128 => {
                let data = TryInto::<&[u8; 16]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Number::U128(u128::from_le_bytes(*data)))
            }
            NumberType::I8 => {
                let data = TryInto::<&[u8; 1]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Number::I8(i8::from_le_bytes(*data)))
            }
            NumberType::I16 => {
                let data = TryInto::<&[u8; 2]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Number::I16(i16::from_le_bytes(*data)))
            }
            NumberType::I32 => {
                let data = TryInto::<&[u8; 4]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Number::I32(i32::from_le_bytes(*data)))
            }
            NumberType::I64 => {
                let data = TryInto::<&[u8; 8]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Number::I64(i64::from_le_bytes(*data)))
            }
            NumberType::I128 => {
                let data = TryInto::<&[u8; 16]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Number::I128(i128::from_le_bytes(*data)))
            }
            NumberType::F64 => {
                let data = TryInto::<&[u8; 8]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Number::F64(f64::from_le_bytes(*data)))
            }
        }
    }
}

impl Printer for NumberType {
    fn build_printer(&self) -> Result<Vec<Casm>, CodeGenerationError> {
        match self {
            NumberType::U8 => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintU8),
            )))]),
            NumberType::U16 => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintU16),
            )))]),
            NumberType::U32 => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintU32),
            )))]),
            NumberType::U64 => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintU64),
            )))]),
            NumberType::U128 => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintU128),
            )))]),
            NumberType::I8 => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintI8),
            )))]),
            NumberType::I16 => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintI16),
            )))]),
            NumberType::I32 => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintI32),
            )))]),
            NumberType::I64 => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintI64),
            )))]),
            NumberType::I128 => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintI128),
            )))]),
            NumberType::F64 => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintF64),
            )))]),
        }
    }
}

impl<Scope: ScopeApi> DeserializeFrom<Scope> for PrimitiveType {
    type Output = Primitive;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        match self {
            PrimitiveType::Number(number) => {
                <NumberType as DeserializeFrom<Scope>>::deserialize_from(number, bytes)
                    .map(|n| Primitive::Number(n.into()))
            }
            PrimitiveType::Char => {
                let data = TryInto::<&[u8; 1]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Primitive::Char(data[0] as char))
            }
            PrimitiveType::Bool => {
                let data = TryInto::<&[u8; 1]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Primitive::Bool(data[0] != 0))
            }
        }
    }
}
impl Printer for PrimitiveType {
    fn build_printer(&self) -> Result<Vec<Casm>, CodeGenerationError> {
        match self {
            PrimitiveType::Number(value) => value.build_printer(),
            PrimitiveType::Char => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintChar),
            )))]),
            PrimitiveType::Bool => Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintBool),
            )))]),
        }
    }
}
// Helper function to extract a u64 value from a byte slice.
pub fn extract_u64(slice: &[u8]) -> Result<(u64, &[u8]), RuntimeError> {
    if slice.len() < 8 {
        return Err(RuntimeError::Deserialization);
    }
    let (bytes, rest) = slice.split_at(8);
    let arr: [u8; 8] = bytes
        .try_into()
        .map_err(|_| RuntimeError::Deserialization)?;
    Ok((u64::from_le_bytes(arr), rest))
}
// Helper function to extract a u64 value at the end from a byte slice.
pub fn extract_end_u64(slice: &[u8]) -> Result<(u64, &[u8]), RuntimeError> {
    if slice.len() < 8 {
        return Err(RuntimeError::Deserialization);
    }
    let (rest, bytes) = slice.split_at(slice.len() - 8);
    let arr: [u8; 8] = bytes
        .try_into()
        .map_err(|_| RuntimeError::Deserialization)?;
    Ok((u64::from_le_bytes(arr), rest))
}

impl<Scope: ScopeApi> DeserializeFrom<Scope> for VecType {
    type Output = Vector<Scope>;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        let (length, rest) = extract_u64(bytes)?;
        let (capacity, rest) = extract_u64(rest)?;
        let rest = rest;

        let size = self.0.size_of();
        let array: Vec<Result<Option<Expression<Scope>>, RuntimeError>> = rest
            .chunks(size)
            .enumerate()
            .map(|(idx, bytes)| {
                if idx as u64 >= length {
                    Ok(None)
                } else {
                    <EType as DeserializeFrom<Scope>>::deserialize_from(&self.0, bytes)
                        .map(|data| Some(Expression::Atomic(Atomic::Data(data))))
                }
            })
            .collect();
        if !array.iter().all(|e| e.is_ok()) {
            return Err(RuntimeError::Deserialization);
        }
        Ok(Vector {
            value: array
                .into_iter()
                .take_while(|e| e.clone().ok().flatten().is_some())
                .map(|e| e.ok().flatten().unwrap())
                .collect(),
            metadata: Metadata {
                info: Rc::new(RefCell::new(Info::Resolved {
                    context: None,
                    signature: Some(Either::Static(Rc::new(StaticType::Vec(self.clone())))),
                })),
            },
            length: length as usize,
            capacity: capacity as usize,
        })
    }
}
impl Printer for VecType {
    fn build_printer(&self) -> Result<Vec<Casm>, CodeGenerationError> {
        let mut instructions = Vec::default();
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintLexem(lexem::SQ_BRA_O),
        )))));

        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintLexem(lexem::SQ_BRA_C),
        )))));
        Ok(instructions)
    }
}

impl<Scope: ScopeApi> DeserializeFrom<Scope> for StringType {
    type Output = StrSlice;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        let (length, rest) = extract_u64(bytes)?;
        let (capacity, rest) = extract_u64(rest)?;
        let rest = rest;

        let str_slice = std::str::from_utf8(&rest[0..length as usize])
            .map_err(|_| RuntimeError::Deserialization)?;

        Ok(StrSlice {
            value: str_slice.to_string(),
            metadata: Metadata {
                info: Rc::new(RefCell::new(Info::Resolved {
                    context: None,
                    signature: Some(Either::Static(Rc::new(StaticType::String(self.clone())))),
                })),
            },
        })
    }
}
impl Printer for StringType {
    fn build_printer(&self) -> Result<Vec<Casm>, CodeGenerationError> {
        let mut instructions = Vec::default();
        instructions.push(Casm::MemCopy(MemCopy::CloneFromSmartPointer));
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintString,
        )))));
        Ok(instructions)
    }
}
impl<Scope: ScopeApi> DeserializeFrom<Scope> for StrSliceType {
    type Output = StrSlice;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        let str_slice = std::str::from_utf8(&bytes).map_err(|_| RuntimeError::Deserialization)?;

        Ok(StrSlice {
            value: str_slice.to_string(),
            metadata: Metadata {
                info: Rc::new(RefCell::new(Info::Resolved {
                    context: None,
                    signature: Some(Either::Static(Rc::new(StaticType::StrSlice(self.clone())))),
                })),
            },
        })
    }
}
impl Printer for StrSliceType {
    fn build_printer(&self) -> Result<Vec<Casm>, CodeGenerationError> {
        Ok(vec![Casm::Platform(LibCasm::Std(StdCasm::IO(
            IOCasm::Print(PrintCasm::PrintStr(self.size)),
        )))])
    }
}
impl<Scope: ScopeApi> DeserializeFrom<Scope> for TupleType {
    type Output = Tuple<Scope>;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        let mut offset = 0;
        let mut value = Vec::default();
        for element_type in &self.0 {
            let size = element_type.size_of();
            if offset + size > bytes.len() {
                return Err(RuntimeError::Deserialization);
            }
            let data = <EType as DeserializeFrom<Scope>>::deserialize_from(
                &element_type,
                &bytes[offset..offset + size],
            )?;
            value.push(Expression::Atomic(Atomic::Data(data)));
            offset += size;
        }

        Ok(Tuple {
            value,
            metadata: Metadata {
                info: Rc::new(RefCell::new(Info::Resolved {
                    context: None,
                    signature: Some(Either::Static(Rc::new(StaticType::Tuple(self.clone())))),
                })),
            },
        })
    }
}
impl Printer for TupleType {
    fn build_printer(&self) -> Result<Vec<Casm>, CodeGenerationError> {
        let mut instructions = Vec::default();
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintLexem(lexem::PAR_O),
        )))));
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::StdOutBufOpen,
        )))));
        for (idx, item) in self.0.iter().enumerate().rev() {
            instructions.extend(item.build_printer()?);
            if idx > 0 {
                instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
                    PrintCasm::PrintLexem(lexem::COMA),
                )))));
            }
        }
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::StdOutBufRevFlush,
        )))));
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintLexem(lexem::PAR_C),
        )))));
        Ok(instructions)
    }
}
impl<Scope: ScopeApi> DeserializeFrom<Scope> for SliceType {
    type Output = Slice<Scope>;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        let array: Vec<Result<Option<Expression<Scope>>, RuntimeError>> = bytes
            .chunks(self.item_type.size_of())
            .enumerate()
            .map(|(idx, bytes)| {
                if idx >= self.size {
                    Ok(None)
                } else {
                    <EType as DeserializeFrom<Scope>>::deserialize_from(&self.item_type, bytes)
                        .map(|data| Some(Expression::Atomic(Atomic::Data(data))))
                }
            })
            .collect();
        if !array.iter().all(|e| e.is_ok()) {
            return Err(RuntimeError::Deserialization);
        }
        Ok(Slice {
            value: array
                .into_iter()
                .take_while(|e| e.clone().ok().flatten().is_some())
                .map(|e| e.ok().flatten().unwrap())
                .collect(),
            metadata: Metadata {
                info: Rc::new(RefCell::new(Info::Resolved {
                    context: None,
                    signature: Some(Either::Static(Rc::new(StaticType::Slice(self.clone())))),
                })),
            },
        })
    }
}
impl Printer for SliceType {
    fn build_printer(&self) -> Result<Vec<Casm>, CodeGenerationError> {
        todo!()
    }
}
