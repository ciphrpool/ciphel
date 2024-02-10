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
        EType, Either, MutRc, Resolve, SemanticError, TypeOf,
    },
    vm::{
        casm::{Casm, CasmProgram},
        vm::{CodeGenerationError, GenerateCode},
    },
};

use self::operation::operation_parse::TryParseOperation;

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
    Inclusion(operation::Inclusion<InnerScope>),
    LogicalAnd(operation::LogicalAnd<InnerScope>),
    LogicalOr(operation::LogicalOr<InnerScope>),
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
        eater::ws::<_, Expression<Scope>>(LogicalOr::<Scope>::parse)(input)
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
            Expression::Inclusion(value) => value.resolve(scope, context, extra),
            Expression::LogicalOr(value) => value.resolve(scope, context, extra),
            Expression::Atomic(value) => value.resolve(scope, context, extra),
            Expression::Cast(value) => value.resolve(scope, context, extra),
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
            Expression::Inclusion(value) => value.type_of(&scope),
            Expression::Atomic(value) => value.type_of(&scope),
            Expression::Cast(value) => value.type_of(&scope),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Expression<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Expression::Product(_) => todo!(),
            Expression::Addition(_) => todo!(),
            Expression::Substraction(_) => todo!(),
            Expression::Shift(_) => todo!(),
            Expression::BitwiseAnd(_) => todo!(),
            Expression::BitwiseXOR(_) => todo!(),
            Expression::BitwiseOR(_) => todo!(),
            Expression::Cast(_) => todo!(),
            Expression::Comparaison(_) => todo!(),
            Expression::Equation(_) => todo!(),
            Expression::Inclusion(_) => todo!(),
            Expression::LogicalAnd(_) => todo!(),
            Expression::LogicalOr(_) => todo!(),
            Expression::Atomic(value) => value.gencode(scope, instructions),
        }
    }
}
impl<Scope: ScopeApi> GenerateCode<Scope> for Atomic<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Atomic::Data(value) => value.gencode(scope, instructions),
            Atomic::UnaryOperation(_) => todo!(),
            Atomic::Paren(value) => value.gencode(scope, instructions),
            Atomic::ExprFlow(value) => value.gencode(scope, instructions),
            Atomic::Error(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use tests::{data::Number, operation::Addition};

    use crate::{
        ast::expressions::data::{Data, Primitive},
        semantic::{scope::scope_impl::MockScope, Metadata},
    };

    use super::*;

    #[test]
    fn valid_expression() {
        let res = Expression::<MockScope>::parse(" 1 + 2 * 4".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Addition(Addition {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(Number::U64(1))
                )))),
                right: Box::new(Expression::Product(operation::Product::Mult {
                    left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                        Primitive::Number(Number::U64(2))
                    )))),
                    right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                        Primitive::Number(Number::U64(4))
                    )))),
                    metadata: Metadata::default()
                })),
                metadata: Metadata::default()
            }),
            value
        );
    }
}
