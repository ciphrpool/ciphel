use num_traits::ToBytes;
use ulid::Ulid;

use crate::{
    semantic::{scope::static_types::NumberType, AccessLevel, SizeOf},
    vm::{
        allocator::{stack::Offset, MemoryAddress},
        casm::{
            alloc::Access,
            branch::BranchIf,
            data::Data,
            locate::{Locate, LocateNextUTF8Char},
            mem::Mem,
            operation::{
                Addition, Greater, Less, LessEqual, Mult, OpPrimitive, Operation, OperationKind,
            },
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, NextItem},
    },
};

use super::{RangeType, SliceType, StaticType, StrSliceType, StringType, VecType};

impl NextItem for StaticType {
    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        match self {
            StaticType::Slice(value) => value.init_index(instructions),
            StaticType::String(value) => value.init_index(instructions),
            StaticType::StrSlice(value) => value.init_index(instructions),
            StaticType::Vec(value) => value.init_index(instructions),
            StaticType::Range(value) => value.init_index(instructions),
            StaticType::Chan(_) => todo!(),
            StaticType::Error => todo!(),
            StaticType::Address(_) => todo!(),
            StaticType::Map(_) => todo!(),
            _ => Err(CodeGenerationError::UnresolvedError),
        }
    }
    fn build_item(
        &self,
        instructions: &CasmProgram,
        end_label: Ulid,
    ) -> Result<(), CodeGenerationError> {
        match self {
            StaticType::Slice(value) => value.build_item(instructions, end_label),
            StaticType::String(value) => value.build_item(instructions, end_label),
            StaticType::StrSlice(value) => value.build_item(instructions, end_label),
            StaticType::Vec(value) => value.build_item(instructions, end_label),
            StaticType::Range(value) => value.build_item(instructions, end_label),
            StaticType::Chan(_) => todo!(),
            StaticType::Error => todo!(),
            StaticType::Address(_) => todo!(),
            StaticType::Map(_) => todo!(),
            _ => Err(CodeGenerationError::UnresolvedError),
        }
    }
    fn next(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        match self {
            StaticType::Slice(value) => value.next(instructions),
            StaticType::String(value) => value.next(instructions),
            StaticType::StrSlice(value) => value.next(instructions),
            StaticType::Vec(value) => value.next(instructions),
            StaticType::Range(value) => value.next(instructions),
            StaticType::Chan(_) => todo!(),
            StaticType::Error => todo!(),
            StaticType::Address(_) => todo!(),
            StaticType::Map(_) => todo!(),
            _ => Err(CodeGenerationError::UnresolvedError),
        }
    }
}

impl NextItem for SliceType {
    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        instructions.push(Casm::Locate(Locate {
            address: MemoryAddress::Stack {
                offset: Offset::ST(0),
                level: AccessLevel::Direct,
            },
        }));
        instructions.push(Casm::Locate(Locate {
            address: MemoryAddress::Stack {
                offset: Offset::ST(-(self.size_of() as isize + 8)),
                level: AccessLevel::Direct,
            },
        }));
        /* STACK + 16 : UPPER | START */
        Ok(())
    }
    fn build_item(
        &self,
        instructions: &CasmProgram,
        end_label: Ulid,
    ) -> Result<(), CodeGenerationError> {
        /* Dup loop index and end offset*/
        instructions.push(Casm::MemCopy(Mem::Dup(16)));

        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Greater(Greater {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        instructions.push(Casm::If(BranchIf {
            else_label: end_label,
        }));

        instructions.push(Casm::MemCopy(Mem::Dup(8)));
        instructions.push(Casm::Access(Access::Runtime {
            size: Some(self.item_type.size_of()),
        }));

        /* STACK : UPPER | INDEX (8) | ITEM (item_size) */
        Ok(())
    }
    fn next(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        /* STACK : UPPER | INDEX (8) */
        instructions.push(Casm::Data(Data::Serialized {
            data: self.item_type.size_of().to_le_bytes().into(),
        }));

        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Addition(Addition {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));

        Ok(())
    }
}

impl NextItem for StringType {
    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        /* STACK : POINTER TO STRING */
        instructions.push(Casm::MemCopy(Mem::Dup(8)));
        instructions.push(Casm::MemCopy(Mem::Dup(8)));
        /* STACK : POINTER | POINTER | POINTER */
        instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));

        /* STACK : POINTER | POINTER | SIZE OF STRING  */
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Addition(Addition {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        instructions.push(Casm::Data(Data::Serialized {
            data: (16u64).to_le_bytes().into(),
        }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Addition(Addition {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        /* STACK : POINTER | UPPER */

        instructions.push(Casm::Access(Access::Static {
            address: MemoryAddress::Stack {
                offset: Offset::ST(-16),
                level: AccessLevel::Direct,
            },
            size: 8,
        }));

        /* STACK : POINTER | UPPER | POINTER */

        instructions.push(Casm::Data(Data::Serialized {
            data: (16u64).to_le_bytes().into(),
        }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Addition(Addition {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        /* STACK : POINTER (original) | UPPER | START */
        /* STACK +16 : UPPER | START (8) */
        Ok(())
    }
    fn build_item(
        &self,
        instructions: &CasmProgram,
        end_label: Ulid,
    ) -> Result<(), CodeGenerationError> {
        /* Dup loop index and end offset*/
        instructions.push(Casm::MemCopy(Mem::Dup(16)));

        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Greater(Greater {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        instructions.push(Casm::If(BranchIf {
            else_label: end_label,
        }));

        instructions.push(Casm::MemCopy(Mem::Dup(8)));
        instructions.push(Casm::Access(Access::RuntimeCharUTF8));
        /* STACK : UPPER | INDEX (8) | CHAR (4) */
        Ok(())
    }
    fn next(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        /* STACK : UPPER | INDEX */
        instructions.push(Casm::LocateNextUTF8Char(LocateNextUTF8Char::RuntimeNext));

        Ok(())
    }
}

impl NextItem for StrSliceType {
    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        instructions.push(Casm::Locate(Locate {
            address: MemoryAddress::Stack {
                offset: Offset::ST(0),
                level: AccessLevel::Direct,
            },
        }));

        /* STACK : TOP */
        instructions.push(Casm::Locate(Locate {
            address: MemoryAddress::Stack {
                offset: Offset::ST(-(self.size_of() as isize + 8)),
                level: AccessLevel::Direct,
            },
        }));
        /* STACK +16 : UPPER | START*/
        Ok(())
    }
    fn build_item(
        &self,
        instructions: &CasmProgram,
        end_label: Ulid,
    ) -> Result<(), CodeGenerationError> {
        /* Dup loop index and end offset*/
        instructions.push(Casm::MemCopy(Mem::Dup(16)));

        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Greater(Greater {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        instructions.push(Casm::If(BranchIf {
            else_label: end_label,
        }));

        instructions.push(Casm::MemCopy(Mem::Dup(8)));
        instructions.push(Casm::Access(Access::RuntimeCharUTF8));
        /* STACK : UPPER | INDEX (8) | CHAR (4) */
        Ok(())
    }
    fn next(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        /* STACK : UPPER | INDEX */
        instructions.push(Casm::LocateNextUTF8Char(LocateNextUTF8Char::RuntimeNext));

        Ok(())
    }
}

impl NextItem for VecType {
    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        /* STACK : POINTER TO VEC */
        instructions.push(Casm::MemCopy(Mem::Dup(8)));
        instructions.push(Casm::MemCopy(Mem::Dup(8)));
        /* STACK : POINTER | POINTER | POINTER */
        instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
        /* STACK : POINTER | POINTER | LENGTH */

        instructions.push(Casm::Data(Data::Serialized {
            data: self.0.size_of().to_le_bytes().into(),
        }));

        /* STACK : POINTER | POINTER | LENGTH | ITEM_SIZE */
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Mult(Mult {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        /* STACK : POINTER | POINTER | LENGTH * ITEM_SIZE */
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Addition(Addition {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        instructions.push(Casm::Data(Data::Serialized {
            data: (16u64).to_le_bytes().into(),
        }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Addition(Addition {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        /* STACK : POINTER | UPPER */

        instructions.push(Casm::Access(Access::Static {
            address: MemoryAddress::Stack {
                offset: Offset::ST(-16),
                level: AccessLevel::Direct,
            },
            size: 8,
        }));
        /* STACK : POINTER | UPPER | POINTER */
        instructions.push(Casm::Data(Data::Serialized {
            data: (16u64).to_le_bytes().into(),
        }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Addition(Addition {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        /* STACK : POINTER (original) | UPPER | START */
        /* STACK +16 : UPPER | START */
        Ok(())
    }

    fn build_item(
        &self,
        instructions: &CasmProgram,
        end_label: Ulid,
    ) -> Result<(), CodeGenerationError> {
        /* Dup loop index and end offset*/
        instructions.push(Casm::MemCopy(Mem::Dup(16)));

        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Greater(Greater {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        instructions.push(Casm::If(BranchIf {
            else_label: end_label,
        }));

        instructions.push(Casm::MemCopy(Mem::Dup(8)));
        instructions.push(Casm::Access(Access::Runtime {
            size: Some(self.0.size_of()),
        }));
        /* STACK : UPPER | INDEX (8) | ITEM_SIZE */
        Ok(())
    }

    fn next(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        /* STACK : UPPER | INDEX (8) */
        instructions.push(Casm::Data(Data::Serialized {
            data: self.0.size_of().to_le_bytes().into(),
        }));

        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Addition(Addition {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));

        Ok(())
    }
}
impl NextItem for RangeType {
    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        /* STACK : LOWER | UPPER | INCR */
        instructions.push(Casm::Access(Access::Static {
            address: MemoryAddress::Stack {
                offset: Offset::ST(-(self.size_of() as isize)),
                level: AccessLevel::Direct,
            },
            size: self.num.size_of(),
        }));

        /* STACK : LOWER | UPPER | INCR | LOWER */
        /* STACK +(num size) : LOWER */
        Ok(())
    }

    fn build_item(
        &self,
        instructions: &CasmProgram,
        end_label: Ulid,
    ) -> Result<(), CodeGenerationError> {
        /* STACK : (LOWER | UPPER | INCR ) (original) | INDEX */

        /* Dup loop index */
        instructions.push(Casm::MemCopy(Mem::Dup(self.num.size_of())));

        /* STACK : LOWER | UPPER | INCR | INDEX | INDEX */
        /* Get upper bound value */
        instructions.push(Casm::Access(Access::Static {
            address: MemoryAddress::Stack {
                offset: Offset::ST(-(self.num.size_of() as isize * 4)), /* Layout : LOWER | UPPER | INCR | INDEX | INDEX */
                level: AccessLevel::Direct,
            },
            size: self.num.size_of(),
        }));
        /* STACK : LOWER | UPPER | INCR | INDEX | INDEX | UPPER*/

        if self.inclusive {
            instructions.push(Casm::Operation(Operation {
                kind: OperationKind::LessEqual(LessEqual {
                    left: OpPrimitive::Number(self.num),
                    right: OpPrimitive::Number(self.num),
                }),
            }));
        } else {
            instructions.push(Casm::Operation(Operation {
                kind: OperationKind::Less(Less {
                    left: OpPrimitive::Number(self.num),
                    right: OpPrimitive::Number(self.num),
                }),
            }));
        }
        /* STACK : LOWER | UPPER | INCR | INDEX | CONTINUE boolean */

        instructions.push(Casm::If(BranchIf {
            else_label: end_label,
        }));

        /* Only the loop index left in the stack */
        instructions.push(Casm::MemCopy(Mem::Dup(self.num.size_of())));
        /* STACK : LOWER | UPPER | INCR | INDEX | INDEX*/

        Ok(())
    }

    fn next(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        /* STACK : LOWER | UPPER | INCR | INDEX */

        /* Get increment value */
        instructions.push(Casm::Access(Access::Static {
            address: MemoryAddress::Stack {
                offset: Offset::ST(-(self.num.size_of() as isize * 2)), /* Layout : LOWER | UPPER | INCR | INDEX */
                level: AccessLevel::Direct,
            },
            size: self.num.size_of(),
        }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Addition(Addition {
                left: OpPrimitive::Number(self.num),
                right: OpPrimitive::Number(self.num),
            }),
        }));

        Ok(())
    }
}
