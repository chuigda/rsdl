//! 使用 pest 实现的解析器，将 RSDL 代码解析为 Pest AST

use pest_derive::Parser;

/// 使用 pest 实现的解析器，将 RSDL 代码解析为 Pest AST
///
/// 一般而言，下游不应该直接使用此结构。请参见 [`crate::driver::application_start`]
#[derive(Parser)]
#[grammar = "parser/rsdl.pest"]
pub struct PestRSDLParser;
