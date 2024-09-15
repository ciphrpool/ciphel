use nom::{
    branch::alt,
    combinator::{cut, map},
    sequence::delimited,
};

use crate::semantic::scope::scope::ScopeManager;
use crate::{
    ast::{
        expressions::operation::LogicalOr,
        utils::{
            io::{PResult, Span},
            lexem,
            strings::{eater, wst},
        },
    },
    semantic::{
        AccessLevel, EType, Metadata, Resolve, ResolveFromStruct, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        allocator::MemoryAddress,
        casm::{
            locate::{Locate, LocateOffsetFromStackPointer},
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use self::operation::{
    operation_parse::TryParseOperation, Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast,
    Comparaison, Equation, FieldAccess, FnCall, ListAccess, LogicalAnd, Product, Range, Shift,
    Substraction, TupleAccess, UnaryOperation,
};

use super::{utils::error::squash, TryParse};

pub mod data;
pub mod flows;
pub mod locate;
pub mod operation;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Product(operation::Product),
    Addition(operation::Addition),
    Substraction(operation::Substraction),
    Shift(operation::Shift),
    BitwiseAnd(operation::BitwiseAnd),
    BitwiseXOR(operation::BitwiseXOR),
    BitwiseOR(operation::BitwiseOR),
    Cast(operation::Cast),
    Comparaison(operation::Comparaison),
    Equation(operation::Equation),
    LogicalAnd(operation::LogicalAnd),
    LogicalOr(operation::LogicalOr),
    Range(operation::Range),
    FieldAccess(operation::FieldAccess),
    ListAccess(operation::ListAccess),
    TupleAccess(operation::TupleAccess),
    FnCall(operation::FnCall),
    Atomic(Atomic),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atomic {
    Data(data::Data),
    UnaryOperation(operation::UnaryOperation),
    Paren(Box<Expression>),
    ExprFlow(flows::ExprFlow),
}

impl TryParse for Atomic {
    /*
     * @desc Parse an atomic expression
     *
     * @grammar
     * Atomic :=
     * | Data
     * | Negation
     * | \(  Expr \)
     * | ExprStatement
     * | Error
     */
    fn parse(input: Span) -> PResult<Self> {
        squash(
            alt((
                map(
                    delimited(wst(lexem::PAR_O), Expression::parse, wst(lexem::PAR_C)),
                    |value| Atomic::Paren(Box::new(value)),
                ),
                map(flows::ExprFlow::parse, Atomic::ExprFlow),
                map(data::Data::parse, Atomic::Data),
                map(operation::UnaryOperation::parse, Atomic::UnaryOperation),
            )),
            "Expected a valid expression",
        )(input)
    }
}

impl Resolve for Atomic {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Atomic::Data(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Atomic::UnaryOperation(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Atomic::Paren(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Atomic::ExprFlow(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
        }
    }
}

impl ResolveFromStruct for Atomic {
    fn resolve_from_struct<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        match self {
            Atomic::Data(value) => {
                value.resolve_from_struct::<G>(scope_manager, scope_id, struct_id)
            }
            Atomic::Paren(value) => {
                value.resolve_from_struct::<G>(scope_manager, scope_id, struct_id)
            }
            _ => Err(SemanticError::UnknownField),
        }
    }
}

impl TypeOf for Atomic {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Atomic::Data(value) => value.type_of(&scope_manager, scope_id),
            Atomic::UnaryOperation(value) => value.type_of(&scope_manager, scope_id),
            Atomic::Paren(value) => value.type_of(&scope_manager, scope_id),
            Atomic::ExprFlow(value) => value.type_of(&scope_manager, scope_id),
        }
    }
}

impl TryParse for Expression {
    /*
     * @desc Parse an expression
     *
     * @grammar
     * Expr := AndOp Or AndOp | AndOp
     * AndOp := CompOp And CompOp | CompOp
     * CompOp := BOr (< |<= | >= | >| == | != | in ) BOr | BOr
     * BOr := XOr \| XOr  | XOr
     * XOr := BAnd ^ BAnd | BAnd
     * BAnd := Shift & Shift | Shift
     * Shift := LowM (<<|>>) LowM | LowM
     * LowM := HighM + HighM | HighM
     * HighM := Atom (* | / | % ) Atom | Atom
     */
    fn parse(input: Span) -> PResult<Self> {
        eater::ws::<_, Expression>(Range::parse)(input)
    }
}

impl Resolve for Expression {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Expression::Product(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Expression::Addition(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Expression::Substraction(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Expression::Shift(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Expression::BitwiseAnd(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Expression::BitwiseXOR(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Expression::BitwiseOR(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Expression::Comparaison(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Expression::LogicalAnd(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Expression::Equation(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Expression::LogicalOr(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Expression::Atomic(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, extra)
            }
            Expression::Cast(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Expression::Range(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, &mut ())
            }
            Expression::FieldAccess(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, extra)
            }
            Expression::ListAccess(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, extra)
            }
            Expression::TupleAccess(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, extra)
            }
            Expression::FnCall(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, extra)
            }
        }
    }
}

impl ResolveFromStruct for Expression {
    fn resolve_from_struct<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        match self {
            Expression::FieldAccess(value) => {
                value.resolve_from_struct::<G>(scope_manager, scope_id, struct_id)
            }
            Expression::ListAccess(value) => {
                value.resolve_from_struct::<G>(scope_manager, scope_id, struct_id)
            }
            Expression::TupleAccess(value) => {
                value.resolve_from_struct::<G>(scope_manager, scope_id, struct_id)
            }
            Expression::FnCall(value) => {
                value.resolve_from_struct::<G>(scope_manager, scope_id, struct_id)
            }
            Expression::Atomic(value) => {
                value.resolve_from_struct::<G>(scope_manager, scope_id, struct_id)
            }
            _ => Err(SemanticError::UnknownField),
        }
    }
}

impl TypeOf for Expression {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Expression::Product(value) => value.type_of(&scope_manager, scope_id),
            Expression::Addition(value) => value.type_of(&scope_manager, scope_id),
            Expression::Substraction(value) => value.type_of(&scope_manager, scope_id),
            Expression::Shift(value) => value.type_of(&scope_manager, scope_id),
            Expression::BitwiseAnd(value) => value.type_of(&scope_manager, scope_id),
            Expression::BitwiseXOR(value) => value.type_of(&scope_manager, scope_id),
            Expression::BitwiseOR(value) => value.type_of(&scope_manager, scope_id),
            Expression::Comparaison(value) => value.type_of(&scope_manager, scope_id),
            Expression::LogicalAnd(value) => value.type_of(&scope_manager, scope_id),
            Expression::LogicalOr(value) => value.type_of(&scope_manager, scope_id),
            Expression::Equation(value) => value.type_of(&scope_manager, scope_id),
            Expression::Atomic(value) => value.type_of(&scope_manager, scope_id),
            Expression::Cast(value) => value.type_of(&scope_manager, scope_id),
            Expression::Range(value) => value.type_of(&scope_manager, scope_id),
            Expression::FieldAccess(value) => value.type_of(&scope_manager, scope_id),
            Expression::ListAccess(value) => value.type_of(&scope_manager, scope_id),
            Expression::TupleAccess(value) => value.type_of(&scope_manager, scope_id),
            Expression::FnCall(value) => value.type_of(&scope_manager, scope_id),
        }
    }
}

impl GenerateCode for Expression {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Expression::Product(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::Addition(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::Substraction(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::Shift(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::BitwiseAnd(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::BitwiseXOR(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::BitwiseOR(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::Cast(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::Comparaison(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::Equation(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::LogicalAnd(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::LogicalOr(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::Atomic(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::Range(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::FieldAccess(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::ListAccess(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::TupleAccess(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Expression::FnCall(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
        }
    }
}

// impl Locatable for Expression {
//     fn locate(
//         &self,
//         scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//         instructions: &mut CasmProgram,
//         context: &crate::vm::vm::CodeGenerationContext,
//     ) -> Result<(), CodeGenerationError> {
//         match self {
//             Expression::Atomic(value) => {
//                 value.locate(scope_manager, scope_id, instructions, context)
//             }
//             Expression::FieldAccess(value) => {
//                 value.locate(scope_manager, scope_id, instructions, context)
//             }
//             Expression::ListAccess(value) => {
//                 value.locate(scope_manager, scope_id, instructions, context)
//             }
//             Expression::TupleAccess(value) => {
//                 value.locate(scope_manager, scope_id, instructions, context)
//             }
//             Expression::FnCall(value) => {
//                 value.locate(scope_manager, scope_id, instructions, context)
//             }
//             _ => {
//                 let _ = self.gencode(scope_manager, scope_id, instructions, context)?;
//                 let Some(value_type) = self.signature() else {
//                     return Err(CodeGenerationError::UnresolvedError);
//                 };
//                 instructions.push(Casm::OffsetSP(LocateOffsetFromStackPointer {
//                     offset: value_type.size_of(),
//                 }));
//                 Ok(())
//             }
//         }
//     }

//     fn is_assignable(&self) -> bool {
//         match self {
//             Expression::Product(_) => false,
//             Expression::Addition(_) => false,
//             Expression::Substraction(_) => false,
//             Expression::Shift(_) => false,
//             Expression::BitwiseAnd(_) => false,
//             Expression::BitwiseXOR(_) => false,
//             Expression::BitwiseOR(_) => false,
//             Expression::Cast(_) => false,
//             Expression::Comparaison(_) => false,
//             Expression::Equation(_) => false,
//             Expression::LogicalAnd(_) => false,
//             Expression::LogicalOr(_) => false,
//             Expression::Range(_) => false,
//             Expression::Atomic(value) => value.is_assignable(),
//             Expression::FieldAccess(value) => value.is_assignable(),
//             Expression::ListAccess(value) => value.is_assignable(),
//             Expression::TupleAccess(value) => value.is_assignable(),
//             Expression::FnCall(_) => false,
//         }
//     }
//     fn most_left_id(&self) -> Option<super::utils::strings::ID> {
//         match self {
//             Expression::FieldAccess(value) => value.most_left_id(),
//             Expression::ListAccess(value) => value.most_left_id(),
//             Expression::TupleAccess(value) => value.most_left_id(),
//             Expression::Atomic(value) => value.most_left_id(),
//             _ => None,
//         }
//     }
// }

impl GenerateCode for Atomic {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Atomic::Data(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Atomic::UnaryOperation(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Atomic::Paren(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Atomic::ExprFlow(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
        }
    }
}

// impl Locatable for Atomic {
//     fn locate(
//         &self,
//         scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//         instructions: &mut CasmProgram,
//         context: &crate::vm::vm::CodeGenerationContext,
//     ) -> Result<(), CodeGenerationError> {
//         match self {
//             Atomic::Data(value) => value.locate(scope_manager, scope_id, instructions, context),
//             Atomic::Paren(value) => value.locate(scope_manager, scope_id, instructions, context),
//             _ => {
//                 let _ = self.gencode(scope_manager, scope_id, instructions, context)?;
//                 let Some(value_type) = self.signature() else {
//                     return Err(CodeGenerationError::UnresolvedError);
//                 };
//                 instructions.push(Casm::OffsetSP(LocateOffsetFromStackPointer {
//                     offset: value_type.size_of(),
//                 }));
//                 Ok(())
//             }
//         }
//     }

//     fn is_assignable(&self) -> bool {
//         match self {
//             Atomic::Data(value) => value.is_assignable(),
//             Atomic::UnaryOperation(_) => false,
//             Atomic::Paren(value) => value.is_assignable(),
//             Atomic::ExprFlow(_) => false,
//         }
//     }

//     fn most_left_id(&self) -> Option<super::utils::strings::ID> {
//         match self {
//             Atomic::Data(value) => value.most_left_id(),
//             Atomic::UnaryOperation(_) => None,
//             Atomic::Paren(value) => value.most_left_id(),
//             Atomic::ExprFlow(value) => None,
//         }
//     }
// }

impl Expression {
    pub fn metadata(&self) -> Option<&Metadata> {
        match self {
            Expression::Product(Product::Div { metadata, .. }) => Some(metadata),
            Expression::Product(Product::Mod { metadata, .. }) => Some(metadata),
            Expression::Product(Product::Mult { metadata, .. }) => Some(metadata),
            Expression::Addition(Addition { metadata, .. }) => Some(metadata),
            Expression::Substraction(Substraction { metadata, .. }) => Some(metadata),
            Expression::Shift(Shift::Left { metadata, .. }) => Some(metadata),
            Expression::Shift(Shift::Right { metadata, .. }) => Some(metadata),
            Expression::BitwiseAnd(BitwiseAnd { metadata, .. }) => Some(metadata),
            Expression::BitwiseXOR(BitwiseXOR { metadata, .. }) => Some(metadata),
            Expression::BitwiseOR(BitwiseOR { metadata, .. }) => Some(metadata),
            Expression::Cast(Cast { metadata, .. }) => Some(metadata),
            Expression::Comparaison(Comparaison::Greater { metadata, .. }) => Some(metadata),
            Expression::Comparaison(Comparaison::GreaterEqual { metadata, .. }) => Some(metadata),
            Expression::Comparaison(Comparaison::Less { metadata, .. }) => Some(metadata),
            Expression::Comparaison(Comparaison::LessEqual { metadata, .. }) => Some(metadata),
            Expression::Equation(Equation::Equal { metadata, .. }) => Some(metadata),
            Expression::Equation(Equation::NotEqual { metadata, .. }) => Some(metadata),
            Expression::LogicalAnd(LogicalAnd { metadata, .. }) => Some(metadata),
            Expression::LogicalOr(LogicalOr { metadata, .. }) => Some(metadata),
            Expression::Range(Range { metadata, .. }) => Some(metadata),
            Expression::Atomic(value) => value.metadata(),
            Expression::FieldAccess(FieldAccess { metadata, .. }) => Some(metadata),
            Expression::ListAccess(ListAccess { metadata, .. }) => Some(metadata),
            Expression::TupleAccess(TupleAccess { metadata, .. }) => Some(metadata),
            Expression::FnCall(FnCall { metadata, .. }) => Some(metadata),
        }
    }

    pub fn metadata_mut(&mut self) -> Option<&mut Metadata> {
        match self {
            Expression::Product(Product::Div { metadata, .. }) => Some(metadata),
            Expression::Product(Product::Mod { metadata, .. }) => Some(metadata),
            Expression::Product(Product::Mult { metadata, .. }) => Some(metadata),
            Expression::Addition(Addition { metadata, .. }) => Some(metadata),
            Expression::Substraction(Substraction { metadata, .. }) => Some(metadata),
            Expression::Shift(Shift::Left { metadata, .. }) => Some(metadata),
            Expression::Shift(Shift::Right { metadata, .. }) => Some(metadata),
            Expression::BitwiseAnd(BitwiseAnd { metadata, .. }) => Some(metadata),
            Expression::BitwiseXOR(BitwiseXOR { metadata, .. }) => Some(metadata),
            Expression::BitwiseOR(BitwiseOR { metadata, .. }) => Some(metadata),
            Expression::Cast(Cast { metadata, .. }) => Some(metadata),
            Expression::Comparaison(Comparaison::Greater { metadata, .. }) => Some(metadata),
            Expression::Comparaison(Comparaison::GreaterEqual { metadata, .. }) => Some(metadata),
            Expression::Comparaison(Comparaison::Less { metadata, .. }) => Some(metadata),
            Expression::Comparaison(Comparaison::LessEqual { metadata, .. }) => Some(metadata),
            Expression::Equation(Equation::Equal { metadata, .. }) => Some(metadata),
            Expression::Equation(Equation::NotEqual { metadata, .. }) => Some(metadata),
            Expression::LogicalAnd(LogicalAnd { metadata, .. }) => Some(metadata),
            Expression::LogicalOr(LogicalOr { metadata, .. }) => Some(metadata),
            Expression::Range(Range { metadata, .. }) => Some(metadata),
            Expression::Atomic(value) => value.metadata_mut(),
            Expression::FieldAccess(FieldAccess { metadata, .. }) => Some(metadata),
            Expression::ListAccess(ListAccess { metadata, .. }) => Some(metadata),
            Expression::TupleAccess(TupleAccess { metadata, .. }) => Some(metadata),
            Expression::FnCall(FnCall { metadata, .. }) => Some(metadata),
        }
    }

    pub fn signature(&self) -> Option<EType> {
        match self {
            Expression::Product(Product::Div { metadata, .. }) => metadata.signature(),
            Expression::Product(Product::Mod { metadata, .. }) => metadata.signature(),
            Expression::Product(Product::Mult { metadata, .. }) => metadata.signature(),
            Expression::Addition(Addition { metadata, .. }) => metadata.signature(),
            Expression::Substraction(Substraction { metadata, .. }) => metadata.signature(),
            Expression::Shift(Shift::Left { metadata, .. }) => metadata.signature(),
            Expression::Shift(Shift::Right { metadata, .. }) => metadata.signature(),
            Expression::BitwiseAnd(BitwiseAnd { metadata, .. }) => metadata.signature(),
            Expression::BitwiseXOR(BitwiseXOR { metadata, .. }) => metadata.signature(),
            Expression::BitwiseOR(BitwiseOR { metadata, .. }) => metadata.signature(),
            Expression::Cast(Cast { metadata, .. }) => metadata.signature(),
            Expression::Comparaison(Comparaison::Greater { metadata, .. }) => metadata.signature(),
            Expression::Comparaison(Comparaison::GreaterEqual { metadata, .. }) => {
                metadata.signature()
            }
            Expression::Comparaison(Comparaison::Less { metadata, .. }) => metadata.signature(),
            Expression::Comparaison(Comparaison::LessEqual { metadata, .. }) => {
                metadata.signature()
            }
            Expression::Equation(Equation::Equal { metadata, .. }) => metadata.signature(),
            Expression::Equation(Equation::NotEqual { metadata, .. }) => metadata.signature(),
            Expression::LogicalAnd(LogicalAnd { metadata, .. }) => metadata.signature(),
            Expression::LogicalOr(LogicalOr { metadata, .. }) => metadata.signature(),
            Expression::Range(Range { metadata, .. }) => metadata.signature(),
            Expression::Atomic(value) => value.signature(),
            Expression::FieldAccess(FieldAccess { metadata, .. }) => metadata.signature(),
            Expression::ListAccess(ListAccess { metadata, .. }) => metadata.signature(),
            Expression::TupleAccess(TupleAccess { metadata, .. }) => metadata.signature(),
            Expression::FnCall(FnCall { metadata, .. }) => metadata.signature(),
        }
    }
}

impl Atomic {
    pub fn metadata(&self) -> Option<&Metadata> {
        match self {
            Atomic::Data(value) => value.metadata(),
            Atomic::UnaryOperation(UnaryOperation::Minus { value: _, metadata }) => Some(metadata),
            Atomic::UnaryOperation(UnaryOperation::Not { value: _, metadata }) => Some(metadata),
            Atomic::Paren(value) => value.metadata(),
            Atomic::ExprFlow(value) => value.metadata(),
        }
    }
    pub fn metadata_mut(&mut self) -> Option<&mut Metadata> {
        match self {
            Atomic::Data(value) => value.metadata_mut(),
            Atomic::UnaryOperation(UnaryOperation::Minus { value: _, metadata }) => Some(metadata),
            Atomic::UnaryOperation(UnaryOperation::Not { value: _, metadata }) => Some(metadata),
            Atomic::Paren(value) => value.metadata_mut(),
            Atomic::ExprFlow(value) => value.metadata_mut(),
        }
    }

    pub fn signature(&self) -> Option<EType> {
        match self {
            Atomic::Data(value) => value.signature(),
            Atomic::UnaryOperation(UnaryOperation::Minus { value: _, metadata }) => {
                metadata.signature()
            }
            Atomic::UnaryOperation(UnaryOperation::Not { value: _, metadata }) => {
                metadata.signature()
            }
            Atomic::Paren(value) => value.signature(),
            Atomic::ExprFlow(value) => value.signature(),
        }
    }
}
