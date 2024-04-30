use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{
    ast::{
        expressions::{
            data::{Data, Number, Primitive, Slice, StrSlice, Tuple, Vector},
            Atomic, Expression,
        },
        utils::lexem,
    },
    semantic::{scope::static_types::StaticType, EType, Either, Info, Metadata, SizeOf},
    vm::{
        casm::{branch::Label, mem::Mem, Casm, CasmProgram},
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

use super::{
    NumberType, PrimitiveType, RangeType, SliceType, StrSliceType, StringType, TupleType, VecType,
};

impl DeserializeFrom for StaticType {
    type Output = Data;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        match self {
            StaticType::Primitive(value) => Ok(Data::Primitive(
                <PrimitiveType as DeserializeFrom>::deserialize_from(value, bytes)?,
            )),
            StaticType::Slice(value) => Ok(Data::Slice(value.deserialize_from(bytes)?)),
            StaticType::Vec(value) => Ok(Data::Vec(value.deserialize_from(bytes)?)),
            StaticType::StaticFn(_value) => unimplemented!(),
            StaticType::Closure(_value) => unimplemented!(),
            StaticType::Chan(_value) => unimplemented!(),
            StaticType::Tuple(value) => Ok(Data::Tuple(value.deserialize_from(bytes)?)),
            StaticType::Unit => Ok(Data::Unit),
            StaticType::Any => Err(RuntimeError::Deserialization),
            StaticType::Error => {
                if bytes.len() != 1 {
                    return Err(RuntimeError::Deserialization);
                }
                return Ok(Data::Primitive(Primitive::Bool(bytes[0] > 0)));
            }
            StaticType::Address(_value) => unimplemented!(),
            StaticType::Map(_value) => unimplemented!(),
            StaticType::String(value) => Ok(Data::StrSlice(
                <StringType as DeserializeFrom>::deserialize_from(value, bytes)?,
            )),
            StaticType::StrSlice(value) => Ok(Data::StrSlice(
                <StrSliceType as DeserializeFrom>::deserialize_from(value, bytes)?,
            )),
            StaticType::Range(_) => unimplemented!(),
        }
    }
}

impl Printer for StaticType {
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        match self {
            StaticType::Primitive(value) => value.build_printer(instructions),
            StaticType::Slice(value) => value.build_printer(instructions),
            StaticType::String(value) => value.build_printer(instructions),
            StaticType::StrSlice(value) => value.build_printer(instructions),
            StaticType::Vec(value) => value.build_printer(instructions),
            StaticType::StaticFn(_value) => {
                let _ = instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                    IOCasm::Print(PrintCasm::PrintAddr),
                ))));
                Ok(())
            }
            StaticType::Closure(_value) => {
                let _ = instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                    IOCasm::Print(PrintCasm::PrintAddr),
                ))));
                Ok(())
            }
            StaticType::Chan(_value) => todo!(),
            StaticType::Tuple(value) => value.build_printer(instructions),
            StaticType::Range(value) => value.build_printer(instructions),
            StaticType::Unit => {
                let _ = instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                    IOCasm::Print(PrintCasm::PrintID("unit".into())),
                ))));
                Ok(())
            }
            StaticType::Any => {
                let _ = instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                    IOCasm::Print(PrintCasm::PrintID("any".into())),
                ))));
                Ok(())
            }
            StaticType::Error => {
                let _ = instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                    IOCasm::Print(PrintCasm::PrintID("error".into())),
                ))));
                Ok(())
            }
            StaticType::Address(_value) => {
                let _ = instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                    IOCasm::Print(PrintCasm::PrintAddr),
                ))));
                Ok(())
            }
            StaticType::Map(_value) => todo!(),
        }
    }
}

impl DeserializeFrom for NumberType {
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
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        match self {
            NumberType::U8 => instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintU8),
            )))),
            NumberType::U16 => instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintU16),
            )))),
            NumberType::U32 => instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintU32),
            )))),
            NumberType::U64 => instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintU64),
            )))),
            NumberType::U128 => instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintU128),
            )))),
            NumberType::I8 => instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintI8),
            )))),
            NumberType::I16 => instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintI16),
            )))),
            NumberType::I32 => instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintI32),
            )))),
            NumberType::I64 => instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintI64),
            )))),
            NumberType::I128 => instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintI128),
            )))),
            NumberType::F64 => instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintF64),
            )))),
        }
        Ok(())
    }
}

impl DeserializeFrom for PrimitiveType {
    type Output = Primitive;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        match self {
            PrimitiveType::Number(number) => {
                <NumberType as DeserializeFrom>::deserialize_from(number, bytes)
                    .map(|n| Primitive::Number(n.into()))
            }
            PrimitiveType::Char => {
                let data = TryInto::<&[u8; 4]>::try_into(bytes)
                    .map_err(|_| RuntimeError::Deserialization)?;
                Ok(Primitive::Char(
                    std::str::from_utf8(data.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?
                        .chars()
                        .next()
                        .ok_or(RuntimeError::Deserialization)?,
                ))
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
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        match self {
            PrimitiveType::Number(value) => {
                let _ = value.build_printer(instructions)?;
            }
            PrimitiveType::Char => instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintChar),
            )))),
            PrimitiveType::Bool => instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                IOCasm::Print(PrintCasm::PrintBool),
            )))),
        }
        Ok(())
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

impl DeserializeFrom for VecType {
    type Output = Vector;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        let (length, rest) = extract_u64(bytes)?;
        let (capacity, rest) = extract_u64(rest)?;
        let rest = rest;

        let size = self.0.size_of();
        let array: Vec<Result<Option<Expression>, RuntimeError>> = rest
            .chunks(size)
            .enumerate()
            .map(|(idx, bytes)| {
                if idx as u64 >= length {
                    Ok(None)
                } else {
                    <EType as DeserializeFrom>::deserialize_from(&self.0, bytes)
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
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        instructions.push(Casm::Mem(Mem::CloneFromSmartPointer(self.0.size_of())));

        let continue_label = Label::gen();
        let end_label = Label::gen();

        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintList {
                length: None,
                continue_label,
                end_label,
            },
        )))));

        let _ = self.0.build_printer(instructions)?;
        instructions.push_label_id(continue_label, "print_continue".into());
        instructions.push_label_id(end_label, "print_end".into());
        Ok(())
    }
}

impl DeserializeFrom for StringType {
    type Output = StrSlice;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        let (length, rest) = extract_u64(bytes)?;
        let (_capacity, rest) = extract_u64(rest)?;
        let rest = rest;

        let str_slice = std::str::from_utf8(&rest[0..length as usize])
            .map_err(|_| RuntimeError::Deserialization)?;

        Ok(StrSlice {
            value: str_slice.to_string(),
            padding: Cell::new(0),
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
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        instructions.push(Casm::Mem(Mem::CloneFromSmartPointer(1)));
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintString,
        )))));
        Ok(())
    }
}
impl DeserializeFrom for StrSliceType {
    type Output = StrSlice;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        let bytes_len = bytes.len();
        let str_slice = std::str::from_utf8(&bytes[..bytes_len - 8])
            .map_err(|_| RuntimeError::Deserialization)?;

        Ok(StrSlice {
            value: str_slice.to_string(),
            padding: Cell::new(0),
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
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintString,
        )))));
        Ok(())
    }
}
impl DeserializeFrom for TupleType {
    type Output = Tuple;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        let mut offset = 0;
        let mut value = Vec::default();
        for element_type in &self.0 {
            let size = element_type.size_of();
            if offset + size > bytes.len() {
                return Err(RuntimeError::Deserialization);
            }
            let data = <EType as DeserializeFrom>::deserialize_from(
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
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::StdOutBufOpen,
        )))));
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintLexem(lexem::PAR_C),
        )))));
        for (idx, item) in self.0.iter().enumerate().rev() {
            let _ = item.build_printer(instructions)?;
            if idx > 0 {
                instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
                    PrintCasm::PrintLexem(lexem::COMA),
                )))));
            }
        }
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintLexem(lexem::PAR_O),
        )))));
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::StdOutBufRevFlush,
        )))));
        Ok(())
    }
}
impl DeserializeFrom for SliceType {
    type Output = Slice;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        let array: Vec<Result<Option<Expression>, RuntimeError>> = bytes
            .chunks(self.item_type.size_of())
            .enumerate()
            .map(|(idx, bytes)| {
                if idx >= self.size {
                    Ok(None)
                } else {
                    <EType as DeserializeFrom>::deserialize_from(&self.item_type, bytes)
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
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        let continue_label = Label::gen();
        let end_label = Label::gen();

        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintList {
                length: Some(self.size),
                continue_label,
                end_label,
            },
        )))));

        let _ = self.item_type.build_printer(instructions)?;
        instructions.push_label_id(continue_label, "print_continue".into());
        instructions.push_label_id(end_label, "print_end".into());
        Ok(())
    }
}

// impl DeserializeFrom for RangeType {
//     type Output = Range;

//     fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
//         if bytes.len() < self.size_of() {
//             return Err(RuntimeError::Deserialization);
//         }
//         let (bytes_lower, rest) = bytes.split_at(self.num.size_of());
//         let lower =
//             <NumberType as DeserializeFrom>::deserialize_from(&self.num, bytes_lower)?;

//         let (bytes_upper, rest) = rest.split_at(self.num.size_of());
//         let upper =
//             <NumberType as DeserializeFrom>::deserialize_from(&self.num, bytes_upper)?;

//         let (bytes_incr, rest) = rest.split_at(self.num.size_of());
//         let incr = <NumberType as DeserializeFrom>::deserialize_from(&self.num, bytes_incr)?;
//         Ok(Range {
//             lower: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
//                 Primitive::Number(lower.into()),
//             )))),
//             upper: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
//                 Primitive::Number(lower.into()),
//             )))),
//             incr: Some(incr.into()),
//             inclusive: self.inclusive,
//             metadata: Metadata {
//                 info: Rc::new(RefCell::new(Info::Resolved {
//                     context: None,
//                     signature: Some(Either::Static(Rc::new(StaticType::Range(self.clone())))),
//                 })),
//             },
//         })
//     }
// }

impl Printer for RangeType {
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::StdOutBufOpen,
        )))));

        /* Increment printing */
        self.num.build_printer(instructions);
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintLexem(lexem::COLON),
        )))));
        /* Upper printing */
        self.num.build_printer(instructions);

        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintLexem(lexem::RANGE_SEP),
        )))));

        /* Lower printing */
        self.num.build_printer(instructions);

        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::StdOutBufRevFlush,
        )))));
        Ok(())
    }
}
