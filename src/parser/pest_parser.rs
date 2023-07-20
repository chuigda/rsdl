use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/rsdl.pest"]
pub struct PestRSDLParser;
