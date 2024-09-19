use std::sync::Arc;

use crate::{
    ast::{
        expressions::{
            data::{Data, Number, Primitive, Slice, StrSlice, Tuple, Vector},
            Atomic, Expression,
        },
        utils::lexem,
    },
    semantic::{scope::static_types::StaticType, EType, Info, Metadata, SizeOf},
    vm::{
        casm::{branch::Label, mem::Mem, Casm, CasmProgram},
        platform::{
            stdlib::{
                io::{IOCasm, PrintCasm},
                StdCasm,
            },
            LibCasm,
        },
        vm::{CodeGenerationError, Printer, RuntimeError},
    },
};

use super::{
    NumberType, PrimitiveType, RangeType, SliceType, StrSliceType, StringType, TupleType, VecType,
};

impl Printer for StaticType {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
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
            StaticType::Tuple(value) => value.build_printer(instructions),
            StaticType::Range(value) => value.build_printer(instructions),
            StaticType::Unit => {
                let _ = instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                    IOCasm::Print(PrintCasm::PrintID("unit".to_string().into())),
                ))));
                Ok(())
            }
            StaticType::Any => {
                let _ = instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                    IOCasm::Print(PrintCasm::PrintID("any".to_string().into())),
                ))));
                Ok(())
            }
            StaticType::Error => {
                let _ = instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                    IOCasm::Print(PrintCasm::PrintError),
                ))));
                Ok(())
            }
            StaticType::Address(_value) => {
                let _ = instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(
                    IOCasm::Print(PrintCasm::PrintAddr),
                ))));
                Ok(())
            }
            StaticType::Map(_value) => Err(CodeGenerationError::Default),
        }
    }
}

impl Printer for NumberType {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
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

impl Printer for PrimitiveType {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
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

impl Printer for VecType {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
        todo!();
        // instructions.push(Casm::Mem(Mem::CloneFromSmartPointer(self.0.size_of())));

        // let continue_label = Label::gen();
        // let end_label = Label::gen();

        // instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
        //     PrintCasm::PrintList {
        //         length: None,
        //         continue_label,
        //         end_label,
        //     },
        // )))));

        // let _ = self.0.build_printer(instructions)?;
        // instructions.push_label_id(continue_label, "print_continue".to_string().into());
        // instructions.push_label_id(end_label, "print_end".to_string().into());
        Ok(())
    }
}

impl Printer for StringType {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
        todo!();
        // instructions.push(Casm::Mem(Mem::CloneFromSmartPointer(1)));
        // instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
        //     PrintCasm::PrintString,
        // )))));
        Ok(())
    }
}

impl Printer for StrSliceType {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintString,
        )))));
        Ok(())
    }
}

impl Printer for TupleType {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
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

impl Printer for SliceType {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
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
        instructions.push_label_id(continue_label, "print_continue".to_string().into());
        instructions.push_label_id(end_label, "print_end".to_string().into());
        Ok(())
    }
}

impl Printer for RangeType {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
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
