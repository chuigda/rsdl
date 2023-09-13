//! 使用 pest 实现的解析器，将 RSDL 代码解析为 Pest AST

use pest_derive::Parser;

/// 使用 pest 实现的解析器，将 RSDL 代码解析为 Pest AST
#[derive(Parser)]
#[grammar = "parser/rsdl.pest"]
pub struct PestRSDLParser;
