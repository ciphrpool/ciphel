use std::{
    cell::{Ref},
};

use nom::{
    branch::alt,
    combinator::{map},
};

use self::return_stat::Return;
use crate::semantic::scope::scope_impl::Scope;

use super::{
    TryParse,
};
use crate::{
    ast::utils::io::{PResult, Span},
    semantic::{
        scope::{static_types::StaticType},
        EType, Either, MutRc, Resolve, SemanticError, TypeOf,
    },
    vm::{
        casm::{
            branch::{Call, Goto, Label},
            Casm, CasmProgram,
        },
        vm::CodeGenerationError,
    },
};
use crate::{
    semantic::{
        scope::{BuildStaticType},
    },
    vm::vm::GenerateCode,
};

pub mod assignation;
pub mod block;
pub mod declaration;
pub mod definition;
pub mod flows;
pub mod loops;
pub mod return_stat;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Scope(block::Block),
    Flow(flows::Flow),
    Assignation(assignation::Assignation),
    Declaration(declaration::Declaration),
    Definition(definition::Definition),
    Loops(loops::Loop),
    Return(Return),
}

impl TryParse for Statement {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(Return::parse, |value| Statement::Return(value)),
            map(block::Block::parse, |value| Statement::Scope(value)),
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
impl Resolve for Statement {
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

impl TypeOf for Statement {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Statement::Scope(value) => value.type_of(&scope),
            Statement::Flow(value) => value.type_of(&scope),
            Statement::Assignation(value) => value.type_of(&scope),
            Statement::Declaration(value) => value.type_of(&scope),
            Statement::Definition(_value) => Ok(Either::Static(
                <StaticType as BuildStaticType>::build_unit().into(),
            )),
            Statement::Loops(value) => value.type_of(&scope),
            Statement::Return(value) => value.type_of(&scope),
        }
    }
}

impl GenerateCode for Statement {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Statement::Scope(value) => {
                // let scope_casm = Rc::new(RefCell::new(CasmProgram::default()));
                // let _ = value.gencode(block, &scope_casm)?;

                // let scope_casm = scope_casm.take();
                // let next_instruction_idx =
                //     instructions.as_ref().borrow().len() + 1 + scope_casm.len();
                // let mut borrowed = instructions
                //     .as_ref()
                //     .try_borrow_mut()
                //     .map_err(|_| CodeGenerationError::Default)?;
                // instructions.push(Casm::Serialize(Serialized {
                //     data: (next_instruction_idx as u64).to_le_bytes().to_vec(),
                // }));
                // borrowed.extend(scope_casm.main);
                let scope_label = Label::gen();
                let end_scope_label = Label::gen();

                instructions.push(Casm::Goto(Goto {
                    label: Some(end_scope_label),
                }));
                instructions.push_label_id(scope_label, "block".into());

                let _ = value.gencode(scope, &instructions)?;

                instructions.push_label_id(end_scope_label, "end_scope".into());
                instructions.push(Casm::Call(Call::From {
                    label: scope_label,
                    param_size: 0,
                }));
                instructions.push(Casm::Pop(9)); /* Pop the unused return size and return flag */
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
