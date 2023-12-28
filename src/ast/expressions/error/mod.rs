pub mod error_parse;
pub mod error_resolve;
pub mod error_typeof;

#[derive(Debug, Clone, PartialEq)]
pub struct Error(usize);
