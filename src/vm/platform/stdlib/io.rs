use ulid::Ulid;

use crate::ast::utils::strings::ID;
use crate::e_static;
use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::static_types::{StaticType, StringType};
use crate::semantic::TypeOf;

use crate::vm::allocator::align;
use crate::vm::allocator::heap::Heap;
use crate::vm::allocator::stack::Stack;
use crate::vm::casm::branch::Label;
use crate::vm::casm::operation::OpPrimitive;
use crate::vm::casm::Casm;
use crate::vm::platform::utils::lexem;

use crate::vm::platform::LibCasm;
use crate::vm::stdio::StdIO;
use crate::vm::vm::{CasmMetadata, Executable, Printer, RuntimeError};
use crate::{
    ast::expressions::Expression,
    semantic::{EType, Resolve, SemanticError},
    vm::{
        casm::CasmProgram,
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{ERROR_VALUE, OK_VALUE};

#[derive(Debug, Clone, PartialEq)]
pub enum IOFn {
    Scan,
    Print(Option<EType>),
    Println(Option<EType>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IOCasm {
    Print(PrintCasm),
    Flush(bool),
    Scan,
    RequestScan,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for IOCasm {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            IOCasm::Print(_) => stdio.push_casm_lib(engine, "print"),
            IOCasm::Flush(true) => stdio.push_casm_lib(engine, "flushln"),
            IOCasm::Flush(false) => stdio.push_casm_lib(engine, "flush"),
            IOCasm::Scan => stdio.push_casm_lib(engine, "scan"),
            IOCasm::RequestScan => stdio.push_casm_lib(engine, "rscan"),
        }
    }
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
    PrintString,
    PrintError,
    PrintList {
        length: Option<usize>,
        continue_label: Ulid,
        end_label: Ulid,
    },
}

impl IOFn {
    pub fn from(suffixe: &Option<ID>, id: &ID) -> Option<Self> {
        match suffixe {
            Some(suffixe) => {
                if **suffixe != lexem::IO {
                    return None;
                }
            }
            None => {}
        }
        match id.as_str() {
            lexem::PRINT => Some(IOFn::Print(None)),
            lexem::PRINTLN => Some(IOFn::Println(None)),
            lexem::SCAN => Some(IOFn::Scan),
            _ => None,
        }
    }
}
impl Resolve for IOFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression>;
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        _context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            IOFn::Print(param_type) => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let param = extra.first_mut().unwrap();
                let _ = param.resolve::<G>(scope, &None, &mut None)?;
                *param_type = Some(
                    param.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?,
                );
                Ok(())
            }
            IOFn::Println(param_type) => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let param = extra.first_mut().unwrap();
                let _ = param.resolve::<G>(scope, &None, &mut None)?;
                *param_type = Some(
                    param.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?,
                );
                Ok(())
            }
            IOFn::Scan => {
                if extra.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }
                Ok(())
            }
        }
    }
}
impl TypeOf for IOFn {
    fn type_of(&self, _scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            IOFn::Print(_) => Ok(e_static!(StaticType::Unit)),
            IOFn::Println(_) => Ok(e_static!(StaticType::Unit)),
            IOFn::Scan => Ok(e_static!(StaticType::String(StringType()))),
        }
    }
}

impl GenerateCode for IOFn {
    fn gencode(
        &self,
        _scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            IOFn::Print(inner) => {
                let binding = inner;

                let Some(param_type) = binding.as_ref() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = param_type.build_printer(instructions)?;
                instructions.push(Casm::Platform(LibCasm::Std(super::StdCasm::IO(
                    IOCasm::Flush(false),
                ))));

                Ok(())
            }
            IOFn::Println(inner) => {
                let binding = inner;

                let Some(param_type) = binding.as_ref() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = param_type.build_printer(instructions)?;

                instructions.push(Casm::Platform(LibCasm::Std(super::StdCasm::IO(
                    IOCasm::Flush(true),
                ))));
                Ok(())
            }
            IOFn::Scan => {
                instructions.push(Casm::Platform(LibCasm::Std(super::StdCasm::IO(
                    IOCasm::RequestScan,
                ))));
                instructions.push(Casm::Platform(LibCasm::Std(super::StdCasm::IO(
                    IOCasm::Scan,
                ))));
                Ok(())
            }
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for IOCasm {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match self {
            IOCasm::Print(print) => print.execute(program, stack, heap, stdio, engine),
            IOCasm::Flush(ln) => {
                if *ln {
                    stdio.stdout.flushln(engine);
                } else {
                    stdio.stdout.flush(engine);
                }
                program.incr();
                Ok(())
            }
            IOCasm::Scan => {
                let res = stdio.stdin.read(engine);
                if let Some(content) = res {
                    // Alloc and fill the string with the content, then ^push the address onto the stack
                    let len = content.len();
                    let cap = align(len as usize) as u64;
                    let alloc_size = cap + 16;

                    let len_bytes = len.to_le_bytes().as_slice().to_vec();
                    let cap_bytes = cap.to_le_bytes().as_slice().to_vec();

                    let address = heap.alloc(alloc_size as usize)?;
                    let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

                    let data = content.as_bytes();
                    /* Write len */
                    let _ = heap.write(address, &len_bytes)?;
                    /* Write capacity */
                    let _ = heap.write(address + 8, &cap_bytes)?;
                    /* Write slice */
                    let _ = heap.write(address + 16, &data.to_vec())?;

                    let _ = stack.push_with(&address.to_le_bytes())?;
                    program.incr();
                    Ok(())
                } else {
                    // the program instruction cursor is not increment, therefore when content will be available in the stdin
                    // the instruction will be run again and only then the program cursor will get incremented
                    Err(RuntimeError::Signal(crate::vm::vm::Signal::WAIT_STDIN))
                }
            }
            IOCasm::RequestScan => {
                stdio.stdin.request(engine);
                program.incr();
                Err(RuntimeError::Signal(crate::vm::vm::Signal::WAIT_STDIN))
            }
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for PrintCasm {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match self {
            PrintCasm::PrintID(id) => {
                stdio.stdout.push(&id);
            }
            PrintCasm::PrintLexem(lexem) => {
                stdio.stdout.push(lexem);
            }
            PrintCasm::PrintU8 => {
                let n = OpPrimitive::get_num1::<u8>(stack)?;
                stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintU16 => {
                let n = OpPrimitive::get_num2::<u16>(stack)?;
                stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintU32 => {
                let n = OpPrimitive::get_num4::<u32>(stack)?;
                stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintU64 => {
                let n = OpPrimitive::get_num8::<u64>(stack)?;
                stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintU128 => {
                let n = OpPrimitive::get_num16::<u128>(stack)?;
                stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintI8 => {
                let n = OpPrimitive::get_num1::<i8>(stack)?;
                stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintI16 => {
                let n = OpPrimitive::get_num2::<i16>(stack)?;
                stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintI32 => {
                let n = OpPrimitive::get_num4::<i32>(stack)?;
                stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintI64 => {
                let n = OpPrimitive::get_num8::<i64>(stack)?;
                stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintI128 => {
                let n = OpPrimitive::get_num16::<i128>(stack)?;
                stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintF64 => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;
                stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintAddr => {
                let n = OpPrimitive::get_num8::<u64>(stack)?;
                stdio.stdout.push(&format!("0x{:X}", n));
            }
            PrintCasm::PrintChar => {
                let n = OpPrimitive::get_char(stack)?;
                stdio.stdout.push(&format!("'{}'", n));
            }
            PrintCasm::PrintBool => {
                let n = OpPrimitive::get_bool(stack)?;
                stdio.stdout.push(&format!("{}", n));
            }
            PrintCasm::PrintString => {
                let n = OpPrimitive::get_str_slice(stack)?;
                let n = n.trim_end_matches(char::from(0));
                stdio.stdout.push(&format!("\"{}\"", n));
            }

            PrintCasm::PrintError => {
                let n = OpPrimitive::get_num1::<u8>(stack)?;
                if n == ERROR_VALUE[0] {
                    stdio.stdout.push("Error");
                }else if n == OK_VALUE[0]{
                    stdio.stdout.push("Ok");
                }
            },
            PrintCasm::StdOutBufOpen => {
                stdio.stdout.spawn_buffer();
            }
            PrintCasm::StdOutBufRevFlush => {
                stdio.stdout.rev_flush_buffer();
            }
            PrintCasm::StdOutBufFlush => {
                stdio.stdout.flush_buffer();
            }
            PrintCasm::PrintList {
                length,
                continue_label,
                end_label,
            } => {
                let length = match length {
                    Some(length) => *length,
                    None => {
                        let n = OpPrimitive::get_num8::<u64>(stack)?;
                        n as usize
                    }
                };

                program.incr();
                let start = program.cursor_get();

                stdio.stdout.spawn_buffer();

                stdio.stdout.push(crate::ast::utils::lexem::SQ_BRA_C);
                for idx in 0..length {
                    loop {
                        let cursor = program.cursor_get();
                        let instruction = match program.main.get(cursor) {
                            Some(instruction) => instruction.clone(), // Clone to avoid borrow conflict
                            None => return Ok(()),
                        };
                        match instruction {
                            Casm::Label(Label { id, .. }) => {
                                if id == *continue_label {
                                    program.cursor_set(start);
                                    break;
                                } else {
                                    program.incr();
                                }
                            }
                            _ => {
                                let _ = instruction.execute(program, stack, heap, stdio, engine)?;
                            }
                        }
                    }
                    if idx != length - 1 {
                        stdio.stdout.push(crate::ast::utils::lexem::COMA);
                    }
                }

                stdio.stdout.push(crate::ast::utils::lexem::SQ_BRA_O);
                stdio.stdout.rev_flush_buffer();
                let Some(idx) = program.get(end_label) else {
                    return Err(RuntimeError::CodeSegmentation);
                };
                program.cursor_set(idx);
                return Ok(());
            }
        }
        program.incr();
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{statements::Statement, TryParse},
        compile_statement_for_stdout,
        semantic::scope::scope::Scope,
        vm::vm::{Runtime, StdinTestGameEngine},
        Ciphel,
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
            let mut statement = Statement::parse(format!("print(64{});", text).as_str().into())
                .expect("Parsing should have succeeded")
                .1;

            let output = compile_statement_for_stdout!(statement);
            assert_eq!(&output, "64");
        }
    }

    #[test]
    fn valid_print_char() {
        let mut statement = Statement::parse("print('a');".into())
            .expect("Parsing should have succeeded")
            .1;
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "'a'");
    }
    #[test]
    fn valid_print_bool() {
        for text in vec!["true", "false"] {
            let mut statement = Statement::parse(format!("print({});", text).as_str().into())
                .expect("Parsing should have succeeded")
                .1;

            let output = compile_statement_for_stdout!(statement);
            assert_eq!(&output, text);
        }
    }
    #[test]
    fn valid_print_strslice_complex() {
        for text in vec!["\"Hello World\"", "\"你好世界\""] {
            let mut statement = Statement::parse(format!("print({});", text).as_str().into())
                .expect("Parsing should have succeeded")
                .1;

            let output = compile_statement_for_stdout!(statement);
            assert_eq!(&output, text.trim_matches('\"'));
        }
    }

    #[test]
    fn valid_print_strslice_with_padding() {
        let mut statement = Statement::parse(
            r##"
        {
            let x:str<20> = "Hello World";
            print(x);
        }
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "Hello World");
    }
    #[test]
    fn valid_print_strslice() {
        let mut statement = Statement::parse(
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
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "Hello World");
    }
    #[test]
    fn valid_print_string() {
        let mut statement = Statement::parse(
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
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "Hello World");
    }

    #[test]
    fn valid_print_tuple() {
        let mut statement = Statement::parse(
            r##"
            print((420,true));
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "(420,true)");
        // assert_eq!(&output, "\"Hello World\"");
    }

    #[test]
    fn valid_print_rec_tuple() {
        let mut statement = Statement::parse(
            r##"
            print((420,(69,27),true));
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "(420,(69,27),true)");
        // assert_eq!(&output, "\"Hello World\"");
    }

    #[test]
    fn valid_print_slice() {
        let mut statement = Statement::parse(
            r##"
            print([5,7,8,9,10]);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "[5,7,8,9,10]");
        // assert_eq!(&output, "\"Hello World\"");
    }
    #[test]
    fn valid_print_rec_slice() {
        let mut statement = Statement::parse(
            r##"
            print([[2,4],[1,3]]);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "[[2,4],[1,3]]");
        // assert_eq!(&output, "\"Hello World\"");
    }
    #[test]
    fn valid_print_rec_slice_complex() {
        let mut statement = Statement::parse(
            r##"
            print([[[1,2],[3,4]],[[5,6],[7,8]],[[9,10],[11,12]]]);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "[[[1,2],[3,4]],[[5,6],[7,8]],[[9,10],[11,12]]]");
        // assert_eq!(&output, "\"Hello World\"");
    }

    #[test]
    fn valid_print_vec() {
        let mut statement = Statement::parse(
            r##"
            print(vec[1,2,3,4]);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "[1,2,3,4]");
        // assert_eq!(&output, "\"Hello World\"");
    }

    #[test]
    fn valid_print_vec_complex() {
        let mut statement = Statement::parse(
            r##"
            print(vec[string("Hello"),string(" "),string("world")]);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, r##"["Hello"," ","world"]"##);
        // assert_eq!(&output, "\"Hello World\"");
    }

    #[test]
    fn valid_print_addr() {
        let mut statement = Statement::parse(
            r##"
            {
                let x = 420; // 0x20
                let y = 420; // 0x28
                let z = 420; // 0x30
                print(&y);
            }
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "0x28");
        // assert_eq!(&output, "\"Hello World\"");
    }

    #[test]
    fn valid_print_struct() {
        let mut statement = Statement::parse(
            r##"
            {
                struct Point {
                    x : u64,
                    y : u64
                }
                let point = Point {
                    x:420,
                    y:69,
                };
                print(point);
            }
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "Point{x:420,y:69}");
        // assert_eq!(&output, "\"Hello World\"");
    }

    #[test]
    fn valid_print_enum() {
        let mut statement = Statement::parse(
            r##"
            {
                enum Color {
                    RED,
                    YELLOW,
                    BLUE,
                }
                let color = Color::YELLOW;
                print(color);
            }
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "Color::YELLOW");
        // assert_eq!(&output, "\"Hello World\"");
    }

    #[test]
    fn valid_print_union() {
        let mut statement = Statement::parse(
            r##"
            {
                union Geo {
                    Point {
                        x: u64,
                        y: u64,
                    },
                    Axe {
                        x : i64,
                    }
                }
                let geo = Geo::Point {
                    x : 420,
                    y : 69,
                };
                print(geo);
            }
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let output = compile_statement_for_stdout!(statement);
        assert_eq!(&output, "Geo::Point{x:420,y:69}");
        // assert_eq!(&output, "\"Hello World\"");
    }

    #[test]
    fn valid_scan() {
        let mut engine = StdinTestGameEngine {
            out: String::new(),
            in_buf: String::new(),
        };
        let mut ciphel = Ciphel::new();
        let tid = ciphel
            .start_arena(&mut engine)
            .expect("starting should not fail");

        let src = r##"
        
        let res = scan();
        println(res);
        
        "##;

        ciphel
            .compile::<StdinTestGameEngine>(crate::vm::vm::Player::P1, tid, src)
            .expect("Compilation should have succeeded");
        ciphel.run(&mut engine).expect("no error should arise");
        ciphel.run(&mut engine).expect("no error should arise");
        engine.in_buf = "Hello World".to_string().into();
        ciphel.run(&mut engine).expect("no error should arise");

        let output = engine.out;
        assert_eq!(&output, "Hello World\n")
    }
}
