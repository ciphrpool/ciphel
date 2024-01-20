pub const ENUM: &str = "enum";
pub const STRUCT: &str = "struct";
pub const UNION: &str = "union";
pub const AS: &str = "as";
pub const ARROW: &str = "->";
pub const BIGARROW: &str = "=>";
pub const ELSE: &str = "else";
pub const TRY: &str = "try";
pub const IF: &str = "if";
pub const THEN: &str = "then";
pub const MATCH: &str = "match";
pub const CASE: &str = "case";
pub const RETURN: &str = "return";
pub const LET: &str = "let";
pub const EVENT: &str = "event";
pub const WHILE: &str = "while";
pub const FOR: &str = "for";
pub const LOOP: &str = "loop";
// TYPE
pub const U8: &str = "u8";
pub const U16: &str = "u16";
pub const U32: &str = "u32";
pub const U64: &str = "u64";
pub const U128: &str = "u128";
pub const I8: &str = "i8";
pub const I16: &str = "i16";
pub const I32: &str = "i32";
pub const I64: &str = "i64";
pub const I128: &str = "i128";

pub const FLOAT: &str = "float";
pub const CHAR: &str = "char";
pub const STRING: &str = "string";
pub const BOOL: &str = "bool";
pub const UNIT: &str = "unit";
pub const UUNIT: &str = "Unit";
pub const UVEC: &str = "Vec";
pub const UMAP: &str = "Map";
pub const UCHAN: &str = "Chan";
pub const FN: &str = "fn";

// PONCTUATION
pub const COMA: &str = ",";
pub const DOT: &str = ".";
pub const SEMI_COLON: &str = ";";
pub const BRA_O: &str = "{";
pub const BRA_C: &str = "}";
pub const SQ_BRA_O: &str = "[";
pub const SQ_BRA_C: &str = "]";
pub const PAR_O: &str = "(";
pub const PAR_C: &str = ")";
pub const ANNOTATION: &str = "@";
pub const HASHTAG: &str = "#";
pub const SL_COMMENT: &str = "//";
pub const ML_OP_COMMENT: &str = "/*";
pub const ML_CL_COMMENT: &str = "*/";
pub const EQUAL: &str = "=";
pub const TRUE: &str = "true";
pub const FALSE: &str = "false";
pub const COLON: &str = ":";
pub const ADDR: &str = "&";
pub const SEP: &str = "::";
pub const ACCESS: &str = "*";
pub const BAR: &str = "|";

// OPERATOR
pub const LESSER: &str = "<";
pub const GREATER: &str = ">";
pub const BOR: &str = "|";
pub const BAND: &str = "&";
pub const MINUS: &str = "-";
pub const NEGATION: &str = "!";
pub const ADD: &str = "+";
pub const MULT: &str = "*";
pub const DIV: &str = "/";
pub const MOD: &str = "%";
pub const SHL: &str = "<<";
pub const SHR: &str = ">>";
pub const OR: &str = "or";
pub const AND: &str = "and";
pub const XOR: &str = "^";
pub const IN: &str = "in";
pub const LE: &str = "<";
pub const ELE: &str = "<=";
pub const GE: &str = ">";
pub const EGE: &str = ">=";
pub const EQ: &str = "==";
pub const NEQ: &str = "!=";

pub mod platform {
    pub const RECEIVE: &str = "receive";
    pub const SEND: &str = "send";
    pub const VEC: &str = "vec";
    pub const MAP: &str = "map";
    pub const CHAN: &str = "chan";
    pub const ERROR: &str = "error";
}
