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
use crate::vm::vm::{DeserializeFrom, Executable, Printer, RuntimeError};
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
    PrintID(ID),
    PrintSep,
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
        instructions: &MutRc<CasmProgram>,
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
                let mut borrowed = instructions.as_ref().borrow_mut();
                borrowed.extend(printers);

                Ok(())
            }
        }
    }
}

impl Executable for IOCasm {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match self {
            IOCasm::Print(print) => print.execute(program, memory),
        }
    }
}

impl Executable for PrintCasm {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match self {
            PrintCasm::PrintID(id) => {
                memory.stdout.as_ref().borrow_mut().push_str(&id);
            }
            PrintCasm::PrintSep => {
                memory.stdout.as_ref().borrow_mut().push_str("::");
            }
            PrintCasm::PrintU8 => {
                let n = OpPrimitive::get_num1::<u8>(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("{}", n));
            }
            PrintCasm::PrintU16 => {
                let n = OpPrimitive::get_num2::<u16>(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("{}", n));
            }
            PrintCasm::PrintU32 => {
                let n = OpPrimitive::get_num4::<u32>(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("{}", n));
            }
            PrintCasm::PrintU64 => {
                let n = OpPrimitive::get_num8::<u64>(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("{}", n));
            }
            PrintCasm::PrintU128 => {
                let n = OpPrimitive::get_num16::<u128>(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("{}", n));
            }
            PrintCasm::PrintI8 => {
                let n = OpPrimitive::get_num1::<i8>(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("{}", n));
            }
            PrintCasm::PrintI16 => {
                let n = OpPrimitive::get_num2::<i16>(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("{}", n));
            }
            PrintCasm::PrintI32 => {
                let n = OpPrimitive::get_num4::<i32>(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("{}", n));
            }
            PrintCasm::PrintI64 => {
                let n = OpPrimitive::get_num8::<i64>(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("{}", n));
            }
            PrintCasm::PrintI128 => {
                let n = OpPrimitive::get_num16::<i128>(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("{}", n));
            }
            PrintCasm::PrintF64 => {
                let n = OpPrimitive::get_num8::<f64>(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("{}", n));
            }
            PrintCasm::PrintAddr => {
                let n = OpPrimitive::get_num8::<u64>(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("{:X}", n));
            }
            PrintCasm::PrintChar => {
                let n = OpPrimitive::get_char(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("'{}'", n));
            }
            PrintCasm::PrintBool => {
                let n = OpPrimitive::get_bool(memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("{}", n));
            }
            PrintCasm::PrintStr(size) => {
                let n = OpPrimitive::get_str_slice(*size, memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("\"{}\"", n));
            }
            PrintCasm::PrintString => {
                let size = OpPrimitive::get_num8::<u64>(memory)?;
                let n = OpPrimitive::get_str_slice(size as usize, memory)?;
                memory
                    .stdout
                    .as_ref()
                    .borrow_mut()
                    .push_str(&format!("\"{}\"", n));
            }
        }
        program.incr();
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
            let instructions = Rc::new(RefCell::new(CasmProgram::default()));
            statement
                .gencode(&scope, &instructions)
                .expect("Code generation should have succeeded");

            let instructions = instructions.as_ref().take();
            assert!(instructions.len() > 0, "No instructions generated");
            // Execute the instructions.
            let memory = Memory::new();
            instructions
                .execute(&memory)
                .expect("Execution should have succeeded");
            let output = memory.stdout.as_ref().take();
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
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let output = memory.stdout.as_ref().take();
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
            let instructions = Rc::new(RefCell::new(CasmProgram::default()));
            statement
                .gencode(&scope, &instructions)
                .expect("Code generation should have succeeded");

            let instructions = instructions.as_ref().take();
            assert!(instructions.len() > 0, "No instructions generated");
            // Execute the instructions.
            let memory = Memory::new();
            instructions
                .execute(&memory)
                .expect("Execution should have succeeded");
            let output = memory.stdout.as_ref().take();
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
            let instructions = Rc::new(RefCell::new(CasmProgram::default()));
            statement
                .gencode(&scope, &instructions)
                .expect("Code generation should have succeeded");

            let instructions = instructions.as_ref().take();
            assert!(instructions.len() > 0, "No instructions generated");
            // Execute the instructions.
            let memory = Memory::new();
            instructions
                .execute(&memory)
                .expect("Execution should have succeeded");
            let output = memory.stdout.as_ref().take();
            assert_eq!(&output, text);
        }
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
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let output = memory.stdout.as_ref().take();
        assert_eq!(&output, "\"Hello World\"");
    }
}
