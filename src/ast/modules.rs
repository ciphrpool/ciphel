use nom::{
    branch::alt,
    combinator::map,
    multi::fold_many0,
    sequence::{delimited, pair, preceded},
    Finish,
};

use crate::{
    ast::utils::{
        lexem,
        strings::{parse_id, wst},
    },
    semantic::{scope::scope::Variable, Desugar, Resolve},
    vm::{external::ExternThreadIdentifier, GenerateCode},
    CompilationError,
};

use super::{
    statements::definition::{FnDef, TypeDef},
    utils::{error::generate_error_report, io::Span},
    TryParse,
};

#[derive(Debug, Clone, Default)]
pub struct Module {
    name: String,
    types: Vec<TypeDef>,
    functions: Vec<FnDef>,
}

impl Module {
    pub fn find_var(&self, path: &[String], name: &str) -> Option<Variable> {
        if path.len() != 1 {
            return None;
        }
        if path[0] != self.name {
            return None;
        }
        match self.functions.iter().find(|func| &func.name == name) {
            Some(func) => {
                let Some((id, _, ctype)) = &func.id else {
                    return None;
                };
                return Some(Variable {
                    id: id.clone(),
                    ctype: ctype.clone(),
                    scope: None,
                });
            }
            None => return None,
        }
    }
}

impl TryParse for Module {
    fn parse(input: super::utils::io::Span) -> super::utils::io::PResult<Self>
    where
        Self: Sized,
    {
        enum ModuleItem {
            Type(TypeDef),
            Function(FnDef),
        }
        map(
            pair(
                preceded(wst(lexem::MODULE), parse_id),
                delimited(
                    wst(lexem::BRA_O),
                    fold_many0(
                        alt((
                            map(TypeDef::parse, ModuleItem::Type),
                            map(FnDef::parse, ModuleItem::Function),
                        )),
                        Module::default,
                        |mut acc, value| match value {
                            ModuleItem::Type(type_def) => {
                                acc.types.push(type_def);
                                acc
                            }
                            ModuleItem::Function(fn_def) => {
                                acc.functions.push(fn_def);
                                acc
                            }
                        },
                    ),
                    wst(lexem::BRA_C),
                ),
            ),
            |(name, mut module)| {
                module.name = name;
                module
            },
        )(input)
    }
}

impl Resolve for Module {
    type Output = ();
    type Context = ();
    type Extra = ();

    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, crate::semantic::SemanticError>
    where
        Self: Sized,
    {
        for t in self.types.iter_mut() {
            let _ = t.resolve::<E>(scope_manager, scope_id, context, extra)?;
        }

        for func in self.functions.iter_mut() {
            let _ = func.desugar::<E>(scope_manager, scope_id)?;
            let _ = func.resolve::<E>(scope_manager, scope_id, context, extra)?;
        }

        Ok(())
    }
}

impl GenerateCode for Module {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        for func in self.functions.iter() {
            func.gencode(scope_manager, scope_id, instructions, context)?;
        }
        Ok(())
    }
}

pub fn parse_module<TID: ExternThreadIdentifier>(
    module: Span,
    line_offset: usize,
) -> Result<Module, CompilationError<TID>> {
    match Module::parse(module).finish() {
        Ok((remaining, module)) => {
            if !remaining.fragment().is_empty() {
                let error_report = format!(
                    "Parsing completed, but input remains: {:?}",
                    remaining.fragment()
                );
                Err(CompilationError::ParsingError(error_report))
            } else {
                Ok(module)
            }
        }
        Err(e) => {
            let error_report = generate_error_report(&module, &e, line_offset);
            Err(CompilationError::ParsingError(error_report))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{test_extract_variable, Ciphel};

    use super::*;

    #[test]
    fn valid_module() {
        let mut engine = crate::vm::external::test::NoopEngine {};
        let mut ciphel = Ciphel::<
            crate::vm::external::test::NoopEngine,
            crate::vm::scheduler::ToCompletion,
        >::default();

        let tid = ciphel
            .runtime
            .spawn(&mut engine)
            .expect("Spawning should have succeeded");

        ciphel
            .import(
                &[tid.clone()],
                r##"
        
        module Test {
        
            fn test() -> i64 {
                5
            }
        
        }    
            "##,
                0,
            )
            .expect("Module parsing should have succeeded");

        ciphel
            .compile(
                tid.clone(),
                r##"
        let res1 = Test::test();
        fn test() -> i64 {
            6
        }
        let res2 = Test::test();
        let res3 = test();

            "##,
                0,
            )
            .expect("Compilation should have succeeded");

        ciphel
            .run(&mut engine)
            .expect("Execution should have succeeded");

        let (
            crate::vm::runtime::Thread { stack, .. },
            crate::vm::runtime::ThreadContext { scope_manager, .. },
        ) = ciphel
            .runtime
            .thread_with_context_of(&tid)
            .expect("Thread should have been found");

        let res = test_extract_variable::<i64>("res1", scope_manager, stack, &ciphel.heap)
            .expect("Deserialization should have succeeded");
        assert_eq!(res, 5);

        let res = test_extract_variable::<i64>("res2", scope_manager, stack, &ciphel.heap)
            .expect("Deserialization should have succeeded");
        assert_eq!(res, 5);

        let res = test_extract_variable::<i64>("res3", scope_manager, stack, &ciphel.heap)
            .expect("Deserialization should have succeeded");
        assert_eq!(res, 6);
    }
}
