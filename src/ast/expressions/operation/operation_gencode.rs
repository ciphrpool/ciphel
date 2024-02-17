use crate::{
    semantic::{
        scope::{static_types::NumberType, ScopeApi},
        MutRc, SizeOf, TypeOf,
    },
    vm::{
        casm::{
            operation::{
                Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Division, Equal, Greater,
                GreaterEqual, Less, LessEqual, LogicalAnd, LogicalOr, Minus, Mod, Mult, Not,
                NotEqual, OpPrimitive, Operation, OperationKind, ShiftLeft, ShiftRight,
                Substraction,
            },
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

impl<Scope: ScopeApi> GenerateCode<Scope> for super::UnaryOperation<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        match self {
            super::UnaryOperation::Minus { value, metadata } => {
                let Some(value_type) = value.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                dbg!(&value_type);
                let _ = value.gencode(scope, instructions)?;

                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Operation(Operation {
                    kind: OperationKind::Minus(Minus {
                        data_type: value_type.try_into()?,
                    }),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                Ok(())
            }
            super::UnaryOperation::Not { value, metadata } => {
                let _ = value.gencode(scope, instructions)?;

                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Operation(Operation {
                    kind: OperationKind::Not(Not()),
                    // result: OpPrimitive::Number(NumberType::U64),
                }));
                Ok(())
            }
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for super::Product<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        match self {
            super::Product::Mult {
                left,
                right,
                metadata,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope, instructions)?;
                let _ = right.gencode(scope, instructions)?;

                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Operation(Operation {
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
                metadata,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope, instructions)?;
                let _ = right.gencode(scope, instructions)?;

                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Operation(Operation {
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
                metadata,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope, instructions)?;
                let _ = right.gencode(scope, instructions)?;

                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Operation(Operation {
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

impl<Scope: ScopeApi> GenerateCode<Scope> for super::Addition<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self.left.gencode(scope, instructions)?;
        let _ = self.right.gencode(scope, instructions)?;

        let mut borrowed = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;
        borrowed.push(Casm::Operation(Operation {
            kind: OperationKind::Addition(Addition {
                left: left_type.try_into()?,
                right: right_type.try_into()?,
            }),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for super::Substraction<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self.left.gencode(scope, instructions)?;
        let _ = self.right.gencode(scope, instructions)?;

        let mut borrowed = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;
        borrowed.push(Casm::Operation(Operation {
            kind: OperationKind::Substraction(Substraction {
                left: left_type.try_into()?,
                right: right_type.try_into()?,
            }),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for super::Shift<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        match self {
            super::Shift::Left {
                left,
                right,
                metadata,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope, instructions)?;
                let _ = right.gencode(scope, instructions)?;

                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Operation(Operation {
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
                metadata,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope, instructions)?;
                let _ = right.gencode(scope, instructions)?;

                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Operation(Operation {
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

impl<Scope: ScopeApi> GenerateCode<Scope> for super::BitwiseAnd<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self.left.gencode(scope, instructions)?;
        let _ = self.right.gencode(scope, instructions)?;

        let mut borrowed = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;
        borrowed.push(Casm::Operation(Operation {
            kind: OperationKind::BitwiseAnd(BitwiseAnd {
                left: left_type.try_into()?,
                right: right_type.try_into()?,
            }),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for super::BitwiseXOR<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self.left.gencode(scope, instructions)?;
        let _ = self.right.gencode(scope, instructions)?;

        let mut borrowed = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;
        borrowed.push(Casm::Operation(Operation {
            kind: OperationKind::BitwiseXOR(BitwiseXOR {
                left: left_type.try_into()?,
                right: right_type.try_into()?,
            }),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for super::BitwiseOR<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self.left.gencode(scope, instructions)?;
        let _ = self.right.gencode(scope, instructions)?;

        let mut borrowed = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;
        borrowed.push(Casm::Operation(Operation {
            kind: OperationKind::BitwiseOR(BitwiseOR {
                left: left_type.try_into()?,
                right: right_type.try_into()?,
            }),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for super::Cast<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.type_of(&scope.borrow()).ok() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self.left.gencode(scope, instructions)?;

        let mut borrowed = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;
        borrowed.push(Casm::Operation(Operation {
            kind: OperationKind::Cast(Cast {
                from: left_type.try_into()?,
                to: right_type.try_into()?,
            }),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for super::Comparaison<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        match self {
            super::Comparaison::Less {
                left,
                right,
                metadata,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope, instructions)?;
                let _ = right.gencode(scope, instructions)?;

                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Operation(Operation {
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
                metadata,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope, instructions)?;
                let _ = right.gencode(scope, instructions)?;

                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Operation(Operation {
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
                metadata,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope, instructions)?;
                let _ = right.gencode(scope, instructions)?;

                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Operation(Operation {
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
                metadata,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope, instructions)?;
                let _ = right.gencode(scope, instructions)?;

                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Operation(Operation {
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

impl<Scope: ScopeApi> GenerateCode<Scope> for super::Equation<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        match self {
            super::Equation::Equal {
                left,
                right,
                metadata,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope, instructions)?;
                let _ = right.gencode(scope, instructions)?;

                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Operation(Operation {
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
                metadata,
            } => {
                let Some(left_type) = left.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = left.gencode(scope, instructions)?;
                let _ = right.gencode(scope, instructions)?;

                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Operation(Operation {
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

impl<Scope: ScopeApi> GenerateCode<Scope> for super::Inclusion<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for super::LogicalAnd<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.type_of(&scope.borrow()).ok() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self.left.gencode(scope, instructions)?;

        let mut borrowed = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;
        borrowed.push(Casm::Operation(Operation {
            kind: OperationKind::LogicalAnd(LogicalAnd()),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for super::LogicalOr<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        let Some(left_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let Some(right_type) = self.right.type_of(&scope.borrow()).ok() else {
            return Err(CodeGenerationError::UnresolvedError);
        };
        let _ = self.left.gencode(scope, instructions)?;

        let mut borrowed = instructions
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| CodeGenerationError::Default)?;
        borrowed.push(Casm::Operation(Operation {
            kind: OperationKind::LogicalOr(LogicalOr()),
            // result: OpPrimitive::Number(NumberType::U64),
        }));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use crate::{
        ast::{
            expressions::{
                data::{Number, Primitive, StringData},
                Expression,
            },
            TryParse,
        },
        clear_stack,
        semantic::{
            scope::{
                scope_impl::Scope,
                static_types::{PrimitiveType, StringType},
            },
            Resolve,
        },
        vm::{allocator::Memory, vm::DeserializeFrom},
    };

    use super::*;
    #[macro_export]
    macro_rules! eval_and_compare {
        ($expr:expr, $expected:expr,$size:ident) => {{
            // Assuming `Expression`, `Scope`, `CasmProgram`, `Memory`, and `Primitive` are defined in the context.
            let expr = Expression::parse($expr.into()).expect("Parsing should have succeeded").1;

            let scope = Scope::new();
            let _ = expr
                .resolve(&scope, &None, &())
                .expect("Semantic resolution should have succeeded");

            // Code generation.
            let instructions = std::rc::Rc::new(std::cell::RefCell::new(CasmProgram::default()));
            expr.gencode(&scope, &instructions)
                .expect("Code generation should have succeeded");

            let instructions = instructions.borrow().clone();
            assert!(instructions.len() > 0, "No instructions generated");

            // Execute the instructions.
            let memory = Memory::new();
            instructions.execute(&memory).expect("Execution should have succeeded");
            let data = clear_stack!(memory);
            let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
                &PrimitiveType::Number(NumberType::$size),
                &data,
            )
            .expect("Deserialization should have succeeded");

            assert_eq!(result, $expected, "Result does not match the expected value");
        }};
    }

    #[macro_export]
    macro_rules! eval_and_compare_bool {
        ($expr:expr, $expected:expr) => {{
            // Assuming `Expression`, `Scope`, `CasmProgram`, `Memory`, and `Primitive` are defined in the context.
            let expr = Expression::parse($expr.into()).expect("Parsing should have succeeded").1;

            let scope = Scope::new();
            let _ = expr
                .resolve(&scope, &None, &())
                .expect("Semantic resolution should have succeeded");

            // Code generation.
            let instructions = std::rc::Rc::new(std::cell::RefCell::new(CasmProgram::default()));
            expr.gencode(&scope, &instructions)
                .expect("Code generation should have succeeded");

            let instructions = instructions.borrow().clone();
            assert!(instructions.len() > 0, "No instructions generated");

            // Execute the instructions.
            let memory = Memory::new();
            instructions.execute(&memory).expect("Execution should have succeeded");
            let data = clear_stack!(memory);
            let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
                &PrimitiveType::Bool,
                &data,
            )
            .expect("Deserialization should have succeeded");

            assert_eq!(result, $expected, "Result does not match the expected value");
        }};
    }

    #[test]
    fn valid_operation_u128() {
        eval_and_compare!(
            r##"400u128 + 20u128"##,
            Primitive::Number(Cell::new(Number::U128(420))),
            U128
        );
        eval_and_compare!(
            r##"400u128 - 20u128"##,
            Primitive::Number(Cell::new(Number::U128(400 - 20))),
            U128
        );
        eval_and_compare!(
            r##"400u128 * 20u128"##,
            Primitive::Number(Cell::new(Number::U128(400 * 20))),
            U128
        );
        eval_and_compare!(
            r##"400u128 / 20u128"##,
            Primitive::Number(Cell::new(Number::U128(400 / 20))),
            U128
        );
        eval_and_compare!(
            r##"400u128 % 20u128"##,
            Primitive::Number(Cell::new(Number::U128(400 % 20))),
            U128
        );
        eval_and_compare!(
            r##"400u128 << 20u128"##,
            Primitive::Number(Cell::new(Number::U128(400u128 << 20u128))),
            U128
        );
        eval_and_compare!(
            r##"400u128 >> 20u128"##,
            Primitive::Number(Cell::new(Number::U128(400u128 >> 20u128))),
            U128
        );
        eval_and_compare!(
            r##"428u128 & 428u128"##,
            Primitive::Number(Cell::new(Number::U128(428u128 & 428u128))),
            U128
        );
        eval_and_compare!(
            r##"400u128 | 420u128"##,
            Primitive::Number(Cell::new(Number::U128(400u128 | 420u128))),
            U128
        );
        eval_and_compare!(
            r##"400u128 ^ 420u128"##,
            Primitive::Number(Cell::new(Number::U128(400u128 ^ 420u128))),
            U128
        );
        eval_and_compare!(
            r##"400u128 as u64"##,
            Primitive::Number(Cell::new(Number::U64(400))),
            U64
        );
        eval_and_compare_bool!(r##"20u128 > 2u128"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u128 > 20u128"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u128 >= 2u128"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u128 >= 20u128"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"2u128 <= 20u128"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u128 <= 2u128"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u128 > 2u128"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u128 > 20u128"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u128 == 20u128"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u128 == 2u128"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u128 != 2u128"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u128 != 20u128"##, Primitive::Bool(false));
    }

    #[test]
    fn valid_operation_u64() {
        eval_and_compare!(
            r##"400 + 20"##,
            Primitive::Number(Cell::new(Number::U64(420))),
            U64
        );
        eval_and_compare!(
            r##"400 - 20"##,
            Primitive::Number(Cell::new(Number::U64(400 - 20))),
            U64
        );
        eval_and_compare!(
            r##"400 * 20"##,
            Primitive::Number(Cell::new(Number::U64(400 * 20))),
            U64
        );
        eval_and_compare!(
            r##"400 / 20"##,
            Primitive::Number(Cell::new(Number::U64(400 / 20))),
            U64
        );
        eval_and_compare!(
            r##"400 % 20"##,
            Primitive::Number(Cell::new(Number::U64(400 % 20))),
            U64
        );
        eval_and_compare!(
            r##"400u64 << 20u64"##,
            Primitive::Number(Cell::new(Number::U64(400u64 << 20u64))),
            U64
        );
        eval_and_compare!(
            r##"400u64 >> 20u64"##,
            Primitive::Number(Cell::new(Number::U64(400u64 >> 20u64))),
            U64
        );
        eval_and_compare!(
            r##"428u64 & 428u64"##,
            Primitive::Number(Cell::new(Number::U64(428u64 & 428u64))),
            U64
        );
        eval_and_compare!(
            r##"400u64 | 420u64"##,
            Primitive::Number(Cell::new(Number::U64(400u64 | 420u64))),
            U64
        );
        eval_and_compare!(
            r##"400u64 ^ 420u64"##,
            Primitive::Number(Cell::new(Number::U64(400u64 ^ 420u64))),
            U64
        );
        eval_and_compare_bool!(r##"20u64 > 2u64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u64 > 20u64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u64 >= 2u64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u64 >= 20u64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"2u64 <= 20u64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u64 <= 2u64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u64 > 2u64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u64 > 20u64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u64 == 20u64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u64 == 2u64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u64 != 2u64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u64 != 20u64"##, Primitive::Bool(false));
    }

    #[test]
    fn valid_operation_u32() {
        eval_and_compare!(
            r##"400u32 + 20u32"##,
            Primitive::Number(Cell::new(Number::U32(420))),
            U32
        );
        eval_and_compare!(
            r##"400u32 - 20u32"##,
            Primitive::Number(Cell::new(Number::U32(400 - 20))),
            U32
        );
        eval_and_compare!(
            r##"400u32 * 20u32"##,
            Primitive::Number(Cell::new(Number::U32(400 * 20))),
            U32
        );
        eval_and_compare!(
            r##"400u32 / 20u32"##,
            Primitive::Number(Cell::new(Number::U32(400 / 20))),
            U32
        );
        eval_and_compare!(
            r##"400u32 % 20u32"##,
            Primitive::Number(Cell::new(Number::U32(400 % 20))),
            U32
        );
        eval_and_compare!(
            r##"400u32 << 20u32"##,
            Primitive::Number(Cell::new(Number::U32(400u32 << 20u32))),
            U32
        );
        eval_and_compare!(
            r##"400u32 >> 20u32"##,
            Primitive::Number(Cell::new(Number::U32(400u32 >> 20u32))),
            U32
        );
        eval_and_compare!(
            r##"428u32 & 428u32"##,
            Primitive::Number(Cell::new(Number::U32(428u32 & 428u32))),
            U32
        );
        eval_and_compare!(
            r##"400u32 | 420u32"##,
            Primitive::Number(Cell::new(Number::U32(400u32 | 420u32))),
            U32
        );
        eval_and_compare!(
            r##"400u32 ^ 420u32"##,
            Primitive::Number(Cell::new(Number::U32(400u32 ^ 420u32))),
            U32
        );
        eval_and_compare_bool!(r##"20u32 > 2u32"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u32 > 20u32"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u32 >= 2u32"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u32 >= 20u32"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"2u32 <= 20u32"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u32 <= 2u32"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u32 > 2u32"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u32 > 20u32"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u32 == 20u32"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u32 == 2u32"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u32 != 2u32"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u32 != 20u32"##, Primitive::Bool(false));
    }
    #[test]
    fn valid_operation_u16() {
        eval_and_compare!(
            r##"400u16 + 20u16"##,
            Primitive::Number(Cell::new(Number::U16(420))),
            U16
        );
        eval_and_compare!(
            r##"400u16 - 20u16"##,
            Primitive::Number(Cell::new(Number::U16(400 - 20))),
            U16
        );
        eval_and_compare!(
            r##"400u16 * 20u16"##,
            Primitive::Number(Cell::new(Number::U16(400 * 20))),
            U16
        );
        eval_and_compare!(
            r##"400u16 / 20u16"##,
            Primitive::Number(Cell::new(Number::U16(400 / 20))),
            U16
        );
        eval_and_compare!(
            r##"400u16 % 20u16"##,
            Primitive::Number(Cell::new(Number::U16(400 % 20))),
            U16
        );
        eval_and_compare!(
            r##"400u16 << 2u16"##,
            Primitive::Number(Cell::new(Number::U16(400u16 << 2u16))),
            U16
        );
        eval_and_compare!(
            r##"400u16 >> 2u16"##,
            Primitive::Number(Cell::new(Number::U16(400u16 >> 2u16))),
            U16
        );
        eval_and_compare!(
            r##"428u16 & 428u16"##,
            Primitive::Number(Cell::new(Number::U16(428u16 & 428u16))),
            U16
        );
        eval_and_compare!(
            r##"400u16 | 420u16"##,
            Primitive::Number(Cell::new(Number::U16(400u16 | 420u16))),
            U16
        );
        eval_and_compare!(
            r##"400u16 ^ 420u16"##,
            Primitive::Number(Cell::new(Number::U16(400u16 ^ 420u16))),
            U16
        );
        eval_and_compare_bool!(r##"20u16 > 2u16"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u16 > 20u16"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u16 >= 2u16"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u16 >= 20u16"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"2u16 <= 20u16"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u16 <= 2u16"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u16 > 2u16"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u16 > 20u16"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u16 == 20u16"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u16 == 2u16"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u16 != 2u16"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u16 != 20u16"##, Primitive::Bool(false));
    }
    #[test]
    fn valid_operation_u8() {
        eval_and_compare!(
            r##"100u8 + 20u8"##,
            Primitive::Number(Cell::new(Number::U8(120))),
            U8
        );
        eval_and_compare!(
            r##"50u8 - 2u8"##,
            Primitive::Number(Cell::new(Number::U8(50 - 2))),
            U8
        );
        eval_and_compare!(
            r##"50u8 * 2u8"##,
            Primitive::Number(Cell::new(Number::U8(50 * 2))),
            U8
        );
        eval_and_compare!(
            r##"50u8 / 2u8"##,
            Primitive::Number(Cell::new(Number::U8(50 / 2))),
            U8
        );
        eval_and_compare!(
            r##"50u8 % 2u8"##,
            Primitive::Number(Cell::new(Number::U8(50 % 2))),
            U8
        );
        eval_and_compare!(
            r##"40u8 << 2u8"##,
            Primitive::Number(Cell::new(Number::U8(40u8 << 2u8))),
            U8
        );
        eval_and_compare!(
            r##"40u8 >> 2u8"##,
            Primitive::Number(Cell::new(Number::U8(40u8 >> 2u8))),
            U8
        );
        eval_and_compare!(
            r##"48u8 & 48u8"##,
            Primitive::Number(Cell::new(Number::U8(48u8 & 48u8))),
            U8
        );
        eval_and_compare!(
            r##"40u8 | 42u8"##,
            Primitive::Number(Cell::new(Number::U8(40u8 | 42u8))),
            U8
        );
        eval_and_compare!(
            r##"40u8 ^ 42u8"##,
            Primitive::Number(Cell::new(Number::U8(40u8 ^ 42u8))),
            U8
        );
        eval_and_compare_bool!(r##"20u8 > 2u8"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u8 > 20u8"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u8 >= 2u8"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u8 >= 20u8"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"2u8 <= 20u8"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u8 <= 2u8"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u8 > 2u8"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2u8 > 20u8"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u8 == 20u8"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u8 == 2u8"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20u8 != 2u8"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20u8 != 20u8"##, Primitive::Bool(false));
    }

    #[test]
    fn valid_operation_i128() {
        eval_and_compare!(
            r##"400i128 + 20i128"##,
            Primitive::Number(Cell::new(Number::I128(420))),
            I128
        );
        eval_and_compare!(
            r##"400i128 - 800i128"##,
            Primitive::Number(Cell::new(Number::I128(400 - 800))),
            I128
        );
        eval_and_compare!(
            r##"400i128 * 5i128"##,
            Primitive::Number(Cell::new(Number::I128(400 * 5))),
            I128
        );
        eval_and_compare!(
            r##"400i128 / 2i128"##,
            Primitive::Number(Cell::new(Number::I128(400 / 2))),
            I128
        );
        eval_and_compare!(
            r##"400i128 % 2i128"##,
            Primitive::Number(Cell::new(Number::I128(400 % 2))),
            I128
        );
        eval_and_compare!(
            r##"-20i128"##,
            Primitive::Number(Cell::new(Number::I128(-20))),
            I128
        );
        eval_and_compare!(
            r##"400i128 << 20i128"##,
            Primitive::Number(Cell::new(Number::I128(400i128 << 20i128))),
            I128
        );
        eval_and_compare!(
            r##"400i128 >> 20i128"##,
            Primitive::Number(Cell::new(Number::I128(400i128 >> 20i128))),
            I128
        );
        eval_and_compare!(
            r##"428i128 & 428i128"##,
            Primitive::Number(Cell::new(Number::I128(428i128 & 428i128))),
            I128
        );
        eval_and_compare!(
            r##"400i128 | 420i128"##,
            Primitive::Number(Cell::new(Number::I128(400i128 | 420i128))),
            I128
        );
        eval_and_compare!(
            r##"400i128 ^ 420i128"##,
            Primitive::Number(Cell::new(Number::I128(400i128 ^ 420i128))),
            I128
        );
        eval_and_compare_bool!(r##"20i128 > 2i128"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i128 > 20i128"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i128 >= 2i128"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i128 >= 20i128"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"2i128 <= 20i128"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i128 <= 2i128"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i128 > 2i128"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i128 > 20i128"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i128 == 20i128"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i128 == 2i128"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i128 != 2i128"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i128 != 20i128"##, Primitive::Bool(false));
    }

    #[test]
    fn valid_operation_i64() {
        eval_and_compare!(
            r##"400i64 + 20i64"##,
            Primitive::Number(Cell::new(Number::I64(420))),
            I64
        );
        eval_and_compare!(
            r##"400i64 - 800i64"##,
            Primitive::Number(Cell::new(Number::I64(400 - 800))),
            I64
        );
        eval_and_compare!(
            r##"400i64 * 5i64"##,
            Primitive::Number(Cell::new(Number::I64(400 * 5))),
            I64
        );
        eval_and_compare!(
            r##"400i64 / 2i64"##,
            Primitive::Number(Cell::new(Number::I64(400 / 2))),
            I64
        );
        eval_and_compare!(
            r##"400i64 % 2i64"##,
            Primitive::Number(Cell::new(Number::I64(400 % 2))),
            I64
        );
        eval_and_compare!(
            r##"-20i64"##,
            Primitive::Number(Cell::new(Number::I64(-20))),
            I64
        );
        eval_and_compare!(
            r##"-20"##,
            Primitive::Number(Cell::new(Number::I64(-20))),
            I64
        );
        eval_and_compare!(
            r##"400i64 << 20i64"##,
            Primitive::Number(Cell::new(Number::I64(400i64 << 20i64))),
            I64
        );
        eval_and_compare!(
            r##"400i64 >> 20i64"##,
            Primitive::Number(Cell::new(Number::I64(400i64 >> 20i64))),
            I64
        );
        eval_and_compare!(
            r##"428i64 & 428i64"##,
            Primitive::Number(Cell::new(Number::I64(428i64 & 428i64))),
            I64
        );
        eval_and_compare!(
            r##"400i64 | 420i64"##,
            Primitive::Number(Cell::new(Number::I64(400i64 | 420i64))),
            I64
        );
        eval_and_compare!(
            r##"400i64 ^ 420i64"##,
            Primitive::Number(Cell::new(Number::I64(400i64 ^ 420i64))),
            I64
        );
        eval_and_compare_bool!(r##"20i64 > 2i64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i64 > 20i64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i64 >= 2i64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i64 >= 20i64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"2i64 <= 20i64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i64 <= 2i64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i64 > 2i64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i64 > 20i64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i64 == 20i64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i64 == 2i64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i64 != 2i64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i64 != 20i64"##, Primitive::Bool(false));
    }
    #[test]
    fn valid_operation_i32() {
        eval_and_compare!(
            r##"400i32 + 20i32"##,
            Primitive::Number(Cell::new(Number::I32(420))),
            I32
        );
        eval_and_compare!(
            r##"400i32 - 800i32"##,
            Primitive::Number(Cell::new(Number::I32(400 - 800))),
            I32
        );
        eval_and_compare!(
            r##"400i32 * 5i32"##,
            Primitive::Number(Cell::new(Number::I32(400 * 5))),
            I32
        );
        eval_and_compare!(
            r##"400i32 / 2i32"##,
            Primitive::Number(Cell::new(Number::I32(400 / 2))),
            I32
        );
        eval_and_compare!(
            r##"400i32 % 2i32"##,
            Primitive::Number(Cell::new(Number::I32(400 % 2))),
            I32
        );
        eval_and_compare!(
            r##"-20i32"##,
            Primitive::Number(Cell::new(Number::I32(-20))),
            I32
        );
        eval_and_compare!(
            r##"400i32 << 20i32"##,
            Primitive::Number(Cell::new(Number::I32(400i32 << 20i32))),
            I32
        );
        eval_and_compare!(
            r##"400i32 >> 20i32"##,
            Primitive::Number(Cell::new(Number::I32(400i32 >> 20i32))),
            I32
        );
        eval_and_compare!(
            r##"428i32 & 428i32"##,
            Primitive::Number(Cell::new(Number::I32(428i32 & 428i32))),
            I32
        );
        eval_and_compare!(
            r##"400i32 | 420i32"##,
            Primitive::Number(Cell::new(Number::I32(400i32 | 420i32))),
            I32
        );
        eval_and_compare!(
            r##"400i32 ^ 420i32"##,
            Primitive::Number(Cell::new(Number::I32(400i32 ^ 420i32))),
            I32
        );
        eval_and_compare_bool!(r##"20i32 > 2i32"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i32 > 20i32"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i32 >= 2i32"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i32 >= 20i32"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"2i32 <= 20i32"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i32 <= 2i32"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i32 > 2i32"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i32 > 20i32"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i32 == 20i32"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i32 == 2i32"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i32 != 2i32"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i32 != 20i32"##, Primitive::Bool(false));
    }
    #[test]
    fn valid_operation_i16() {
        eval_and_compare!(
            r##"400i16 + 20i16"##,
            Primitive::Number(Cell::new(Number::I16(420))),
            I16
        );
        eval_and_compare!(
            r##"400i16 - 800i16"##,
            Primitive::Number(Cell::new(Number::I16(400 - 800))),
            I16
        );
        eval_and_compare!(
            r##"400i16 * 5i16"##,
            Primitive::Number(Cell::new(Number::I16(400 * 5))),
            I16
        );
        eval_and_compare!(
            r##"400i16 / 2i16"##,
            Primitive::Number(Cell::new(Number::I16(400 / 2))),
            I16
        );
        eval_and_compare!(
            r##"400i16 % 2i16"##,
            Primitive::Number(Cell::new(Number::I16(400 % 2))),
            I16
        );
        eval_and_compare!(
            r##"-20i16"##,
            Primitive::Number(Cell::new(Number::I16(-20))),
            I16
        );
        eval_and_compare!(
            r##"400i16 << 2i16"##,
            Primitive::Number(Cell::new(Number::I16(400i16 << 2i16))),
            I16
        );
        eval_and_compare!(
            r##"400i16 >> 2i16"##,
            Primitive::Number(Cell::new(Number::I16(400i16 >> 2i16))),
            I16
        );
        eval_and_compare!(
            r##"428i16 & 428i16"##,
            Primitive::Number(Cell::new(Number::I16(428i16 & 428i16))),
            I16
        );
        eval_and_compare!(
            r##"400i16 | 420i16"##,
            Primitive::Number(Cell::new(Number::I16(400i16 | 420i16))),
            I16
        );
        eval_and_compare!(
            r##"400i16 ^ 420i16"##,
            Primitive::Number(Cell::new(Number::I16(400i16 ^ 420i16))),
            I16
        );
        eval_and_compare_bool!(r##"20i16 > 2i16"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i16 > 20i16"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i16 >= 2i16"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i16 >= 20i16"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"2i16 <= 20i16"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i16 <= 2i16"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i16 > 2i16"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i16 > 20i16"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i16 == 20i16"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i16 == 2i16"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i16 != 2i16"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i16 != 20i16"##, Primitive::Bool(false));
    }
    #[test]
    fn valid_operation_i8() {
        eval_and_compare!(
            r##"100i8 + 20i8"##,
            Primitive::Number(Cell::new(Number::I8(120))),
            I8
        );
        eval_and_compare!(
            r##"20i8 - 10i8"##,
            Primitive::Number(Cell::new(Number::I8(20 - 10))),
            I8
        );
        eval_and_compare!(
            r##"20i8 * 5i8"##,
            Primitive::Number(Cell::new(Number::I8(20 * 5))),
            I8
        );
        eval_and_compare!(
            r##"20i8 / 2i8"##,
            Primitive::Number(Cell::new(Number::I8(20 / 2))),
            I8
        );
        eval_and_compare!(
            r##"20i8 % 2i8"##,
            Primitive::Number(Cell::new(Number::I8(20 % 2))),
            I8
        );
        eval_and_compare!(
            r##"-20i8"##,
            Primitive::Number(Cell::new(Number::I8(-20))),
            I8
        );
        eval_and_compare!(
            r##"40i8 << 2i8"##,
            Primitive::Number(Cell::new(Number::I8(40i8 << 2i8))),
            I8
        );
        eval_and_compare!(
            r##"40i8 >> 2i8"##,
            Primitive::Number(Cell::new(Number::I8(40i8 >> 2i8))),
            I8
        );
        eval_and_compare!(
            r##"48i8 & 48i8"##,
            Primitive::Number(Cell::new(Number::I8(48i8 & 48i8))),
            I8
        );
        eval_and_compare!(
            r##"40i8 | 42i8"##,
            Primitive::Number(Cell::new(Number::I8(40i8 | 42i8))),
            I8
        );
        eval_and_compare!(
            r##"40i8 ^ 42i8"##,
            Primitive::Number(Cell::new(Number::I8(40i8 ^ 42i8))),
            I8
        );
        eval_and_compare_bool!(r##"20i8 > 2i8"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i8 > 20i8"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i8 >= 2i8"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i8 >= 20i8"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"2i8 <= 20i8"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i8 <= 2i8"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i8 > 2i8"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2i8 > 20i8"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i8 == 20i8"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i8 == 2i8"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20i8 != 2i8"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20i8 != 20i8"##, Primitive::Bool(false));
    }
    #[test]
    fn valid_operation_f64() {
        eval_and_compare!(
            r##"10.5 + 20.2"##,
            Primitive::Number(Cell::new(Number::F64(10.5 + 20.2))),
            F64
        );
        eval_and_compare!(
            r##"10.5 - 20.2"##,
            Primitive::Number(Cell::new(Number::F64(10.5 - 20.2))),
            F64
        );
        eval_and_compare!(
            r##"10.5 * 20.2"##,
            Primitive::Number(Cell::new(Number::F64(10.5 * 20.2))),
            F64
        );
        eval_and_compare!(
            r##"10.5 / 20.2"##,
            Primitive::Number(Cell::new(Number::F64(10.5 / 20.2))),
            F64
        );
        eval_and_compare!(
            r##"-20.0"##,
            Primitive::Number(Cell::new(Number::F64(-20.0))),
            F64
        );
        eval_and_compare_bool!(r##"20f64 > 2f64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2f64 > 20f64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20f64 >= 2f64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2f64 >= 20f64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"2f64 <= 20f64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20f64 <= 2f64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20f64 > 2f64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"2f64 > 20f64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20f64 == 20f64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20f64 == 2f64"##, Primitive::Bool(false));
        eval_and_compare_bool!(r##"20f64 != 2f64"##, Primitive::Bool(true));
        eval_and_compare_bool!(r##"20f64 != 20f64"##, Primitive::Bool(false));
    }
    #[test]
    fn valid_addition_string() {
        let expr = Expression::parse(
            r##"
           "Hello " + "world"
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = expr
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        expr.gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);
        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let result: StringData =
            <StringType as DeserializeFrom<Scope>>::deserialize_from(&StringType(), &data)
                .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "Hello World")
    }
}
