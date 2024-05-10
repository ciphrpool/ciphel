use num_traits::ToBytes;
use ulid::Ulid;

use crate::{
    semantic::{scope::static_types::NumberType, AccessLevel, Either, SizeOf},
    vm::{
        allocator::{stack::Offset, MemoryAddress},
        casm::{
            alloc::Access,
            branch::BranchIf,
            data::Data,
            locate::{Locate, LocateUTF8Char},
            mem::Mem,
            operation::{
                Addition, Equal, Greater, Less, LessEqual, Mult, NotEqual, OpPrimitive, Operation,
                OperationKind, Substraction,
            },
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, NextItem},
    },
};

use super::{AddrType, RangeType, SliceType, StaticType, StrSliceType, StringType, VecType};

impl NextItem for StaticType {
    fn init_address(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        match self {
            StaticType::Slice(value) => value.init_address(instructions),
            StaticType::String(value) => value.init_address(instructions),
            StaticType::StrSlice(value) => value.init_address(instructions),
            StaticType::Vec(value) => value.init_address(instructions),
            StaticType::Range(value) => value.init_address(instructions),
            StaticType::Address(value) => value.init_address(instructions),
            _ => Err(CodeGenerationError::UnresolvedError),
        }
    }
    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        match self {
            StaticType::Slice(value) => value.init_index(instructions),
            StaticType::String(value) => value.init_index(instructions),
            StaticType::StrSlice(value) => value.init_index(instructions),
            StaticType::Vec(value) => value.init_index(instructions),
            StaticType::Range(value) => value.init_index(instructions),
            StaticType::Address(value) => value.init_index(instructions),
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
            StaticType::Address(value) => value.build_item(instructions, end_label),
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
            StaticType::Address(value) => value.next(instructions),
            _ => Err(CodeGenerationError::UnresolvedError),
        }
    }
}

impl NextItem for AddrType {
    fn init_address(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        match self.0.as_ref() {
            Either::Static(value) => match value.as_ref() {
                StaticType::Slice(_) => {}
                StaticType::String(_) => {
                    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
                }
                StaticType::StrSlice(_) => {}
                StaticType::Vec(_) => {
                    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
                }
                StaticType::Range(_) => {
                    instructions.push(Casm::Access(Access::Runtime {
                        size: Some(self.0.size_of()),
                    }));
                }
                StaticType::Address(_) => {
                    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
                    let _ = self.0.init_address(instructions)?;
                }
                _ => {}
            },
            Either::User(_) => {}
        }
        Ok(())
    }
    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        self.0.init_index(instructions)
    }

    fn build_item(
        &self,
        instructions: &CasmProgram,
        end_label: Ulid,
    ) -> Result<(), CodeGenerationError> {
        self.0.build_item(instructions, end_label)
    }

    fn next(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        self.0.next(instructions)
    }
}

impl NextItem for SliceType {
    fn init_address(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        instructions.push(Casm::Locate(Locate {
            address: MemoryAddress::Stack {
                offset: Offset::ST(-(self.size_of() as isize)),
                level: AccessLevel::Direct,
            },
        }));
        Ok(())
    }

    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        instructions.push(Casm::Data(Data::Serialized {
            data: (self.size_of() as u64).to_le_bytes().into(),
        }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Addition(Addition {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        instructions.push(Casm::Mem(Mem::Dup(8)));
        instructions.push(Casm::Data(Data::Serialized {
            data: (self.size_of() as u64).to_le_bytes().into(),
        }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Substraction(Substraction {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
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
        instructions.push(Casm::Mem(Mem::Dup(16)));

        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Greater(Greater {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        instructions.push(Casm::If(BranchIf {
            else_label: end_label,
        }));

        instructions.push(Casm::Mem(Mem::Dup(8)));
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
    fn init_address(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        Ok(())
    }
    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        /* STACK : POINTER TO STRING */
        instructions.push(Casm::Mem(Mem::Dup(8)));
        instructions.push(Casm::Mem(Mem::Dup(8)));
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
        instructions.push(Casm::Mem(Mem::Dup(16)));

        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Greater(Greater {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        instructions.push(Casm::If(BranchIf {
            else_label: end_label,
        }));

        instructions.push(Casm::Mem(Mem::Dup(8)));
        instructions.push(Casm::Access(Access::RuntimeCharUTF8));
        instructions.push(Casm::Mem(Mem::Dup(4)));
        instructions.push(Casm::Data(Data::Serialized {
            data: [0; 4].into(),
        }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::NotEqual(NotEqual { left: 4, right: 4 }),
        }));
        instructions.push(Casm::If(BranchIf {
            else_label: end_label,
        }));
        /* STACK : UPPER | INDEX (8) | CHAR (4) */
        Ok(())
    }
    fn next(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        /* STACK : UPPER | INDEX */
        instructions.push(Casm::LocateUTF8Char(LocateUTF8Char::RuntimeNext));

        Ok(())
    }
}

impl NextItem for StrSliceType {
    fn init_address(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        instructions.push(Casm::Locate(Locate {
            address: MemoryAddress::Stack {
                offset: Offset::ST(-(self.size_of() as isize)),
                level: AccessLevel::Direct,
            },
        }));
        Ok(())
    }
    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        instructions.push(Casm::Data(Data::Serialized {
            data: (self.size_of() as u64 - 8).to_le_bytes().into(),
        }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Addition(Addition {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        instructions.push(Casm::Mem(Mem::Dup(8)));
        instructions.push(Casm::Data(Data::Serialized {
            data: (self.size_of() as u64 - 8).to_le_bytes().into(),
        }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Substraction(Substraction {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
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
        instructions.push(Casm::Mem(Mem::Dup(16)));

        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Greater(Greater {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        instructions.push(Casm::If(BranchIf {
            else_label: end_label,
        }));

        instructions.push(Casm::Mem(Mem::Dup(8)));
        instructions.push(Casm::Access(Access::RuntimeCharUTF8));
        instructions.push(Casm::Mem(Mem::Dup(4)));
        instructions.push(Casm::Data(Data::Serialized {
            data: [0; 4].into(),
        }));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::NotEqual(NotEqual { left: 4, right: 4 }),
        }));
        instructions.push(Casm::If(BranchIf {
            else_label: end_label,
        }));
        /* STACK : UPPER | INDEX (8) | CHAR (4) */
        Ok(())
    }
    fn next(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        /* STACK : UPPER | INDEX */
        instructions.push(Casm::LocateUTF8Char(LocateUTF8Char::RuntimeNext));

        Ok(())
    }
}

impl NextItem for VecType {
    fn init_address(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        Ok(())
    }
    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        /* STACK : POINTER TO VEC */
        instructions.push(Casm::Mem(Mem::Dup(8)));
        instructions.push(Casm::Mem(Mem::Dup(8)));
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
        instructions.push(Casm::Mem(Mem::Dup(16)));

        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Greater(Greater {
                left: OpPrimitive::Number(NumberType::U64),
                right: OpPrimitive::Number(NumberType::U64),
            }),
        }));
        instructions.push(Casm::If(BranchIf {
            else_label: end_label,
        }));

        instructions.push(Casm::Mem(Mem::Dup(8)));
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
    fn init_address(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        Ok(())
    }

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
        instructions.push(Casm::Mem(Mem::Dup(self.num.size_of())));

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
        instructions.push(Casm::Mem(Mem::Dup(self.num.size_of())));
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
