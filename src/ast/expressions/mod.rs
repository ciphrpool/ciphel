use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use nom::{branch::alt, combinator::map, sequence::delimited};

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
        scope::{static_types::StaticType, user_type_impl::UserType, ScopeApi},
        EType, Either, Metadata, MutRc, Resolve, SemanticError, TypeOf,
    },
    vm::{
        casm::{Casm, CasmProgram},
        vm::{CodeGenerationError, GenerateCode},
    },
};

use self::operation::{
    operation_parse::TryParseOperation, Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast,
    Comparaison, Equation, LogicalAnd, Product, Range, Shift, Substraction, UnaryOperation,
};

use super::TryParse;

pub mod data;
pub mod error;
pub mod flows;
pub mod operation;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<InnerScope: ScopeApi> {
    Product(operation::Product<InnerScope>),
    Addition(operation::Addition<InnerScope>),
    Substraction(operation::Substraction<InnerScope>),
    Shift(operation::Shift<InnerScope>),
    BitwiseAnd(operation::BitwiseAnd<InnerScope>),
    BitwiseXOR(operation::BitwiseXOR<InnerScope>),
    BitwiseOR(operation::BitwiseOR<InnerScope>),
    Cast(operation::Cast<InnerScope>),
    Comparaison(operation::Comparaison<InnerScope>),
    Equation(operation::Equation<InnerScope>),
    LogicalAnd(operation::LogicalAnd<InnerScope>),
    LogicalOr(operation::LogicalOr<InnerScope>),
    Range(operation::Range<InnerScope>),
    Atomic(Atomic<InnerScope>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atomic<InnerScope: ScopeApi> {
    Data(data::Data<InnerScope>),
    UnaryOperation(operation::UnaryOperation<InnerScope>),
    Paren(Box<Expression<InnerScope>>),
    ExprFlow(flows::ExprFlow<InnerScope>),
    Error(error::Error),
}

impl<Scope: ScopeApi> TryParse for Atomic<Scope> {
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
        alt((
            map(
                delimited(wst(lexem::PAR_O), Expression::parse, wst(lexem::PAR_C)),
                |value| Atomic::Paren(Box::new(value)),
            ),
            map(error::Error::parse, |value| Atomic::Error(value)),
            map(flows::ExprFlow::parse, |value| Atomic::ExprFlow(value)),
            map(data::Data::parse, |value| Atomic::Data(value)),
            map(operation::UnaryOperation::parse, |value| {
                Atomic::UnaryOperation(value)
            }),
        ))(input)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Atomic<Scope> {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Atomic::Data(value) => value.resolve(scope, context, extra),
            Atomic::UnaryOperation(value) => value.resolve(scope, context, extra),
            Atomic::Paren(value) => value.resolve(scope, context, extra),
            Atomic::ExprFlow(value) => value.resolve(scope, context, extra),
            Atomic::Error(value) => value.resolve(scope, &(), &()),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Atomic<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Atomic::Data(value) => value.type_of(&scope),
            Atomic::UnaryOperation(value) => value.type_of(&scope),
            Atomic::Paren(value) => value.type_of(&scope),
            Atomic::ExprFlow(value) => value.type_of(&scope),
            Atomic::Error(value) => value.type_of(&scope),
        }
    }
}

impl<Scope: ScopeApi> TryParse for Expression<Scope> {
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
        eater::ws::<_, Expression<Scope>>(Range::<Scope>::parse)(input)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Expression<Scope> {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Expression::Product(value) => value.resolve(scope, context, extra),
            Expression::Addition(value) => value.resolve(scope, context, extra),
            Expression::Substraction(value) => value.resolve(scope, context, extra),
            Expression::Shift(value) => value.resolve(scope, context, extra),
            Expression::BitwiseAnd(value) => value.resolve(scope, context, extra),
            Expression::BitwiseXOR(value) => value.resolve(scope, context, extra),
            Expression::BitwiseOR(value) => value.resolve(scope, context, extra),
            Expression::Comparaison(value) => value.resolve(scope, context, extra),
            Expression::LogicalAnd(value) => value.resolve(scope, context, extra),
            Expression::Equation(value) => value.resolve(scope, context, extra),
            Expression::LogicalOr(value) => value.resolve(scope, context, extra),
            Expression::Atomic(value) => value.resolve(scope, context, extra),
            Expression::Cast(value) => value.resolve(scope, context, extra),
            Expression::Range(value) => value.resolve(scope, context, extra),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Expression<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Expression::Product(value) => value.type_of(&scope),
            Expression::Addition(value) => value.type_of(&scope),
            Expression::Substraction(value) => value.type_of(&scope),
            Expression::Shift(value) => value.type_of(&scope),
            Expression::BitwiseAnd(value) => value.type_of(&scope),
            Expression::BitwiseXOR(value) => value.type_of(&scope),
            Expression::BitwiseOR(value) => value.type_of(&scope),
            Expression::Comparaison(value) => value.type_of(&scope),
            Expression::LogicalAnd(value) => value.type_of(&scope),
            Expression::LogicalOr(value) => value.type_of(&scope),
            Expression::Equation(value) => value.type_of(&scope),
            Expression::Atomic(value) => value.type_of(&scope),
            Expression::Cast(value) => value.type_of(&scope),
            Expression::Range(value) => value.type_of(&scope),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Expression<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Expression::Product(value) => value.gencode(scope, instructions),
            Expression::Addition(value) => value.gencode(scope, instructions),
            Expression::Substraction(value) => value.gencode(scope, instructions),
            Expression::Shift(value) => value.gencode(scope, instructions),
            Expression::BitwiseAnd(value) => value.gencode(scope, instructions),
            Expression::BitwiseXOR(value) => value.gencode(scope, instructions),
            Expression::BitwiseOR(value) => value.gencode(scope, instructions),
            Expression::Cast(value) => value.gencode(scope, instructions),
            Expression::Comparaison(value) => value.gencode(scope, instructions),
            Expression::Equation(value) => value.gencode(scope, instructions),
            Expression::LogicalAnd(value) => value.gencode(scope, instructions),
            Expression::LogicalOr(value) => value.gencode(scope, instructions),
            Expression::Atomic(value) => value.gencode(scope, instructions),
            Expression::Range(value) => value.gencode(scope, instructions),
        }
    }
}
impl<Scope: ScopeApi> GenerateCode<Scope> for Atomic<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Atomic::Data(value) => value.gencode(scope, instructions),
            Atomic::UnaryOperation(value) => value.gencode(scope, instructions),
            Atomic::Paren(value) => value.gencode(scope, instructions),
            Atomic::ExprFlow(value) => value.gencode(scope, instructions),
            Atomic::Error(_) => todo!(),
        }
    }
}

impl<Scope: ScopeApi> Expression<Scope> {
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
            Expression::Atomic(value) => value.metadata(),
            Expression::Range(value) => value.metadata(),
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
            Expression::Atomic(value) => value.signature(),
            Expression::Range(value) => value.signature(),
        }
    }
}

impl<Scope: ScopeApi> Atomic<Scope> {
    pub fn metadata(&self) -> Option<&Metadata> {
        match self {
            Atomic::Data(value) => value.metadata(),
            Atomic::UnaryOperation(UnaryOperation::Minus { value, metadata }) => Some(metadata),
            Atomic::UnaryOperation(UnaryOperation::Not { value, metadata }) => Some(metadata),
            Atomic::Paren(value) => value.metadata(),
            Atomic::ExprFlow(value) => value.metadata(),
            Atomic::Error(value) => todo!(),
        }
    }

    pub fn signature(&self) -> Option<EType> {
        match self {
            Atomic::Data(value) => value.signature(),
            Atomic::UnaryOperation(UnaryOperation::Minus { value, metadata }) => {
                metadata.signature()
            }
            Atomic::UnaryOperation(UnaryOperation::Not { value, metadata }) => metadata.signature(),
            Atomic::Paren(value) => value.signature(),
            Atomic::ExprFlow(value) => value.signature(),
            Atomic::Error(value) => todo!(),
        }
    }
}
