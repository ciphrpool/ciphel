use std::f64::consts::E;
use std::f64::consts::PI;
use std::f64::INFINITY;
use std::f64::NEG_INFINITY;

use num_traits::ToBytes;

use crate::semantic::scope::static_types::{NumberType, PrimitiveType, StaticType};
use crate::semantic::ResolveCore;
use crate::semantic::{EType, TypeOf};
use crate::vm::asm::operation::OpPrimitive;
use crate::vm::asm::Asm;
use crate::vm::core::lexem;

use crate::vm::runtime::RuntimeError;
use crate::vm::scheduler::Executable;
use crate::vm::stdio::StdIO;
use crate::vm::GenerateCode;
use crate::{
    ast::expressions::Expression,
    semantic::{Resolve, SemanticError},
};
use crate::{e_static, p_num};

use super::PathFinder;
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
pub enum MathAsm {
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

impl PathFinder for MathFn {
    fn find(path: &[String], name: &str) -> Option<Self>
    where
        Self: Sized,
    {
        if (path.len() == 1 && path[0] == lexem::MATH) || path.len() == 0 {
            return match name {
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
            };
        }
        None
    }
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for MathAsm {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            MathAsm::Ceil => stdio.push_asm_lib(engine, "ceil"),
            MathAsm::Floor => stdio.push_asm_lib(engine, "floor"),
            MathAsm::Abs => stdio.push_asm_lib(engine, "abs"),
            MathAsm::Exp => stdio.push_asm_lib(engine, "exp"),
            MathAsm::Ln => stdio.push_asm_lib(engine, "ln"),
            MathAsm::Log => stdio.push_asm_lib(engine, "log"),
            MathAsm::Log10 => stdio.push_asm_lib(engine, "log10"),
            MathAsm::Pow => stdio.push_asm_lib(engine, "pow"),
            MathAsm::Sqrt => stdio.push_asm_lib(engine, "sqrt"),
            MathAsm::Acos => stdio.push_asm_lib(engine, "acos"),
            MathAsm::Asin => stdio.push_asm_lib(engine, "asin"),
            MathAsm::Atan => stdio.push_asm_lib(engine, "atan"),
            MathAsm::Atan2 => stdio.push_asm_lib(engine, "atan2"),
            MathAsm::Cos => stdio.push_asm_lib(engine, "cos"),
            MathAsm::Sin => stdio.push_asm_lib(engine, "sin"),
            MathAsm::Tan => stdio.push_asm_lib(engine, "tan"),
            MathAsm::Hypot => stdio.push_asm_lib(engine, "hypot"),
            MathAsm::Deg => stdio.push_asm_lib(engine, "deg"),
            MathAsm::Rad => stdio.push_asm_lib(engine, "rad"),
            MathAsm::CosH => stdio.push_asm_lib(engine, "cosh"),
            MathAsm::SinH => stdio.push_asm_lib(engine, "sinh"),
            MathAsm::TanH => stdio.push_asm_lib(engine, "tanh"),
            MathAsm::ACosH => stdio.push_asm_lib(engine, "acosh"),
            MathAsm::ASinH => stdio.push_asm_lib(engine, "asinh"),
            MathAsm::ATanH => stdio.push_asm_lib(engine, "atanh"),
            MathAsm::Pi => stdio.push_asm_lib(engine, "pi"),
            MathAsm::E => stdio.push_asm_lib(engine, "e"),
            MathAsm::Inf => stdio.push_asm_lib(engine, "inf"),
            MathAsm::NInf => stdio.push_asm_lib(engine, "ninf"),
            MathAsm::IsNaN => stdio.push_asm_lib(engine, "isnan"),
            MathAsm::IsInf => stdio.push_asm_lib(engine, "isinf"),
        }
    }
}

impl crate::vm::AsmWeight for MathAsm {
    fn weight(&self) -> crate::vm::Weight {
        crate::vm::Weight::EXTREME
    }
}

impl ResolveCore for MathFn {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        match self {
            MathFn::Pi | MathFn::E | MathFn::Inf | MathFn::NInf => {
                if parameters.len() != 0 {
                    return Err(SemanticError::IncorrectArguments);
                }

                Ok(p_num!(F64))
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
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let n = &mut parameters[0];
                let _ = n.resolve::<E>(scope_manager, scope_id, &Some(p_num!(F64)), &mut None)?;
                let n_type = n.type_of(&scope_manager, scope_id)?;

                match &n_type {
                    EType::Static(value) => match value {
                        &StaticType::Primitive(PrimitiveType::Number(NumberType::F64)) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                match self {
                    MathFn::IsNaN | MathFn::IsInf => {
                        Ok(e_static!(StaticType::Primitive(PrimitiveType::Bool)))
                    }
                    _ => Ok(p_num!(F64)),
                }
            }

            MathFn::Atan2 | MathFn::Hypot | MathFn::Log | MathFn::Pow => {
                if parameters.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, second_part) = parameters.split_at_mut(1);
                let x = &mut first_part[0];
                let y = &mut second_part[0];

                let _ = x.resolve::<E>(scope_manager, scope_id, &Some(p_num!(F64)), &mut None)?;
                let _ = y.resolve::<E>(scope_manager, scope_id, &Some(p_num!(F64)), &mut None)?;

                let x_type = x.type_of(&scope_manager, scope_id)?;
                let y_type = y.type_of(&scope_manager, scope_id)?;

                match &x_type {
                    EType::Static(value) => match value {
                        &StaticType::Primitive(PrimitiveType::Number(NumberType::F64)) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                match &y_type {
                    EType::Static(value) => match value {
                        &StaticType::Primitive(PrimitiveType::Number(NumberType::F64)) => {}
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                Ok(p_num!(F64))
            }
        }
    }
}

impl GenerateCode for MathFn {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            MathFn::Ceil => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Ceil)))),

            MathFn::Floor => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Floor)))),
            MathFn::Abs => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Abs)))),

            MathFn::Exp => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Exp)))),

            MathFn::Log => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Log)))),

            MathFn::Ln => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Ln)))),

            MathFn::Log10 => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Log10)))),
            MathFn::Pow => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Pow)))),

            MathFn::Sqrt => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Sqrt)))),

            MathFn::Acos => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Acos)))),

            MathFn::Asin => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Asin)))),

            MathFn::Atan => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Atan)))),

            MathFn::Atan2 => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Atan2)))),
            MathFn::Cos => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Cos)))),

            MathFn::Sin => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Sin)))),

            MathFn::Tan => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Tan)))),

            MathFn::Hypot => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Hypot)))),
            MathFn::Deg => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Deg)))),

            MathFn::Rad => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Rad)))),

            MathFn::CosH => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::CosH)))),

            MathFn::SinH => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::SinH)))),

            MathFn::TanH => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::TanH)))),

            MathFn::ACosH => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::ACosH)))),
            MathFn::ASinH => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::ASinH)))),
            MathFn::ATanH => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::ATanH)))),
            MathFn::Pi => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Pi)))),

            MathFn::E => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::E)))),
            MathFn::Inf => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::Inf)))),

            MathFn::NInf => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::NInf)))),

            MathFn::IsNaN => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::IsNaN)))),
            MathFn::IsInf => Ok(instructions.push(Asm::Core(super::CoreAsm::Math(MathAsm::IsInf)))),
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for MathAsm {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match self {
            MathAsm::Pi => {
                let _ = stack.push_with(&PI.to_le_bytes())?;
            }
            MathAsm::E => {
                let _ = stack.push_with(&E.to_le_bytes())?;
            }
            MathAsm::Inf => {
                let _ = stack.push_with(&INFINITY.to_le_bytes())?;
            }
            MathAsm::NInf => {
                let _ = stack.push_with(&NEG_INFINITY.to_le_bytes())?;
            }
            MathAsm::IsNaN => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::is_nan(n);

                let _ = stack.push_with(&[res as u8])?;
            }
            MathAsm::IsInf => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::is_infinite(n);

                let _ = stack.push_with(&[res as u8])?;
            }

            MathAsm::Ceil => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::ceil(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Floor => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::floor(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Abs => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::abs(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Exp => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::exp(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Ln => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::ln(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Log10 => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::log10(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Sqrt => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::sqrt(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Acos => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::acos(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Asin => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::asin(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Atan => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::atan(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Cos => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::cos(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Sin => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::sin(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Tan => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::tan(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Deg => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::to_degrees(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Rad => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::to_radians(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::CosH => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::cosh(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::SinH => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::sinh(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::TanH => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::tanh(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::ACosH => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::acosh(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::ASinH => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::asinh(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::ATanH => {
                let n = OpPrimitive::pop_float(stack)?;

                let res = f64::atanh(n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }

            MathAsm::Pow => {
                let n = OpPrimitive::pop_float(stack)?;
                let x = OpPrimitive::pop_float(stack)?;

                let res = f64::powf(x, n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Log => {
                let n = OpPrimitive::pop_float(stack)?;
                let x = OpPrimitive::pop_float(stack)?;

                let res = f64::log(x, n);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Hypot => {
                let y = OpPrimitive::pop_float(stack)?;
                let x = OpPrimitive::pop_float(stack)?;

                let res = f64::hypot(x, y);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
            MathAsm::Atan2 => {
                let x = OpPrimitive::pop_float(stack)?;
                let y = OpPrimitive::pop_float(stack)?;

                let res = f64::atan2(y, x);

                let _ = stack.push_with(&res.to_le_bytes())?;
            }
        }
        scheduler.next();
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn valid_nan() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = is_nan(acos(2.0));
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     compile_statement!(statement);

    //     assert_eq!(result, Primitive::Bool(true))
    // }

    // #[test]
    // fn robustness_nan() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = is_nan(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&PrimitiveType::Bool, &data)
    //             .expect("Deserialization should have succeeded");
    //     assert_eq!(result, Primitive::Bool(false))
    // }
    // #[test]
    // fn valid_ceil() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = ceil(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::ceil(2.0)))
    // }
    // #[test]
    // fn valid_floor() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = floor(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::floor(2.0)))
    // }
    // #[test]
    // fn valid_abs() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = abs(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::abs(2.0)))
    // }
    // #[test]
    // fn valid_exp() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = exp(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::exp(2.0)))
    // }
    // #[test]
    // fn valid_ln() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = ln(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::ln(2.0)))
    // }
    // #[test]
    // fn valid_log() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = log(2.0,5.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::log(2.0, 5.0)))
    // }
    // #[test]
    // fn valid_log10() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = log10(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::log10(2.0)))
    // }
    // #[test]
    // fn valid_pow() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = pow(2.0,2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::powf(2.0, 2.0)))
    // }
    // #[test]
    // fn valid_sqrt() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = sqrt(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::sqrt(2.0)))
    // }
    // #[test]
    // fn valid_acos() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = acos(0.5);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::acos(0.5)))
    // }
    // #[test]
    // fn valid_asin() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = asin(0.5);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::asin(0.5)))
    // }
    // #[test]
    // fn valid_atan() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = atan(0.5);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::atan(0.5)))
    // }
    // #[test]
    // fn valid_atan2() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = atan2(2.0,2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::atan2(2.0, 2.0)))
    // }
    // #[test]
    // fn valid_cos() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = cos(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::cos(2.0)))
    // }
    // #[test]
    // fn valid_sin() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = sin(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::sin(2.0)))
    // }
    // #[test]
    // fn valid_tan() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = tan(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::tan(2.0)))
    // }
    // #[test]
    // fn valid_hypot() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = hypot(2.0,2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::hypot(2.0, 2.0)))
    // }
    // #[test]
    // fn valid_deg() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = deg(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::to_degrees(2.0)))
    // }
    // #[test]
    // fn valid_rad() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = rad(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::to_radians(2.0)))
    // }
    // #[test]
    // fn valid_cosh() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = cosh(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::cosh(2.0)))
    // }
    // #[test]
    // fn valid_sinh() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = sinh(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::sinh(2.0)))
    // }
    // #[test]
    // fn valid_tanh() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = tanh(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::tanh(2.0)))
    // }
    // #[test]
    // fn valid_acosh() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = acosh(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::acosh(2.0)))
    // }
    // #[test]
    // fn valid_asinh() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = asinh(2.0);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::asinh(2.0)))
    // }
    // #[test]
    // fn valid_atanh() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = atanh(0.5);
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, f64::atanh(0.5)))
    // }
    // #[test]
    // fn valid_pi() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = pi();
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, PI))
    // }
    // #[test]
    // fn valid_e() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = e();
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(F64, E))
    // }

    // #[test]
    // fn valid_inf() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = inf();
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");

    //     assert_eq!(result, v_num!(F64, INFINITY))
    // }

    // #[test]
    // fn valid_ninf() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = neg_inf();
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::F64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");

    //     assert_eq!(result, v_num!(F64, NEG_INFINITY))
    // }

    // #[test]
    // fn valid_is_inf() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         let res = is_inf(inf());
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;
    //     let data = compile_statement!(statement);
    //     let result =
    //         <PrimitiveType as DeserializeFrom>::deserialize_from(&&PrimitiveType::Bool, &data)
    //             .expect("Deserialization should have succeeded");

    //     assert_eq!(result, Primitive::Bool(true))
    // }
}
