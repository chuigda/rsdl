use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "rsdl.pest"]
pub struct RSDLParser;
