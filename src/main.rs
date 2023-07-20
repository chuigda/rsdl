mod preprocess;
mod parser;

use std::env;

use pest::Parser;
use tracing::error;

use crate::parser::pest_parser::{PestRSDLParser, Rule};
use crate::parser::treeconv;
use crate::preprocess::preprocess;

fn main() {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    };

    tracing_subscriber::fmt::init();

    let args = env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        error!("请指定要处理的文件");
        return;
    }

    let file_name = &args[1];
    let Ok(file_content) = std::fs::read_to_string(file_name) else {
        error!("无法读取文件 {}", file_name);
        return;
    };

    let preprocessed = preprocess(file_name, &file_content);
    let rsdl = match PestRSDLParser::parse(Rule::rsdl_program, &preprocessed.output_src) {
        Ok(rsdl) => rsdl,
        Err(e) => {
            error!("解析 RSDL 文件 {file_name} 失败:\n{e}");
            return;
        }
    };

    dbg!(treeconv(rsdl));
}
