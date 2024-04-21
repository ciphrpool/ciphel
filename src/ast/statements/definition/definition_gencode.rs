use std::{cell::RefCell, rc::Rc};

use super::{Definition, EventDef, FnDef, TypeDef};
use crate::semantic::SizeOf;
use crate::vm::allocator::stack::Offset;
use crate::vm::allocator::MemoryAddress;
use crate::vm::casm::alloc::{Access, Alloc};
use crate::vm::casm::locate::Locate;
use crate::vm::casm::serialize::Serialized;
use crate::{
    semantic::{scope::ScopeApi, MutRc},
    vm::{
        casm::{
            branch::{Goto, Label},
            memcopy::MemCopy,
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

impl<Scope: ScopeApi> GenerateCode<Scope> for Definition<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Definition::Type(value) => value.gencode(scope, instructions),
            Definition::Fn(value) => value.gencode(scope, instructions),
            Definition::Event(value) => value.gencode(scope, instructions),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for TypeDef {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for FnDef<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let end_closure = Label::gen();

        instructions.push(Casm::Goto(Goto {
            label: Some(end_closure),
        }));

        let closure_label = instructions.push_label(format!("fn_{}", self.id).into());
        let _ = self.scope.gencode(scope, instructions);
        instructions.push_label_id(end_closure, "end_closure".into());

        instructions.push(Casm::MemCopy(MemCopy::LabelOffset(closure_label)));

        let (var, address, level) = scope.as_ref().borrow().access_var(&self.id)?;
        let var_type = &var.as_ref().type_sig;
        let var_size = var_type.size_of();

        instructions.push(Casm::Locate(Locate {
            address: MemoryAddress::Stack {
                offset: address,
                level,
            },
        }));
        instructions.push(Casm::MemCopy(MemCopy::TakeToStack { size: var_size }));
        Ok(())
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for EventDef<Scope> {
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
    use crate::ast::expressions::data::{Number, Primitive};
    use crate::ast::TryParse;
    use crate::semantic::scope::static_types::{NumberType, PrimitiveType};
    use crate::semantic::Resolve;
    use crate::vm::vm::{DeserializeFrom, Runtime};
    use crate::{ast::statements::Statement, semantic::scope::scope_impl::Scope};
    use crate::{clear_stack, compile_statement, v_num};

    #[test]
    fn valid_function() {
        let statement = Statement::parse(
            r##"
        let x = {
            fn f(x:u64) -> u64 {
                return x+1;
            }
            return f(68); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 69));
    }
    #[test]
    fn valid_function_with_stack_env() {
        let statement = Statement::parse(
            r##"
        let x = {
            let env:u64 = 31;
            fn f(x:u64) -> u64 {
                if true {
                    return x + env ;
                }else {
                    return env + x;
                }
            }
            env = 50;
            return f(38); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 88));
    }

    #[test]
    fn valid_function_with_heap_env() {
        let statement = Statement::parse(
            r##"
        let x = {
            let env : Vec<u64> = vec[2,5];

            fn f(x:u64) -> u64 {
                return env[1] + x;
            }
            env[1] = 31;
            return f(38); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 69));
    }

    #[test]
    fn valid_function_rec() {
        let statement = Statement::parse(
            r##"
        let x = {
            fn rec(x:u64) -> u64 {
                if x == 0u64 {
                    return 0;
                }
                return 1u64 + rec(x-1);
            }
            return rec(3); 
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

        // dbg!(&instructions);
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
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 3));
    }

    #[test]
    fn valid_function_fibonacci() {
        let statement = Statement::parse(
            r##"
        let x = {
            fn fibonacci(x:u64) -> u64 {
                if x == 0u64 {
                    return 0;
                } else if x == 1u64 or x == 2u64 {
                    return 1;
                }
                return fibonacci(x-1) + fibonacci(x-2);
            }
            return fibonacci(10);
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

        // dbg!(&instructions);
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
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 55));
    }
}
