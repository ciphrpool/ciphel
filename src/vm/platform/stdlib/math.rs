use std::arch::x86_64;
use std::cell::Ref;
use std::f64::consts::E;
use std::f64::consts::PI;
use std::f64::INFINITY;
use std::f64::NEG_INFINITY;

use num_traits::ToBytes;

use crate::ast::utils::strings::ID;
use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::static_types::{NumberType, PrimitiveType, StaticType};
use crate::semantic::{Either, TypeOf};
use crate::vm::allocator::heap::Heap;
use crate::vm::allocator::stack::Stack;
use crate::vm::casm::operation::OpPrimitive;
use crate::vm::casm::Casm;
use crate::vm::platform::utils::lexem;
use crate::vm::platform::LibCasm;

use crate::vm::stdio::StdIO;
use crate::vm::vm::CasmMetadata;
use crate::vm::vm::{Executable, RuntimeError};
use crate::{
    ast::expressions::Expression,
    semantic::{ArcMutex, EType, Resolve, SemanticError},
    vm::{
        casm::CasmProgram,
        vm::{CodeGenerationError, GenerateCode},
    },
};
use crate::{e_static, p_num};
#[derive(Debug, Clone, PartialEq)]
pub enum MathFn {
    Ceil,
    Floor,
    Abs,
    Exp,
    Ln,
    Log,
    Log10,
    Pow,
    Sqrt,
    Acos,
    Asin,
    Atan,
    Atan2,
    Cos,
    Sin,
    Tan,
    Hypot,
    Deg,
    Rad,
    CosH,
    SinH,
    TanH,
    ACosH,
    ASinH,
    ATanH,
    Pi,
    E,
    Inf,
    NInf,
    IsNaN,
    IsInf,
}
#[derive(Debug, Clone, PartialEq)]
pub enum MathCasm {
    Ceil,
    Floor,
    Abs,
    Exp,
    Ln,
    Log,
    Log10,
    Pow,
    Sqrt,
    Acos,
    Asin,
    Atan,
    Atan2,
    Cos,
    Sin,
    Tan,
    Hypot,
    Deg,
    Rad,
    CosH,
    SinH,
    TanH,
    ACosH,
    ASinH,
    ATanH,
    Pi,
    E,
    Inf,
    NInf,
    IsNaN,
    IsInf,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for MathCasm {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram, engine: &mut G) {
        match self {
            MathCasm::Ceil => stdio.push_casm_lib(engine, "ceil"),
            MathCasm::Floor => stdio.push_casm_lib(engine, "floor"),
            MathCasm::Abs => stdio.push_casm_lib(engine, "abs"),
            MathCasm::Exp => stdio.push_casm_lib(engine, "exp"),
            MathCasm::Ln => stdio.push_casm_lib(engine, "ln"),
            MathCasm::Log => stdio.push_casm_lib(engine, "log"),
            MathCasm::Log10 => stdio.push_casm_lib(engine, "log10"),
            MathCasm::Pow => stdio.push_casm_lib(engine, "pow"),
            MathCasm::Sqrt => stdio.push_casm_lib(engine, "sqrt"),
            MathCasm::Acos => stdio.push_casm_lib(engine, "acos"),
            MathCasm::Asin => stdio.push_casm_lib(engine, "asin"),
            MathCasm::Atan => stdio.push_casm_lib(engine, "atan"),
            MathCasm::Atan2 => stdio.push_casm_lib(engine, "atan2"),
            MathCasm::Cos => stdio.push_casm_lib(engine, "cos"),
            MathCasm::Sin => stdio.push_casm_lib(engine, "sin"),
            MathCasm::Tan => stdio.push_casm_lib(engine, "tan"),
            MathCasm::Hypot => stdio.push_casm_lib(engine, "hypot"),
            MathCasm::Deg => stdio.push_casm_lib(engine, "deg"),
            MathCasm::Rad => stdio.push_casm_lib(engine, "rad"),
            MathCasm::CosH => stdio.push_casm_lib(engine, "cosh"),
            MathCasm::SinH => stdio.push_casm_lib(engine, "sinh"),
            MathCasm::TanH => stdio.push_casm_lib(engine, "tanh"),
            MathCasm::ACosH => stdio.push_casm_lib(engine, "acosh"),
            MathCasm::ASinH => stdio.push_casm_lib(engine, "asinh"),
            MathCasm::ATanH => stdio.push_casm_lib(engine, "atanh"),
            MathCasm::Pi => stdio.push_casm_lib(engine, "pi"),
            MathCasm::E => stdio.push_casm_lib(engine, "e"),
            MathCasm::Inf => stdio.push_casm_lib(engine, "inf"),
            MathCasm::NInf => stdio.push_casm_lib(engine, "ninf"),
            MathCasm::IsNaN => stdio.push_casm_lib(engine, "isnan"),
            MathCasm::IsInf => stdio.push_casm_lib(engine, "isinf"),
        }
    }
}

impl MathFn {
    pub fn from(suffixe: &Option<ID>, id: &ID) -> Option<Self> {
        match suffixe {
            Some(suffixe) => {
                if **suffixe != lexem::STD {
                    return None;
                }
            }
            None => {}
        }
        match id.as_str() {
            lexem::CEIL => Some(MathFn::Ceil),
            lexem::FLOOR => Some(MathFn::Floor),
            lexem::ABS => Some(MathFn::Abs),
            lexem::EXP => Some(MathFn::Exp),
            lexem::LN => Some(MathFn::Ln),
            lexem::LOG => Some(MathFn::Log),
            lexem::LOG10 => Some(MathFn::Log10),
            lexem::POW => Some(MathFn::Pow),
            lexem::SQRT => Some(MathFn::Sqrt),
            lexem::ACOS => Some(MathFn::Acos),
            lexem::ASIN => Some(MathFn::Asin),
            lexem::ATAN => Some(MathFn::Atan),
            lexem::ATAN2 => Some(MathFn::Atan2),
            lexem::COS => Some(MathFn::Cos),
            lexem::SIN => Some(MathFn::Sin),
            lexem::TAN => Some(MathFn::Tan),
            lexem::HYPOT => Some(MathFn::Hypot),
            lexem::DEG => Some(MathFn::Deg),
            lexem::RAD => Some(MathFn::Rad),
            lexem::COSH => Some(MathFn::CosH),
            lexem::SINH => Some(MathFn::SinH),
            lexem::TANH => Some(MathFn::TanH),
            lexem::ACOSH => Some(MathFn::ACosH),
            lexem::ASINH => Some(MathFn::ASinH),
            lexem::ATANH => Some(MathFn::ATanH),
            lexem::PI => Some(MathFn::Pi),
            lexem::E => Some(MathFn::E),
            lexem::INF => Some(MathFn::Inf),
            lexem::NEG_INF => Some(MathFn::NInf),
            lexem::IS_NAN => Some(MathFn::IsNaN),
            lexem::IS_INF => Some(MathFn::IsInf),
            _ => None,
        }
    }
}
impl Resolve for MathFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression>;
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        _context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            MathFn::Pi | MathFn::E | MathFn::Inf | MathFn::NInf => {
                if extra.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }

                Ok(())
            }
            MathFn::IsNaN
            | MathFn::IsInf
            | MathFn::Ceil
            | MathFn::Floor
            | MathFn::Abs
            | MathFn::Exp
            | MathFn::Ln
            | MathFn::Log10
            | MathFn::Sqrt
            | MathFn::Acos
            | MathFn::Asin
            | MathFn::Atan
            | MathFn::Cos
            | MathFn::Sin
            | MathFn::Tan
            | MathFn::Deg
            | MathFn::Rad
            | MathFn::CosH
            | MathFn::SinH
            | MathFn::TanH
            | MathFn::ACosH
            | MathFn::ASinH
            | MathFn::ATanH => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let n = &mut extra[0];
                let _ = n.resolve(scope, &Some(p_num!(F64)), &mut None)?;
                let n_type = n.type_of(&crate::arw_read!(scope,SemanticError::ConcurrencyError)?)?;

                match &n_type {
                    Either::Static(value) => match value.as_ref() {
                        &StaticType::Primitive(PrimitiveType::Number(NumberType::F64)) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                Ok(())
            }

            MathFn::Atan2 | MathFn::Hypot | MathFn::Log | MathFn::Pow => {
                if extra.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, second_part) = extra.split_at_mut(1);
                let x = &mut first_part[0];
                let y = &mut second_part[0];

                let _ = x.resolve(scope, &Some(p_num!(F64)), &mut None)?;
                let _ = y.resolve(scope, &Some(p_num!(F64)), &mut None)?;

                let x_type = x.type_of(&crate::arw_read!(scope,SemanticError::ConcurrencyError)?)?;
                let y_type = y.type_of(&crate::arw_read!(scope,SemanticError::ConcurrencyError)?)?;

                match &x_type {
                    Either::Static(value) => match value.as_ref() {
                        &StaticType::Primitive(PrimitiveType::Number(NumberType::F64)) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                match &y_type {
                    Either::Static(value) => match value.as_ref() {
                        &StaticType::Primitive(PrimitiveType::Number(NumberType::F64)) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                Ok(())
            }
        }
    }
}
impl TypeOf for MathFn {
    fn type_of(&self, _scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            MathFn::IsNaN | MathFn::IsInf => {
                Ok(e_static!(StaticType::Primitive(PrimitiveType::Bool)))
            }
            _ => Ok(p_num!(F64)),
        }
    }
}

impl GenerateCode for MathFn {
    fn gencode(
        &self,
        _scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            MathFn::Ceil => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Ceil),
            )))),
            MathFn::Floor => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Floor),
            )))),
            MathFn::Abs => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Abs),
            )))),
            MathFn::Exp => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Exp),
            )))),
            MathFn::Log => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Log),
            )))),
            MathFn::Ln => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Ln),
            )))),
            MathFn::Log10 => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Log10),
            )))),
            MathFn::Pow => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Pow),
            )))),
            MathFn::Sqrt => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Sqrt),
            )))),
            MathFn::Acos => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Acos),
            )))),
            MathFn::Asin => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Asin),
            )))),
            MathFn::Atan => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Atan),
            )))),
            MathFn::Atan2 => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Atan2),
            )))),
            MathFn::Cos => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Cos),
            )))),
            MathFn::Sin => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Sin),
            )))),
            MathFn::Tan => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Tan),
            )))),
            MathFn::Hypot => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Hypot),
            )))),
            MathFn::Deg => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Deg),
            )))),
            MathFn::Rad => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Rad),
            )))),
            MathFn::CosH => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::CosH),
            )))),
            MathFn::SinH => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::SinH),
            )))),
            MathFn::TanH => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::TanH),
            )))),
            MathFn::ACosH => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::ACosH),
            )))),
            MathFn::ASinH => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::ASinH),
            )))),
            MathFn::ATanH => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::ATanH),
            )))),
            MathFn::Pi => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Pi),
            )))),
            MathFn::E => Ok(
                instructions.push(Casm::Platform(LibCasm::Std(super::StdCasm::Math(
                    MathCasm::E,
                )))),
            ),
            MathFn::Inf => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::Inf),
            )))),
            MathFn::NInf => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::NInf),
            )))),
            MathFn::IsNaN => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::IsNaN),
            )))),
            MathFn::IsInf => Ok(instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Math(MathCasm::IsInf),
            )))),
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for MathCasm {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match self {
            MathCasm::Pi => {
                let _ = stack.push_with(&PI.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::E => {
                let _ = stack.push_with(&E.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Inf => {
                let _ = stack
                    .push_with(&INFINITY.to_le_bytes())
                    .map_err(|e| e.into())?;
            }
            MathCasm::NInf => {
                let _ = stack
                    .push_with(&NEG_INFINITY.to_le_bytes())
                    .map_err(|e| e.into())?;
            }
            MathCasm::IsNaN => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::is_nan(n);

                let _ = stack.push_with(&[res as u8]).map_err(|e| e.into())?;
            }
            MathCasm::IsInf => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::is_infinite(n);

                let _ = stack.push_with(&[res as u8]).map_err(|e| e.into())?;
            }

            MathCasm::Ceil => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::ceil(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Floor => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::floor(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Abs => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::abs(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Exp => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::exp(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Ln => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::ln(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Log10 => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::log10(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Sqrt => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::sqrt(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Acos => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::acos(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Asin => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::asin(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Atan => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::atan(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Cos => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::cos(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Sin => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::sin(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Tan => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::tan(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Deg => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::to_degrees(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Rad => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::to_radians(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::CosH => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::cosh(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::SinH => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::sinh(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::TanH => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::tanh(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::ACosH => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::acosh(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::ASinH => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::asinh(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::ATanH => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::atanh(n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }

            MathCasm::Pow => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;
                let x = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::powf(x, n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Log => {
                let n = OpPrimitive::get_num8::<f64>(stack)?;
                let x = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::log(x, n);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Hypot => {
                let y = OpPrimitive::get_num8::<f64>(stack)?;
                let x = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::hypot(x, y);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
            MathCasm::Atan2 => {
                let x = OpPrimitive::get_num8::<f64>(stack)?;
                let y = OpPrimitive::get_num8::<f64>(stack)?;

                let res = f64::atan2(y, x);

                let _ = stack.push_with(&res.to_le_bytes()).map_err(|e| e.into())?;
            }
        }
        program.incr();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::Cell,
        f64::{
            consts::{E, PI},
            NEG_INFINITY,
        },
    };

    use crate::{
        ast::{
            expressions::data::{Number, Primitive},
            statements::Statement,
            TryParse,
        },
        clear_stack, compile_statement,
        semantic::scope::scope::Scope,
        v_num,
        vm::vm::{DeserializeFrom, Runtime},
    };

    use super::*;

    #[test]
    fn valid_nan() {
        let mut statement = Statement::parse(
            r##"
            let res = is_nan(acos(2.0));
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result =
            <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Bool, &data)
                .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Bool(true))
    }

    #[test]
    fn robustness_nan() {
        let mut statement = Statement::parse(
            r##"
            let res = is_nan(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result =
            <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Bool, &data)
                .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Bool(false))
    }
    #[test]
    fn valid_ceil() {
        let mut statement = Statement::parse(
            r##"
            let res = ceil(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::ceil(2.0)))
    }
    #[test]
    fn valid_floor() {
        let mut statement = Statement::parse(
            r##"
            let res = floor(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::floor(2.0)))
    }
    #[test]
    fn valid_abs() {
        let mut statement = Statement::parse(
            r##"
            let res = abs(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::abs(2.0)))
    }
    #[test]
    fn valid_exp() {
        let mut statement = Statement::parse(
            r##"
            let res = exp(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::exp(2.0)))
    }
    #[test]
    fn valid_ln() {
        let mut statement = Statement::parse(
            r##"
            let res = ln(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::ln(2.0)))
    }
    #[test]
    fn valid_log() {
        let mut statement = Statement::parse(
            r##"
            let res = log(2.0,5.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::log(2.0, 5.0)))
    }
    #[test]
    fn valid_log10() {
        let mut statement = Statement::parse(
            r##"
            let res = log10(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::log10(2.0)))
    }
    #[test]
    fn valid_pow() {
        let mut statement = Statement::parse(
            r##"
            let res = pow(2.0,2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::powf(2.0, 2.0)))
    }
    #[test]
    fn valid_sqrt() {
        let mut statement = Statement::parse(
            r##"
            let res = sqrt(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::sqrt(2.0)))
    }
    #[test]
    fn valid_acos() {
        let mut statement = Statement::parse(
            r##"
            let res = acos(0.5);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::acos(0.5)))
    }
    #[test]
    fn valid_asin() {
        let mut statement = Statement::parse(
            r##"
            let res = asin(0.5);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::asin(0.5)))
    }
    #[test]
    fn valid_atan() {
        let mut statement = Statement::parse(
            r##"
            let res = atan(0.5);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::atan(0.5)))
    }
    #[test]
    fn valid_atan2() {
        let mut statement = Statement::parse(
            r##"
            let res = atan2(2.0,2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::atan2(2.0, 2.0)))
    }
    #[test]
    fn valid_cos() {
        let mut statement = Statement::parse(
            r##"
            let res = cos(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::cos(2.0)))
    }
    #[test]
    fn valid_sin() {
        let mut statement = Statement::parse(
            r##"
            let res = sin(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::sin(2.0)))
    }
    #[test]
    fn valid_tan() {
        let mut statement = Statement::parse(
            r##"
            let res = tan(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::tan(2.0)))
    }
    #[test]
    fn valid_hypot() {
        let mut statement = Statement::parse(
            r##"
            let res = hypot(2.0,2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::hypot(2.0, 2.0)))
    }
    #[test]
    fn valid_deg() {
        let mut statement = Statement::parse(
            r##"
            let res = deg(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::to_degrees(2.0)))
    }
    #[test]
    fn valid_rad() {
        let mut statement = Statement::parse(
            r##"
            let res = rad(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::to_radians(2.0)))
    }
    #[test]
    fn valid_cosh() {
        let mut statement = Statement::parse(
            r##"
            let res = cosh(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::cosh(2.0)))
    }
    #[test]
    fn valid_sinh() {
        let mut statement = Statement::parse(
            r##"
            let res = sinh(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::sinh(2.0)))
    }
    #[test]
    fn valid_tanh() {
        let mut statement = Statement::parse(
            r##"
            let res = tanh(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::tanh(2.0)))
    }
    #[test]
    fn valid_acosh() {
        let mut statement = Statement::parse(
            r##"
            let res = acosh(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::acosh(2.0)))
    }
    #[test]
    fn valid_asinh() {
        let mut statement = Statement::parse(
            r##"
            let res = asinh(2.0);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::asinh(2.0)))
    }
    #[test]
    fn valid_atanh() {
        let mut statement = Statement::parse(
            r##"
            let res = atanh(0.5);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, f64::atanh(0.5)))
    }
    #[test]
    fn valid_pi() {
        let mut statement = Statement::parse(
            r##"
            let res = pi();
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, PI))
    }
    #[test]
    fn valid_e() {
        let mut statement = Statement::parse(
            r##"
            let res = e();
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(F64, E))
    }

    #[test]
    fn valid_inf() {
        let mut statement = Statement::parse(
            r##"
            let res = inf();
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(result, v_num!(F64, INFINITY))
    }

    #[test]
    fn valid_ninf() {
        let mut statement = Statement::parse(
            r##"
            let res = neg_inf();
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::F64),
            &data,
        )
        .expect("Deserialization should have succeeded");

        assert_eq!(result, v_num!(F64, NEG_INFINITY))
    }

    #[test]
    fn valid_is_inf() {
        let mut statement = Statement::parse(
            r##"
            let res = is_inf(inf());
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result =
            <PrimitiveType as DeserializeFrom>::deserialize_from(&&PrimitiveType::Bool, &data)
                .expect("Deserialization should have succeeded");

        assert_eq!(result, Primitive::Bool(true))
    }
}
