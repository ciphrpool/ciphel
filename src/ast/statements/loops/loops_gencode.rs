use crate::semantic::SizeOf;
use crate::{
    ast::statements::scope::Scope,
    semantic::{scope::ScopeApi, MutRc},
    vm::{
        allocator::stack::UReg,
        casm::{
            alloc::StackFrame,
            branch::{Call, Goto, Label},
            memcopy::MemCopy,
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};
use std::{cell::RefCell, rc::Rc};

use super::{ForLoop, Loop, WhileLoop};

fn scope_gencode<S: ScopeApi>(
    scope: &MutRc<S>,
    value: &Scope<S>,
    return_size: Option<usize>,
    param_size: Option<usize>,
    instructions: &CasmProgram,
) -> Result<(), CodeGenerationError> {
    let scope_label = Label::gen();
    let end_scope_label = Label::gen();

    instructions.push(Casm::Goto(Goto {
        label: Some(end_scope_label),
    }));

    instructions.push_label_id(scope_label, "scope_loop".into());

    let _ = value.gencode(scope, &instructions)?;

    instructions.push_label_id(end_scope_label, "end_scope_loop".into());
    instructions.push(Casm::Call(Call::From {
        label: scope_label,
        return_size: return_size.unwrap_or(0),
        param_size: param_size.unwrap_or(0),
    }));
    if let Some(return_size) = return_size {
        if return_size > 0 {
            instructions.push(Casm::StackFrame(StackFrame::Return { return_size }));
        }
    }
    Ok(())
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Loop<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Loop::For(value) => value.gencode(scope, instructions),
            Loop::While(value) => value.gencode(scope, instructions),
            Loop::Loop(value) => {
                let start_label = Label::gen();
                let early_end_label = Label::gen();
                let end_label = Label::gen();

                instructions.push_label_id(start_label, "start_loop".into());
                instructions.push(Casm::MemCopy(MemCopy::LabelOffset(early_end_label)));
                instructions.push(Casm::MemCopy(MemCopy::SetReg(UReg::R4, None)));
                let _ = scope_gencode(
                    scope,
                    value,
                    value.metadata.signature().map(|s| s.size_of()),
                    None,
                    instructions,
                )?;
                instructions.push(Casm::Goto(Goto {
                    label: Some(start_label),
                }));
                instructions.push_label_id(early_end_label, "early_end_label".into());
                instructions.push(Casm::StackFrame(StackFrame::SoftClean));
                instructions.push_label_id(end_label, "end_label".into());
                Ok(())
            }
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for ForLoop<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for WhileLoop<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::ast::TryParse;
    use crate::semantic::Resolve;
    use crate::{
        ast::{
            expressions::data::{Number, Primitive},
            statements::Statement,
        },
        clear_stack,
        semantic::scope::{
            scope_impl::Scope,
            static_types::{NumberType, PrimitiveType},
        },
        vm::vm::{DeserializeFrom, Runtime},
    };

    #[test]
    fn valid_loop() {
        let statement = Statement::parse(
            r##"
            let x = {
                let i:u64 = 0;
                loop {
                    i = i + 1;
                    if i >= 10u64 {
                        break;
                    }
                }
                return i; 
            };
            "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);
        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Cell::new(Number::I64(10))));
    }
}
