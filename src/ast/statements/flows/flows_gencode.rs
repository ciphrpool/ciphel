use std::{cell::RefCell, rc::Rc};

use ulid::Ulid;

use crate::{
    ast::{
        expressions::Expression,
        statements::scope::{scope_gencode::inner_scope_gencode, Scope},
    },
    semantic::{scope::ScopeApi, MutRc, SizeOf},
    vm::{
        casm::{
            alloc::StackFrame,
            branch::{BranchIf, Call, Goto, Label},
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{CallStat, Flow, IfStat, MatchStat, TryStat};

impl<Scope: ScopeApi> GenerateCode<Scope> for Flow<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Flow::If(value) => value.gencode(scope, instructions),
            Flow::Match(value) => value.gencode(scope, instructions),
            Flow::Try(value) => value.gencode(scope, instructions),
            Flow::Call(value) => value.gencode(scope, instructions),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for CallStat<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        self.call.gencode(scope, instructions)
    }
}

impl<InnerScope: ScopeApi> GenerateCode<InnerScope> for IfStat<InnerScope> {
    fn gencode(
        &self,
        scope: &MutRc<InnerScope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let mut else_if_labels: Vec<Ulid> = Vec::default();
        let else_label = match &self.else_branch {
            Some(_) => Some(Label::gen()),
            None => None,
        };
        let end_if_label = Label::gen();

        for (_, _) in &self.else_if_branches {
            else_if_labels.push(Label::gen());
        }

        let _ = self.condition.gencode(scope, &instructions)?;

        match &self.else_if_branches.first() {
            Some(_) => {
                instructions.push(Casm::If(BranchIf {
                    else_label: *else_if_labels.first().unwrap_or(&end_if_label),
                }));
            }
            None => {
                instructions.push(Casm::If(BranchIf {
                    else_label: else_label.unwrap_or(end_if_label),
                }));
            }
        }
        // let _ = self.then_branch.gencode(scope, &instructions)?;
        let _ = inner_scope_gencode(scope, &self.then_branch, None, false, instructions)?;

        for pair in self
            .else_if_branches
            .iter()
            .zip(&else_if_labels)
            .collect::<Vec<(&(Expression<InnerScope>, Scope<InnerScope>), &Ulid)>>()
            .windows(2)
        {
            let ((cond_1, scope_1), label_1) = &pair[0];
            let ((_, _), label_2) = &pair[1];
            instructions.push_label_id(**label_1, "else_if".into());
            let _ = cond_1.gencode(scope, &instructions)?;
            instructions.push(Casm::If(BranchIf {
                else_label: **label_2,
            }));
            // let _ = scope_1.gencode(scope, instructions)?;
            let _ = inner_scope_gencode(scope, &scope_1, None, false, instructions)?;
        }
        if let Some((cond, s)) = &self.else_if_branches.last() {
            instructions.push_label_id(*else_if_labels.last().unwrap(), "else_if".into());
            let _ = cond.gencode(scope, &instructions)?;
            instructions.push(Casm::If(BranchIf {
                else_label: else_label.unwrap_or(end_if_label),
            }));
            // let _ = s.gencode(scope, instructions)?;
            let _ = inner_scope_gencode(scope, &s, None, false, instructions)?;
        }

        if let Some(s) = &self.else_branch {
            instructions.push_label_id(else_label.unwrap(), "else".into());
            let _ = s.gencode(scope, instructions)?;
            let _ = inner_scope_gencode(scope, &s, None, false, instructions)?;
        }

        instructions.push_label_id(end_if_label, "end_if".into());
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for MatchStat<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for TryStat<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}
