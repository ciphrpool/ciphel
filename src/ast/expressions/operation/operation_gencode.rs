use crate::ast::expressions::locate::Locatable;
use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::scope::static_types::{ClosureType, SliceType, StrSliceType, POINTER_SIZE};
use crate::vm::allocator::MemoryAddress;
use crate::vm::asm::alloc::Access;
use crate::vm::asm::branch::Call;
use crate::vm::asm::data;
use crate::vm::asm::locate::{Locate, LocateIndex, LocateOffset};
use crate::vm::asm::mem::Mem;
use crate::{
    semantic::{
        scope::static_types::{NumberType, RangeType, StaticType},
        EType, SizeOf, TypeOf,
    },
    vm::{
        asm::{
            data::Data,
            operation::{
                Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Division, Equal, Greater,
                GreaterEqual, Less, LessEqual, LogicalAnd, LogicalOr, Minus, Mod, Mult, Not,
                NotEqual, OpPrimitive, Operation, OperationKind, ShiftLeft, ShiftRight,
                Substraction,
            },
            Asm, Program,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{ExprCall, FieldAccess, ListAccess, Range, TupleAccess};

impl GenerateCode for super::UnaryOperation {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            super::UnaryOperation::Minus { value, metadata: _ } => {
                let Some(value_type) = value.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = value.gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::Operation(Operation {
                    kind: OperationKind::Minus(Minus {
                        data_type: value_type.try_into()?,
                    }),
                }));
                Ok(())
            }
            super::UnaryOperation::Not { value, metadata: _ } => {
                let _ = value.gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::Operation(Operation {
                    kind: OperationKind::Not(Not()),
                }));
                Ok(())
            }
        }
    }
}

impl GenerateCode for TupleAccess {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(item_type) = self.metadata.signature() else {
            return Err(CodeGenerationError::Unlocatable);
        };

        let Some(offset) = self.offset else {
            return Err(CodeGenerationError::Unlocatable);
        };

        let size = item_type.size_of();

        match self.var.locate(scope_manager, scope_id, instructions)? {
            Some(address) => {
                // the address is static
                instructions.push(Asm::Access(Access::Static {
                    address: address.add(offset),
                    size,
                }))
            }
            None => {
                // the address was pushed on the stack
                instructions.push(Asm::Offset(LocateOffset { offset }));
                instructions.push(Asm::Access(Access::Runtime { size: Some(size) }));
            }
        }
        Ok(())
    }
}

impl GenerateCode for ListAccess {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(item_type) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(array_type) = self.var.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let offset = match array_type {
            EType::Static(StaticType::Vec(_)) => crate::vm::core::vector::VEC_HEADER,
            EType::Static(StaticType::Slice(_)) => 0,
            _ => return Err(CodeGenerationError::UnresolvedError),
        };

        let size = item_type.size_of();

        match self.var.locate(scope_manager, scope_id, instructions)? {
            Some(address) => {
                // the address is static
                let _ = self
                    .index
                    .gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::OffsetIdx(LocateIndex {
                    size,
                    base_address: Some(address),
                    offset: Some(offset),
                }));

                instructions.push(Asm::Access(Access::Runtime { size: Some(size) }));
            }
            None => {
                // the address was pushed on the stack
                instructions.push(Asm::Access(Access::Runtime {
                    size: Some(POINTER_SIZE),
                }));

                let _ = self
                    .index
                    .gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::OffsetIdx(LocateIndex {
                    size,
                    base_address: None,
                    offset: Some(offset),
                }));
                instructions.push(Asm::Access(Access::Runtime { size: Some(size) }));
            }
        }
        Ok(())
    }
}

impl GenerateCode for FieldAccess {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self.var.locate(scope_manager, scope_id, instructions)? {
            Some(address) => {
                // the address is static
                self.field
                    .access_from(scope_manager, scope_id, instructions, address)?;
            }
            None => {
                // the address was pushed on the stack
                self.field
                    .runtime_access(scope_manager, scope_id, instructions)?;
            }
        }
        Ok(())
    }
}

impl GenerateCode for ExprCall {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        for arg in self.args.args.iter() {
            arg.gencode(scope_manager, scope_id, instructions, context)?;
        }
        self.var
            .gencode(scope_manager, scope_id, instructions, context)?;

        let Some(param_size) = self.args.size else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let Some(var_type) = self.var.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        match var_type {
            EType::Static(StaticType::Closure(_)) => {
                instructions.push(Asm::Call(crate::vm::asm::branch::Call::Closure {
                    param_size,
                }));
            }
            EType::Static(StaticType::Function(_) | StaticType::Lambda(_)) => {
                instructions.push(Asm::Call(crate::vm::asm::branch::Call::Function {
                    param_size,
                }));
            }
            _ => return Err(CodeGenerationError::UnresolvedError),
        }

        Ok(())
    }
}

impl GenerateCode for Range {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(signature) = self.metadata.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let (_num_type, incr_data) = match signature {
            EType::Static(value) => match value {
                StaticType::Range(RangeType { num, .. }) => (
                    num.size_of(),
                    match num {
                        NumberType::U8 => (1u8).to_le_bytes().into(),
                        NumberType::U16 => (1u16).to_le_bytes().into(),
                        NumberType::U32 => (1u32).to_le_bytes().into(),
                        NumberType::U64 => (1u64).to_le_bytes().into(),
                        NumberType::U128 => (1u128).to_le_bytes().into(),
                        NumberType::I8 => (1i8).to_le_bytes().into(),
                        NumberType::I16 => (1i16).to_le_bytes().into(),
                        NumberType::I32 => (1i32).to_le_bytes().into(),
                        NumberType::I64 => (1i64).to_le_bytes().into(),
                        NumberType::I128 => (1i128).to_le_bytes().into(),
                        NumberType::F64 => (1f64).to_le_bytes().into(),
                    },
                ),
                _ => return Err(CodeGenerationError::UnresolvedError),
            },
            _ => return Err(CodeGenerationError::UnresolvedError),
        };

        let _ = self
            .lower
            .gencode(scope_manager, scope_id, instructions, context)?;
        let _ = self
            .upper
            .gencode(scope_manager, scope_id, instructions, context)?;
        instructions.push(Asm::Data(Data::Serialized { data: incr_data }));
        Ok(())
    }
}

impl GenerateCode for super::Product {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            super::Product::Mult {
                left,
                right,
                metadata: _,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope_manager, scope_id, instructions, context)?;
                let _ = right.gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::Operation(Operation {
                    kind: OperationKind::Mult(Mult {
                        left: left_type.try_into()?,
                        right: right_type.try_into()?,
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                Ok(())
            }
            super::Product::Div {
                left,
                right,
                metadata: _,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope_manager, scope_id, instructions, context)?;
                let _ = right.gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::Operation(Operation {
                    kind: OperationKind::Div(Division {
                        left: left_type.try_into()?,
                        right: right_type.try_into()?,
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                Ok(())
            }
            super::Product::Mod {
                left,
                right,
                metadata: _,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope_manager, scope_id, instructions, context)?;
                let _ = right.gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::Operation(Operation {
                    kind: OperationKind::Mod(Mod {
                        left: left_type.try_into()?,
                        right: right_type.try_into()?,
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                Ok(())
            }
        }
    }
}

impl GenerateCode for super::Addition {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self
            .left
            .gencode(scope_manager, scope_id, instructions, context)?;
        let _ = self
            .right
            .gencode(scope_manager, scope_id, instructions, context)?;

        instructions.push(Asm::Operation(Operation {
            kind: OperationKind::Addition(Addition {
                left: left_type.try_into()?,
                right: right_type.try_into()?,
            }),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

impl GenerateCode for super::Substraction {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self
            .left
            .gencode(scope_manager, scope_id, instructions, context)?;
        let _ = self
            .right
            .gencode(scope_manager, scope_id, instructions, context)?;

        instructions.push(Asm::Operation(Operation {
            kind: OperationKind::Substraction(Substraction {
                left: left_type.try_into()?,
                right: right_type.try_into()?,
            }),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

impl GenerateCode for super::Shift {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            super::Shift::Left {
                left,
                right,
                metadata: _,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope_manager, scope_id, instructions, context)?;
                let _ = right.gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::Operation(Operation {
                    kind: OperationKind::ShiftLeft(ShiftLeft {
                        left: left_type.try_into()?,
                        right: right_type.try_into()?,
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                Ok(())
            }
            super::Shift::Right {
                left,
                right,
                metadata: _,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope_manager, scope_id, instructions, context)?;
                let _ = right.gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::Operation(Operation {
                    kind: OperationKind::ShiftRight(ShiftRight {
                        left: left_type.try_into()?,
                        right: right_type.try_into()?,
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                Ok(())
            }
        }
    }
}

impl GenerateCode for super::BitwiseAnd {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self
            .left
            .gencode(scope_manager, scope_id, instructions, context)?;
        let _ = self
            .right
            .gencode(scope_manager, scope_id, instructions, context)?;

        instructions.push(Asm::Operation(Operation {
            kind: OperationKind::BitwiseAnd(BitwiseAnd {
                left: left_type.try_into()?,
                right: right_type.try_into()?,
            }),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

impl GenerateCode for super::BitwiseXOR {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self
            .left
            .gencode(scope_manager, scope_id, instructions, context)?;
        let _ = self
            .right
            .gencode(scope_manager, scope_id, instructions, context)?;

        instructions.push(Asm::Operation(Operation {
            kind: OperationKind::BitwiseXOR(BitwiseXOR {
                left: left_type.try_into()?,
                right: right_type.try_into()?,
            }),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

impl GenerateCode for super::BitwiseOR {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self
            .left
            .gencode(scope_manager, scope_id, instructions, context)?;
        let _ = self
            .right
            .gencode(scope_manager, scope_id, instructions, context)?;

        instructions.push(Asm::Operation(Operation {
            kind: OperationKind::BitwiseOR(BitwiseOR {
                left: left_type.try_into()?,
                right: right_type.try_into()?,
            }),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

impl GenerateCode for super::Cast {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.type_of(&scope_manager, scope_id).ok() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self
            .left
            .gencode(scope_manager, scope_id, instructions, context)?;

        let op_left_type: Result<OpPrimitive, CodeGenerationError> = left_type.try_into();
        let op_right_type: Result<OpPrimitive, CodeGenerationError> = right_type.try_into();

        if op_left_type.is_ok() && op_right_type.is_ok() {
            instructions.push(Asm::Operation(Operation {
                kind: OperationKind::Cast(Cast {
                    from: op_left_type.unwrap(),
                    to: op_right_type.unwrap(),
                }),
                // result: OpPrimitive::Number(NumberType::U64),
            }));
        }

        Ok(())
    }
}

impl GenerateCode for super::Comparaison {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            super::Comparaison::Less {
                left,
                right,
                metadata: _,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope_manager, scope_id, instructions, context)?;
                let _ = right.gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::Operation(Operation {
                    kind: OperationKind::Less(Less {
                        left: left_type.try_into()?,
                        right: right_type.try_into()?,
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                Ok(())
            }
            super::Comparaison::LessEqual {
                left,
                right,
                metadata: _,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope_manager, scope_id, instructions, context)?;
                let _ = right.gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::Operation(Operation {
                    kind: OperationKind::LessEqual(LessEqual {
                        left: left_type.try_into()?,
                        right: right_type.try_into()?,
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                Ok(())
            }
            super::Comparaison::Greater {
                left,
                right,
                metadata: _,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope_manager, scope_id, instructions, context)?;
                let _ = right.gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::Operation(Operation {
                    kind: OperationKind::Greater(Greater {
                        left: left_type.try_into()?,
                        right: right_type.try_into()?,
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                Ok(())
            }
            super::Comparaison::GreaterEqual {
                left,
                right,
                metadata: _,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope_manager, scope_id, instructions, context)?;
                let _ = right.gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::Operation(Operation {
                    kind: OperationKind::GreaterEqual(GreaterEqual {
                        left: left_type.try_into()?,
                        right: right_type.try_into()?,
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                Ok(())
            }
        }
    }
}

impl GenerateCode for super::Equation {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            super::Equation::Equal {
                left,
                right,
                metadata: _,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope_manager, scope_id, instructions, context)?;
                let _ = right.gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::Operation(Operation {
                    kind: OperationKind::Equal(Equal {
                        left: left_type.size_of(),
                        right: right_type.size_of(),
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                Ok(())
            }
            super::Equation::NotEqual {
                left,
                right,
                metadata: _,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope_manager, scope_id, instructions, context)?;
                let _ = right.gencode(scope_manager, scope_id, instructions, context)?;

                instructions.push(Asm::Operation(Operation {
                    kind: OperationKind::NotEqual(NotEqual {
                        left: left_type.size_of(),
                        right: right_type.size_of(),
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                Ok(())
            }
        }
    }
}

impl GenerateCode for super::LogicalAnd {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let _ = self
            .left
            .gencode(scope_manager, scope_id, instructions, context)?;
        let _ = self
            .right
            .gencode(scope_manager, scope_id, instructions, context)?;

        instructions.push(Asm::Operation(Operation {
            kind: OperationKind::LogicalAnd(LogicalAnd()),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

impl GenerateCode for super::LogicalOr {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut Program,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let _ = self
            .left
            .gencode(scope_manager, scope_id, instructions, context)?;
        let _ = self
            .right
            .gencode(scope_manager, scope_id, instructions, context)?;

        instructions.push(Asm::Operation(Operation {
            kind: OperationKind::LogicalOr(LogicalOr()),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{
            expressions::{
                data::{Number, Primitive, StrSlice},
                Expression,
            },
            statements::Statement,
            TryParse,
        },
        semantic::{
            scope::{
                scope::ScopeManager,
                static_types::{PrimitiveType, StrSliceType},
            },
            Resolve,
        },
        test_extract_variable, test_statements, v_num,
        vm::vm::Runtime,
    };

    use super::*;

    #[test]
    fn valid_addition() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<u128>("var_u128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1 + 3);
            let res = test_extract_variable::<u64>("var_u64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2 + 3);
            let res = test_extract_variable::<u32>("var_u32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3 + 3);
            let res = test_extract_variable::<u16>("var_u16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 4 + 3);
            let res = test_extract_variable::<u8>("var_u8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5 + 3);
            let res = test_extract_variable::<i128>("var_i128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 6 + 3);
            let res = test_extract_variable::<i64>("var_i64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 7 + 3);
            let res = test_extract_variable::<i32>("var_i32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 8 + 3);
            let res = test_extract_variable::<i16>("var_i16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 9 + 3);
            let res = test_extract_variable::<i8>("var_i8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 10 + 3);
            true
        }

        test_statements(
            r##"
        
        let var_u128 = 1u128 + 3u128;
        let var_u64 = 2 + 3u64;
        let var_u32 = 3 + 3u32;
        let var_u16 = 4u16 + 3u16;
        let var_u8 = 5u8 + 3;
        let var_i128 = 6i128 + 3;
        let var_i64 = 7 + 3;
        let var_i32 = 8i32 + 3i32;
        let var_i16 : i16 = 9 + 3;
        let var_i8 : i8 = 10i8 + 3;
        
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_substraction() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<u128>("var_u128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 10 - 3);
            let res = test_extract_variable::<u64>("var_u64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 20 - 3);
            let res = test_extract_variable::<u32>("var_u32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3 - 3);
            let res = test_extract_variable::<u16>("var_u16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 4 - 3);
            let res = test_extract_variable::<u8>("var_u8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5 - 3);
            let res = test_extract_variable::<i128>("var_i128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 6 - 3);
            let res = test_extract_variable::<i64>("var_i64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 7 - 3);
            let res = test_extract_variable::<i32>("var_i32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 8 - 3);
            let res = test_extract_variable::<i16>("var_i16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 9 - 3);
            let res = test_extract_variable::<i8>("var_i8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 10 - 3);
            true
        }

        test_statements(
            r##"
        
        let var_u128 = 10u128 - 3u128;
        let var_u64 = 20 - 3u64;
        let var_u32 = 3 - 3u32;
        let var_u16 = 4u16 - 3u16;
        let var_u8 = 5u8 - 3;
        let var_i128 = 6i128 - 3;
        let var_i64 = 7 - 3;
        let var_i32 = 8i32 - 3i32;
        let var_i16 : i16 = 9 - 3;
        let var_i8 : i8 = 10i8 - 3;
        
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_multiplaction() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<u128>("var_u128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1 * 3);
            let res = test_extract_variable::<u64>("var_u64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2 * 3);
            let res = test_extract_variable::<u32>("var_u32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3 * 3);
            let res = test_extract_variable::<u16>("var_u16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 4 * 3);
            let res = test_extract_variable::<u8>("var_u8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5 * 3);
            let res = test_extract_variable::<i128>("var_i128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 6 * 3);
            let res = test_extract_variable::<i64>("var_i64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 7 * 3);
            let res = test_extract_variable::<i32>("var_i32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 8 * 3);
            let res = test_extract_variable::<i16>("var_i16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 9 * 3);
            let res = test_extract_variable::<i8>("var_i8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 10 * 3);
            true
        }

        test_statements(
            r##"
        
        let var_u128 = 1u128 * 3u128;
        let var_u64 = 2 * 3u64;
        let var_u32 = 3 * 3u32;
        let var_u16 = 4u16 * 3u16;
        let var_u8 = 5u8 * 3;
        let var_i128 = 6i128 * 3;
        let var_i64 = 7 * 3;
        let var_i32 = 8i32 * 3i32;
        let var_i16 : i16 = 9 * 3;
        let var_i8 : i8 = 10i8 * 3;
        
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_division() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<u128>("var_u128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 10 / 3);
            let res = test_extract_variable::<u64>("var_u64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 20 / 3);
            let res = test_extract_variable::<u32>("var_u32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3 / 3);
            let res = test_extract_variable::<u16>("var_u16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 4 / 3);
            let res = test_extract_variable::<u8>("var_u8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5 / 3);
            let res = test_extract_variable::<i128>("var_i128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 6 / 3);
            let res = test_extract_variable::<i64>("var_i64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 7 / 3);
            let res = test_extract_variable::<i32>("var_i32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 8 / 3);
            let res = test_extract_variable::<i16>("var_i16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 9 / 3);
            let res = test_extract_variable::<i8>("var_i8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 10 / 3);
            true
        }

        test_statements(
            r##"
        
        let var_u128 = 10u128 / 3u128;
        let var_u64 = 20 / 3u64;
        let var_u32 = 3 / 3u32;
        let var_u16 = 4u16 / 3u16;
        let var_u8 = 5u8 / 3;
        let var_i128 = 6i128 / 3;
        let var_i64 = 7 / 3;
        let var_i32 = 8i32 / 3i32;
        let var_i16 : i16 = 9 / 3;
        let var_i8 : i8 = 10i8 / 3;
        
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_shift() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<u128>("var_u128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1 >> 3);
            let res = test_extract_variable::<u64>("var_u64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2 << 3);
            let res = test_extract_variable::<u32>("var_u32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3 >> 3);
            let res = test_extract_variable::<u16>("var_u16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 4 << 3);
            let res = test_extract_variable::<u8>("var_u8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5 >> 3);
            let res = test_extract_variable::<i128>("var_i128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 6 << 3);
            let res = test_extract_variable::<i64>("var_i64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 7 >> 3);
            let res = test_extract_variable::<i32>("var_i32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 8 << 3);
            let res = test_extract_variable::<i16>("var_i16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 9 >> 3);
            let res = test_extract_variable::<i8>("var_i8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 10 << 3);
            true
        }

        test_statements(
            r##"
        
        let var_u128 = 1u128 >> 3u128;
        let var_u64 = 2 << 3u64;
        let var_u32 = 3 >> 3u32;
        let var_u16 = 4u16 << 3u16;
        let var_u8 = 5u8 >> 3;
        let var_i128 = 6i128 << 3;
        let var_i64 = 7 >> 3;
        let var_i32 = 8i32 << 3i32;
        let var_i16 : i16 = 9 >> 3;
        let var_i8 : i8 = 10i8 << 3;
        
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_band_bor_bxor() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<u128>("var_u128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1 & 3);
            let res = test_extract_variable::<u64>("var_u64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2 | 3);
            let res = test_extract_variable::<u32>("var_u32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3 ^ 3);
            let res = test_extract_variable::<u16>("var_u16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 4 | 3);
            let res = test_extract_variable::<u8>("var_u8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5 & 3);
            let res = test_extract_variable::<i128>("var_i128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 6 ^ 3);
            let res = test_extract_variable::<i64>("var_i64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 7 & 3);
            let res = test_extract_variable::<i32>("var_i32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 8 | 3);
            let res = test_extract_variable::<i16>("var_i16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 9 ^ 3);
            let res = test_extract_variable::<i8>("var_i8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 10 | 3);
            true
        }

        test_statements(
            r##"
        
        let var_u128 = 1u128 & 3u128;
        let var_u64 = 2 | 3u64;
        let var_u32 = 3 ^ 3u32;
        let var_u16 = 4u16 | 3u16;
        let var_u8 = 5u8 & 3;
        let var_i128 = 6i128 ^ 3;
        let var_i64 = 7 & 3;
        let var_i32 = 8i32 | 3i32;
        let var_i16 : i16 = 9 ^ 3;
        let var_i8 : i8 = 10i8 | 3;
        
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_cmp() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<u8>("var_u128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, 1 == 3);
            let res = test_extract_variable::<u8>("var_u64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, 2 != 3);
            let res = test_extract_variable::<u8>("var_u32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, 3 > 3);
            let res = test_extract_variable::<u8>("var_u16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, 4 >= 3);
            let res = test_extract_variable::<u8>("var_u8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, 5 < 3);
            let res = test_extract_variable::<u8>("var_i128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, 6 <= 3);
            let res = test_extract_variable::<u8>("var_i64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, 7 == 3);
            let res = test_extract_variable::<u8>("var_i32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, 8 != 3);
            let res = test_extract_variable::<u8>("var_i16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, 9 < 3);
            let res = test_extract_variable::<u8>("var_i8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, 10 > 3);

            let res = test_extract_variable::<u8>("var_and1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, true && false);
            let res = test_extract_variable::<u8>("var_and2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, true && true);
            let res = test_extract_variable::<u8>("var_and3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, false && false);
            let res = test_extract_variable::<u8>("var_and4", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, false && true);

            let res = test_extract_variable::<u8>("var_or1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, true || false);
            let res = test_extract_variable::<u8>("var_or2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, true || true);
            let res = test_extract_variable::<u8>("var_or3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, false || false);
            let res = test_extract_variable::<u8>("var_or4", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, false || true);
            true
        }

        test_statements(
            r##"
        
        let var_u128 = 1u128 == 3u128;
        let var_u64 = 2 != 3u64;
        let var_u32 = 3 > 3u32;
        let var_u16 = 4u16 >= 3u16;
        let var_u8 = 5u8 < 3;
        let var_i128 = 6i128 <= 3;
        let var_i64 = 7 == 3;
        let var_i32 = 8i32 != 3i32;
        let var_i16 = 9 < 3;
        let var_i8 = 10i8 > 3;
        let var_and1 = true and false;
        let var_and2 = true and true;
        let var_and3 = false and false;
        let var_and4 = false and true;
        let var_or1 = true or false;
        let var_or2 = true or true;
        let var_or3 = false or false;
        let var_or4 = false or true;

        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_neg() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i128>("var_i128", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, -5);
            let res = test_extract_variable::<i64>("var_i64", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, -5);
            let res = test_extract_variable::<i32>("var_i32", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1 + -5);
            let res = test_extract_variable::<i16>("var_i16", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1 + (-5));
            let res = test_extract_variable::<i8>("var_i8", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, -5 + 1);

            let res = test_extract_variable::<u8>("var_neg1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, !true);
            let res = test_extract_variable::<u8>("var_neg2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res != 0, !false);
            true
        }

        test_statements(
            r##"
        
        let var_i128 = -5i128;
        let var_i64 = -5i64;
        let var_i32 = 1i32 + -5 as i32;
        let var_i16 = 1 + (-5i16);
        let var_i8 = -5 + 1i8;

        let var_neg1 = ! true;
        let var_neg2 = !false;

        "##,
            &mut engine,
            assert_fn,
        );
    }

    // #[test]
    // fn valid_cast() {
    //    todo!()
    // }

    #[test]
    fn valid_tuple_access() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("x", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1);
            let res = test_extract_variable::<i64>("y", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("z", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3);
            true
        }

        test_statements(
            r##"
        let t = (1,2,3);
        let x = t.0;
        let y = t.1;
        let z = t.2;

        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_field_access() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("x", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1);
            let res = test_extract_variable::<i64>("y", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("z", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3);
            true
        }

        test_statements(
            r##"
        struct Point {
            x :i64,
            y :i64,
            z :i64,
        }

        let t = Point{x:1,y:2,z:3};
        let x = t.x;
        let y = t.y;
        let z = t.z;

        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_slice_access() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("x", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1);
            let res = test_extract_variable::<i64>("y", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("z", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3);
            true
        }

        test_statements(
            r##"
        
        let t = [1,2,3];
        let x = t[0];
        let y = t[1];
        let z = t[2];

        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_vec_access() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("x", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1);
            let res = test_extract_variable::<i64>("y", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("z", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3);
            true
        }

        test_statements(
            r##"
        
        let t = vec[1,2,3];
        let x = t[0];
        let y = t[1];
        let z = t[2];

        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_complex_access() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("x", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1);
            let res = test_extract_variable::<i64>("y", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("z", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3);
            true
        }

        test_statements(
            r##"
        struct Point {
            x :i64,
            y :i64,
            z :i64,
        }

        struct Test {
            tuple : ([4]i64,i64,Point)
        }

        let t = Test{
            tuple : ([1,2,3,4],2,Point{x:1,y:2,z:3})
        };

        let x = t.tuple.0[0];
        let y = t.tuple.1;
        let z = t.tuple.2.z;

        "##,
            &mut engine,
            assert_fn,
        );
    }
}
