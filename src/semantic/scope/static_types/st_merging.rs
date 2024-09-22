use std::cmp::max;

use crate::{
    e_static,
    semantic::{EType, MergeType, SemanticError, TypeOf},
};

use super::{
    AddrType, ClosureType, FnType, MapType, PrimitiveType, RangeType, SliceType, StaticType,
    StrSliceType, StringType, TupleType, VecType,
};
use crate::semantic::scope::scope::ScopeManager;
