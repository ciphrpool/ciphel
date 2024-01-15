#[derive(Debug, Clone)]
pub enum Operation {
    HighOrdMath(HighOrdMath),
    LowOrdMath(LowOrdMath),
    Shift(Shift),
    BitwiseAnd(BitwiseAnd),
    BitwiseXOR(BitwiseXOR),
    BitwiseOR(BitwiseOR),
    Cast(Cast),
    Comparaison(Comparaison),
    Equation(Equation),
    Inclusion(Inclusion),
    LogicalAnd(LogicalAnd),
    LogicalOr(LogicalOr),
    Minus(Minus),
    Not(Not),
}

#[derive(Debug, Clone)]
pub enum HighOrdMath {}

#[derive(Debug, Clone)]
pub enum LowOrdMath {}

#[derive(Debug, Clone)]
pub enum Shift {}

#[derive(Debug, Clone)]
pub enum BitwiseAnd {}

#[derive(Debug, Clone)]
pub enum BitwiseXOR {}

#[derive(Debug, Clone)]
pub enum BitwiseOR {}

#[derive(Debug, Clone)]
pub enum Cast {}

#[derive(Debug, Clone)]
pub enum Comparaison {}

#[derive(Debug, Clone)]
pub enum Equation {}

#[derive(Debug, Clone)]
pub enum Inclusion {}

#[derive(Debug, Clone)]
pub enum LogicalAnd {}

#[derive(Debug, Clone)]
pub enum LogicalOr {}

#[derive(Debug, Clone)]
pub enum Minus {}

#[derive(Debug, Clone)]
pub enum Not {}
