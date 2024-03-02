use std::borrow::Borrow;
use std::cell::{Ref, RefCell};

use crate::ast::utils::strings::ID;
use crate::semantic::scope::static_types::StaticType;
use crate::semantic::{Either, TypeOf};
use crate::vm::allocator::Memory;
use crate::vm::casm::operation::OpPrimitive;
use crate::vm::casm::Casm;
use crate::vm::platform::utils::lexem;
use crate::vm::platform::{GenerateCodePlatform, LibCasm};
use crate::vm::scheduler::Thread;
use crate::vm::vm::{DeserializeFrom, Executable, Printer, Runtime, RuntimeError};
use crate::{
    ast::expressions::Expression,
    semantic::{scope::ScopeApi, EType, MutRc, Resolve, SemanticError},
    vm::{
        casm::CasmProgram,
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::StdCasm;
#[derive(Debug, Clone, PartialEq)]
pub enum IOFn {
    Print(RefCell<Option<EType>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IOCasm {
    Print(PrintCasm),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrintCasm {
    StdOutBufOpen,
    StdOutBufRevFlush,
    StdOutBufFlush,
    PrintID(ID),
    PrintLexem(&'static str),
    PrintU8,
    PrintU16,
    PrintU32,
    PrintU64,
    PrintU128,
    PrintI8,
    PrintI16,
    PrintI32,
    PrintI64,
    PrintI128,
    PrintF64,
    PrintAddr,
    PrintChar,
    PrintBool,
    PrintStr(usize),
    PrintString,
}

impl IOFn {
    pub fn from(id: &String) -> Option<Self> {
        match id.as_str() {
            lexem::PRINT => Some(IOFn::Print(RefCell::default())),
            _ => None,
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for IOFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression<Scope>>;
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            IOFn::Print(param_type) => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let param = extra.first().unwrap();
                let _ = param.resolve(scope, &None, &())?;
                *param_type.borrow_mut() = Some(param.type_of(&scope.as_ref().borrow())?);
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for IOFn {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            IOFn::Print(_) => Ok(Either::Static(StaticType::Unit.into())),
        }
    }
}

impl<Scope: ScopeApi> GenerateCodePlatform<Scope> for IOFn {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
        params_size: usize,
    ) -> Result<(), CodeGenerationError> {
        match self {
            IOFn::Print(inner) => {
                let binding = inner.borrow();

                let Some(param_type) = binding.as_ref() else {
                    dbg!("here");
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let printers = param_type.build_printer()?;
                instructions.extend(printers);

                Ok(())
            }
        }
    }
}

impl Executable for IOCasm {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match self {
            IOCasm::Print(print) => print.execute(thread),
        }
    }
}

impl Executable for PrintCasm {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match self {
            PrintCasm::PrintID(id) => {
                thread.runtime.stdio.stdout.push(&id);
            }
            PrintCasm::PrintLexem(lexem) => {
                thread.runtime.stdio.stdout.push(lexem);
            }
            PrintCasm::PrintU8 => {
                let n = OpPrimitive::get_num1::<u8>(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintU16 => {
                let n = OpPrimitive::get_num2::<u16>(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintU32 => {
                let n = OpPrimitive::get_num4::<u32>(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintU64 => {
                let n = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintU128 => {
                let n = OpPrimitive::get_num16::<u128>(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintI8 => {
                let n = OpPrimitive::get_num1::<i8>(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintI16 => {
                let n = OpPrimitive::get_num2::<i16>(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintI32 => {
                let n = OpPrimitive::get_num4::<i32>(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintI64 => {
                let n = OpPrimitive::get_num8::<i64>(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintI128 => {
                let n = OpPrimitive::get_num16::<i128>(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintF64 => {
                let n = OpPrimitive::get_num8::<f64>(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintAddr => {
                let n = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("{:X}", n));
            }
            PrintCasm::PrintChar => {
                let n = OpPrimitive::get_char(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("'{}'", n));
            }
            PrintCasm::PrintBool => {
                let n = OpPrimitive::get_bool(&thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintStr(size) => {
                let n = OpPrimitive::get_str_slice(*size, &thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("\"{}\"", n));
            }
            PrintCasm::PrintString => {
                let size = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                let n = OpPrimitive::get_str_slice(size as usize, &thread.memory())?;
                thread.runtime.stdio.stdout.push(&format!("\"{}\"", n));
            }
            PrintCasm::StdOutBufOpen => {
                thread.runtime.stdio.stdout.open_buffer();
            }
            PrintCasm::StdOutBufRevFlush => {
                thread.runtime.stdio.stdout.rev_flush_buffer();
            }
            PrintCasm::StdOutBufFlush => {
                thread.runtime.stdio.stdout.flush_buffer();
            }
        }
        thread.env.program.incr();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        ast::{statements::Statement, TryParse},
        semantic::scope::scope_impl::Scope,
    };

    use super::*;

    #[test]
    fn valid_parse() {}

    #[test]
    fn valid_resolve() {}

    #[test]
    fn valid_print_number() {
        for text in vec![
            "u128", "u64", "u32", "u16", "u8", "i128", "i64", "i32", "i16", "i8", "f64", "",
        ] {
            let statement = Statement::parse(format!("print(64{});", text).as_str().into())
                .expect("Parsing should have succeeded")
                .1;
            let scope = Scope::new();
            let _ = statement
                .resolve(&scope, &None, &())
                .expect("Resolution should have succeeded");
            // Code generation.
            let instructions = CasmProgram::default();
            statement
                .gencode(&scope, &instructions)
                .expect("Code generation should have succeeded");

            assert!(instructions.len() > 0, "No instructions generated");

            // Execute the instructions.
            let mut runtime = Runtime::new();
            let tid = runtime
                .spawn()
                .expect("Thread spawning should have succeeded");
            let thread = runtime.get(tid).expect("Thread should exist");
            thread.push_instr(instructions);

            thread.run().expect("Execution should have succeeded");
            let output = runtime.stdio.stdout.take();
            assert_eq!(&output, "64");
        }
    }

    #[test]
    fn valid_print_char() {
        let statement = Statement::parse("print('a');".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);

        thread.run().expect("Execution should have succeeded");
        let output = runtime.stdio.stdout.take();
        assert_eq!(&output, "'a'");
    }
    #[test]
    fn valid_print_bool() {
        for text in vec!["true", "false"] {
            let statement = Statement::parse(format!("print({});", text).as_str().into())
                .expect("Parsing should have succeeded")
                .1;
            let scope = Scope::new();
            let _ = statement
                .resolve(&scope, &None, &())
                .expect("Resolution should have succeeded");
            // Code generation.
            let instructions = CasmProgram::default();
            statement
                .gencode(&scope, &instructions)
                .expect("Code generation should have succeeded");

            assert!(instructions.len() > 0, "No instructions generated");
            // Execute the instructions.
            let mut runtime = Runtime::new();
            let tid = runtime
                .spawn()
                .expect("Thread spawning should have succeeded");
            let thread = runtime.get(tid).expect("Thread should exist");
            thread.push_instr(instructions);

            thread.run().expect("Execution should have succeeded");
            let output = runtime.stdio.stdout.take();
            assert_eq!(&output, text);
        }
    }
    #[test]
    fn valid_print_strslice() {
        for text in vec!["\"Hello World\"", "\"你好世界\""] {
            let statement = Statement::parse(format!("print({});", text).as_str().into())
                .expect("Parsing should have succeeded")
                .1;
            let scope = Scope::new();
            let _ = statement
                .resolve(&scope, &None, &())
                .expect("Resolution should have succeeded");
            // Code generation.
            let instructions = CasmProgram::default();
            statement
                .gencode(&scope, &instructions)
                .expect("Code generation should have succeeded");

            assert!(instructions.len() > 0, "No instructions generated");
            // Execute the instructions.
            let mut runtime = Runtime::new();
            let tid = runtime
                .spawn()
                .expect("Thread spawning should have succeeded");
            let thread = runtime.get(tid).expect("Thread should exist");
            thread.push_instr(instructions);

            thread.run().expect("Execution should have succeeded");
            let output = runtime.stdio.stdout.take();
            assert_eq!(&output, text);
        }
    }

    #[test]
    fn valid_print_strslice_complex() {
        let statement = Statement::parse(
            r##"
        {
            let x = "Hello World";
            print(x);
        }
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);

        thread.run().expect("Execution should have succeeded");
        let output = runtime.stdio.stdout.take();
        assert_eq!(&output, "\"Hello World\"");
    }
    #[test]
    fn valid_print_string() {
        let statement = Statement::parse(
            r##"
        {
            let x = string("Hello World");
            print(x);
        }
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);

        thread.run().expect("Execution should have succeeded");
        let output = runtime.stdio.stdout.take();
        assert_eq!(&output, "\"Hello World\"");
    }

    #[test]
    fn valid_print_tuple() {
        let statement = Statement::parse(
            r##"
            print((420,true));
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);

        thread.run().expect("Execution should have succeeded");
        let output = runtime.stdio.stdout.take();
        assert_eq!(&output, "(420,true)");
        // assert_eq!(&output, "\"Hello World\"");
    }
}
