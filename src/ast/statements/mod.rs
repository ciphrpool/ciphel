use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use nom::{
    branch::alt,
    combinator::{map, opt, value},
    sequence::delimited,
};

use self::return_stat::Return;

use super::{
    expressions::Expression,
    utils::{lexem, strings::wst},
    TryParse,
};
use crate::{
    ast::utils::io::{PResult, Span},
    semantic::{
        scope::{static_types::StaticType, user_type_impl::UserType, ScopeApi},
        EType, Either, MutRc, Resolve, SemanticError, TypeOf,
    },
    vm::{
        casm::{serialize::Serialized, Casm, CasmProgram},
        vm::CodeGenerationError,
    },
};
use crate::{
    semantic::{
        scope::{type_traits::TypeChecking, BuildStaticType},
        CompatibleWith,
    },
    vm::vm::GenerateCode,
};

pub mod assignation;
pub mod declaration;
pub mod definition;
pub mod flows;
pub mod loops;
pub mod return_stat;
pub mod scope;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement<InnerScope: ScopeApi> {
    Scope(scope::Scope<InnerScope>),
    Flow(flows::Flow<InnerScope>),
    Assignation(assignation::Assignation<InnerScope>),
    Declaration(declaration::Declaration<InnerScope>),
    Definition(definition::Definition<InnerScope>),
    Loops(loops::Loop<InnerScope>),
    Return(Return<InnerScope>),
}

impl<InnerScope: ScopeApi> TryParse for Statement<InnerScope> {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(Return::parse, |value| Statement::Return(value)),
            map(scope::Scope::parse, |value| Statement::Scope(value)),
            map(flows::Flow::parse, |value| Statement::Flow(value)),
            map(assignation::Assignation::parse, |value| {
                Statement::Assignation(value)
            }),
            map(declaration::Declaration::parse, |value| {
                Statement::Declaration(value)
            }),
            map(definition::Definition::parse, |value| {
                Statement::Definition(value)
            }),
            map(loops::Loop::parse, |value| Statement::Loops(value)),
        ))(input)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Statement<Scope> {
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
            Statement::Scope(value) => {
                let _ = value.resolve(scope, context, &Vec::default())?;
                Ok(())
            }
            Statement::Flow(value) => value.resolve(scope, context, extra),
            Statement::Assignation(value) => value.resolve(scope, context, extra),
            Statement::Declaration(value) => value.resolve(scope, context, extra),
            Statement::Definition(value) => value.resolve(scope, context, extra),
            Statement::Loops(value) => value.resolve(scope, context, extra),
            Statement::Return(value) => value.resolve(scope, context, extra),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Statement<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Statement::Scope(value) => value.type_of(&scope),
            Statement::Flow(value) => value.type_of(&scope),
            Statement::Assignation(value) => value.type_of(&scope),
            Statement::Declaration(value) => value.type_of(&scope),
            Statement::Definition(_value) => Ok(Either::Static(
                <StaticType as BuildStaticType<Scope>>::build_unit().into(),
            )),
            Statement::Loops(value) => value.type_of(&scope),
            Statement::Return(value) => value.type_of(&scope),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Statement<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Statement::Scope(value) => {
                let scope_casm = Rc::new(RefCell::new(CasmProgram::default()));
                let _ = value.gencode(scope, &scope_casm)?;

                let scope_casm = scope_casm.take();
                let next_instruction_idx =
                    instructions.as_ref().borrow().len() + 1 + scope_casm.len();
                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::Serialize(Serialized {
                    data: (next_instruction_idx as u64).to_le_bytes().to_vec(),
                }));
                borrowed.extend(scope_casm.main);

                Ok(())
            }
            Statement::Flow(value) => value.gencode(scope, instructions),
            Statement::Assignation(value) => value.gencode(scope, instructions),
            Statement::Declaration(value) => value.gencode(scope, instructions),
            Statement::Definition(value) => value.gencode(scope, instructions),
            Statement::Loops(value) => value.gencode(scope, instructions),
            Statement::Return(value) => value.gencode(scope, instructions),
        }
    }
}

#[macro_export]
macro_rules! resolve_metadata {
    ($metadata:expr,$self:expr,$scope:expr,$context:expr) => {{
        let mut borrowed_metadata = $metadata
            .info
            .as_ref()
            .try_borrow_mut()
            .map_err(|_| SemanticError::Default)?;
        *borrowed_metadata = Info::Resolved {
            context: $context.clone(),
            signature: Some($self.type_of(&$scope.borrow())?),
        };
    }};
}
