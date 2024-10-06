use nom::{
    branch::alt,
    combinator::map,
    multi::{many0, many1},
    sequence::{delimited, pair, terminated},
};
use operation::ExprCall;

use crate::{
    ast::{
        expressions::operation::LogicalOr,
        utils::{
            io::{PResult, Span},
            lexem,
            strings::{eater, wst},
        },
    },
    semantic::{EType, Metadata, Resolve, ResolveFromStruct, SemanticError, TypeOf},
};
use crate::{
    semantic::{Desugar, ResolveNumber},
    vm::GenerateCode,
};

use self::operation::{
    operation_parse::TryParseOperation, Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast,
    Comparaison, Equation, FieldAccess, ListAccess, LogicalAnd, Product, Shift, Substraction,
    TupleAccess, UnaryOperation,
};

use super::{
    utils::{error::squash, strings::parse_id},
    TryParse,
};

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
    FieldAccess(operation::FieldAccess),
    ListAccess(operation::ListAccess),
    TupleAccess(operation::TupleAccess),
    ExprCall(operation::ExprCall),
    Atomic(Atomic),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atomic {
    Data(data::Data),
    UnaryOperation(operation::UnaryOperation),
    Paren(Box<Expression>),
    ExprFlow(flows::ExprFlow),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Path {
    Segment(Vec<String>),
    #[default]
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletePath {
    pub path: Path,
    pub name: String,
}

impl TryParse for Path {
    fn parse(input: Span) -> PResult<Self> {
        map(many0(terminated(parse_id, wst(lexem::SEP))), |segments| {
            if segments.len() == 0 {
                Path::Empty
            } else {
                Path::Segment(segments)
            }
        })(input)
    }
}

impl Path {
    fn parse_segment(input: Span) -> PResult<Self> {
        map(many1(terminated(parse_id, wst(lexem::SEP))), |segments| {
            Path::Segment(segments)
        })(input)
    }
}

impl TryParse for CompletePath {
    fn parse(input: Span) -> PResult<Self> {
        map(pair(Path::parse, parse_id), |(path, name)| CompletePath {
            name,
            path,
        })(input)
    }
}

impl CompletePath {
    fn parse_segment(input: Span) -> PResult<Self> {
        map(pair(Path::parse_segment, parse_id), |(path, name)| {
            CompletePath { name, path }
        })(input)
    }
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
    fn resolve<E: crate::vm::external::Engine>(
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
            Atomic::Data(value) => value.resolve::<E>(scope_manager, scope_id, context, extra),
            Atomic::UnaryOperation(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Atomic::Paren(value) => value.resolve::<E>(scope_manager, scope_id, context, extra),
            Atomic::ExprFlow(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
        }
    }
}

impl ResolveFromStruct for Atomic {
    fn resolve_from_struct<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        match self {
            Atomic::Data(value) => {
                value.resolve_from_struct::<E>(scope_manager, scope_id, struct_id)
            }
            Atomic::Paren(value) => {
                value.resolve_from_struct::<E>(scope_manager, scope_id, struct_id)
            }
            _ => Err(SemanticError::UnknownField),
        }
    }
}

impl Desugar<Expression> for Atomic {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        let output = match self {
            Atomic::Data(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Atomic::UnaryOperation(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Atomic::Paren(value) => {
                let _ = value.desugar::<E>(scope_manager, scope_id)?;
                None
            }
            Atomic::ExprFlow(value) => value.desugar::<E>(scope_manager, scope_id)?,
        };

        if let Some(output) = output {
            *self = output;
        }

        Ok(None)
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
        eater::ws::<_, Expression>(LogicalOr::parse)(input)
    }
}

impl Resolve for Expression {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<E: crate::vm::external::Engine>(
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
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Expression::Addition(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Expression::Substraction(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Expression::Shift(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Expression::BitwiseAnd(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Expression::BitwiseXOR(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Expression::BitwiseOR(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Expression::Comparaison(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Expression::LogicalAnd(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Expression::Equation(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Expression::LogicalOr(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Expression::Atomic(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, extra)
            }
            Expression::Cast(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Expression::FieldAccess(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, extra)
            }
            Expression::ListAccess(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, extra)
            }
            Expression::TupleAccess(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, extra)
            }
            Expression::ExprCall(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, extra)
            }
        }
    }
}

impl ResolveNumber for Expression {
    fn is_unresolved_number(&self) -> bool {
        match self {
            Expression::Product(product) => product.is_unresolved_number(),
            Expression::Addition(addition) => addition.is_unresolved_number(),
            Expression::Substraction(substraction) => substraction.is_unresolved_number(),
            Expression::Shift(shift) => shift.is_unresolved_number(),
            Expression::BitwiseAnd(bitwise_and) => bitwise_and.is_unresolved_number(),
            Expression::BitwiseXOR(bitwise_xor) => bitwise_xor.is_unresolved_number(),
            Expression::BitwiseOR(bitwise_or) => bitwise_or.is_unresolved_number(),
            Expression::Cast(cast) => cast.is_unresolved_number(),
            Expression::Comparaison(comparaison) => comparaison.is_unresolved_number(),
            Expression::Equation(equation) => equation.is_unresolved_number(),
            Expression::LogicalAnd(logical_and) => logical_and.is_unresolved_number(),
            Expression::LogicalOr(logical_or) => logical_or.is_unresolved_number(),
            Expression::FieldAccess(_) => false,
            Expression::ListAccess(_) => false,
            Expression::TupleAccess(_) => false,
            Expression::ExprCall(_) => false,
            Expression::Atomic(atomic) => atomic.is_unresolved_number(),
        }
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        match self {
            Expression::Product(product) => product.resolve_number(to),
            Expression::Addition(addition) => addition.resolve_number(to),
            Expression::Substraction(substraction) => substraction.resolve_number(to),
            Expression::Shift(shift) => shift.resolve_number(to),
            Expression::BitwiseAnd(bitwise_and) => bitwise_and.resolve_number(to),
            Expression::BitwiseXOR(bitwise_xor) => bitwise_xor.resolve_number(to),
            Expression::BitwiseOR(bitwise_or) => bitwise_or.resolve_number(to),
            Expression::Cast(cast) => cast.resolve_number(to),
            Expression::Comparaison(comparaison) => comparaison.resolve_number(to),
            Expression::Equation(equation) => equation.resolve_number(to),
            Expression::LogicalAnd(logical_and) => logical_and.resolve_number(to),
            Expression::LogicalOr(logical_or) => logical_or.resolve_number(to),
            Expression::FieldAccess(_) => Ok(()),
            Expression::ListAccess(_) => Ok(()),
            Expression::TupleAccess(_) => Ok(()),
            Expression::ExprCall(_) => Ok(()),
            Expression::Atomic(atomic) => atomic.resolve_number(to),
        }
    }
}

impl ResolveNumber for Atomic {
    fn is_unresolved_number(&self) -> bool {
        match self {
            Atomic::Data(data) => data.is_unresolved_number(),
            Atomic::UnaryOperation(unary_operation) => unary_operation.is_unresolved_number(),
            Atomic::Paren(expression) => expression.is_unresolved_number(),
            Atomic::ExprFlow(expr_flow) => expr_flow.is_unresolved_number(),
        }
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        match self {
            Atomic::Data(data) => data.resolve_number(to),
            Atomic::UnaryOperation(unary_operation) => unary_operation.resolve_number(to),
            Atomic::Paren(expression) => expression.resolve_number(to),
            Atomic::ExprFlow(expr_flow) => expr_flow.resolve_number(to),
        }
    }
}

impl ResolveFromStruct for Expression {
    fn resolve_from_struct<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        match self {
            Expression::FieldAccess(value) => {
                value.resolve_from_struct::<E>(scope_manager, scope_id, struct_id)
            }
            Expression::ListAccess(value) => {
                value.resolve_from_struct::<E>(scope_manager, scope_id, struct_id)
            }
            Expression::TupleAccess(value) => {
                value.resolve_from_struct::<E>(scope_manager, scope_id, struct_id)
            }
            Expression::Atomic(value) => {
                value.resolve_from_struct::<E>(scope_manager, scope_id, struct_id)
            }
            _ => Err(SemanticError::UnknownField),
        }
    }
}

impl Desugar<Expression> for Expression {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        let output = match self {
            Expression::Product(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::Addition(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::Substraction(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::Shift(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::BitwiseAnd(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::BitwiseXOR(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::BitwiseOR(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::Cast(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::Comparaison(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::Equation(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::LogicalAnd(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::LogicalOr(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::FieldAccess(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::ListAccess(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::TupleAccess(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::ExprCall(value) => value.desugar::<E>(scope_manager, scope_id)?,
            Expression::Atomic(value) => value.desugar::<E>(scope_manager, scope_id)?,
        };

        if let Some(output) = output {
            *self = output;
        }

        Ok(None)
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
            Expression::FieldAccess(value) => value.type_of(&scope_manager, scope_id),
            Expression::ListAccess(value) => value.type_of(&scope_manager, scope_id),
            Expression::TupleAccess(value) => value.type_of(&scope_manager, scope_id),
            Expression::ExprCall(value) => value.type_of(&scope_manager, scope_id),
        }
    }
}

impl GenerateCode for Expression {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            Expression::Product(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::Addition(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::Substraction(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::Shift(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::BitwiseAnd(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::BitwiseXOR(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::BitwiseOR(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::Cast(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::Comparaison(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::Equation(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::LogicalAnd(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::LogicalOr(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::Atomic(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::FieldAccess(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::ListAccess(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::TupleAccess(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Expression::ExprCall(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
        }
    }
}

impl GenerateCode for Atomic {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            Atomic::Data(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Atomic::UnaryOperation(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Atomic::Paren(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Atomic::ExprFlow(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
        }
    }
}

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
            Expression::Atomic(value) => value.metadata(),
            Expression::FieldAccess(FieldAccess { metadata, .. }) => Some(metadata),
            Expression::ListAccess(ListAccess { metadata, .. }) => Some(metadata),
            Expression::TupleAccess(TupleAccess { metadata, .. }) => Some(metadata),
            Expression::ExprCall(ExprCall { metadata, .. }) => Some(metadata),
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
            Expression::Atomic(value) => value.metadata_mut(),
            Expression::FieldAccess(FieldAccess { metadata, .. }) => Some(metadata),
            Expression::ListAccess(ListAccess { metadata, .. }) => Some(metadata),
            Expression::TupleAccess(TupleAccess { metadata, .. }) => Some(metadata),
            Expression::ExprCall(ExprCall { metadata, .. }) => Some(metadata),
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
            Expression::FieldAccess(FieldAccess { metadata, .. }) => metadata.signature(),
            Expression::ListAccess(ListAccess { metadata, .. }) => metadata.signature(),
            Expression::TupleAccess(TupleAccess { metadata, .. }) => metadata.signature(),
            Expression::ExprCall(ExprCall { metadata, .. }) => metadata.signature(),
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
