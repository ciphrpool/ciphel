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
    semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError, TypeOf},
};

use self::operation::operation_parse::TryParseOperation;

use super::TryParse;

pub mod data;
pub mod error;
pub mod flows;
pub mod operation;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    HighOrdMath(operation::HighOrdMath),
    LowOrdMath(operation::LowOrdMath),
    Shift(operation::Shift),
    BitwiseAnd(operation::BitwiseAnd),
    BitwiseXOR(operation::BitwiseXOR),
    BitwiseOR(operation::BitwiseOR),
    Comparaison(operation::Comparaison),
    LogicalAnd(operation::LogicalAnd),
    LogicalOr(operation::LogicalOr),
    Atomic(Atomic),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atomic {
    Data(data::Data),
    UnaryOperation(operation::UnaryOperation),
    Paren(Box<Expression>),
    ExprFlow(flows::ExprFlow),
    Error(error::Error),
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

impl<Scope: ScopeApi> Resolve<Scope> for Atomic {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Atomic::Data(value) => value.resolve(scope, context),
            Atomic::UnaryOperation(value) => value.resolve(scope, context),
            Atomic::Paren(value) => value.resolve(scope, context),
            Atomic::ExprFlow(value) => value.resolve(scope, context),
            Atomic::Error(value) => value.resolve(scope, &()),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Atomic {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
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

impl TryParse for Expression {
    /*
     * @desc Parse an expression
     *
     * @grammar
     * Expr := AndOp Or AndOp | AndOp
     * AndOp := CompOp And CompOp | CompOp
     * CompOp := BOr (<|<=|>=|>|\=\=|\!\= | in ) BOr | BOr
     * BOr := XOr \| XOr  | XOr
     * XOr := BAnd ^ BAnd | BAnd
     * BAnd := Shift & Shift | Shift
     * Shift := LowM (<<|>>) LowM | LowM
     * LowM := HighM + HighM | HighM
     * HighM := Atom (* | / | % ) Atom | Atom
     */
    fn parse(input: Span) -> PResult<Self> {
        eater::ws(LogicalOr::parse)(input)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Expression {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Expression::HighOrdMath(value) => value.resolve(scope, context),
            Expression::LowOrdMath(value) => value.resolve(scope, context),
            Expression::Shift(value) => value.resolve(scope, context),
            Expression::BitwiseAnd(value) => value.resolve(scope, context),
            Expression::BitwiseXOR(value) => value.resolve(scope, context),
            Expression::BitwiseOR(value) => value.resolve(scope, context),
            Expression::Comparaison(value) => value.resolve(scope, context),
            Expression::LogicalAnd(value) => value.resolve(scope, context),
            Expression::LogicalOr(value) => value.resolve(scope, context),
            Expression::Atomic(value) => value.resolve(scope, context),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Expression {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Expression::HighOrdMath(value) => value.type_of(&scope),
            Expression::LowOrdMath(value) => value.type_of(&scope),
            Expression::Shift(value) => value.type_of(&scope),
            Expression::BitwiseAnd(value) => value.type_of(&scope),
            Expression::BitwiseXOR(value) => value.type_of(&scope),
            Expression::BitwiseOR(value) => value.type_of(&scope),
            Expression::Comparaison(value) => value.type_of(&scope),
            Expression::LogicalAnd(value) => value.type_of(&scope),
            Expression::LogicalOr(value) => value.type_of(&scope),
            Expression::Atomic(value) => value.type_of(&scope),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Box<Expression> {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        (self.as_ref()).type_of(&scope)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Box<Expression> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        (self.as_ref()).resolve(scope, context)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::expressions::{
        data::{Data, Primitive},
        operation::LowOrdMath,
    };

    use super::*;

    #[test]
    fn valid_expression() {
        let res = Expression::parse(" 1 + 2 * 4".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::LowOrdMath(LowOrdMath::Add {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(1)
                )))),
                right: Box::new(Expression::HighOrdMath(operation::HighOrdMath::Mult {
                    left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                        Primitive::Number(2)
                    )))),
                    right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                        Primitive::Number(4)
                    ))))
                }))
            }),
            value
        );
    }
}
