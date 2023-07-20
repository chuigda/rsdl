mod min_resolv;
mod parser;
mod preprocess;
mod codegen;

use std::collections::{HashSet, VecDeque};
use std::env;
use std::path::{Path, PathBuf};

use pest::Parser;
use tracing::{error, info, warn};
use crate::min_resolv::ResolveContext;

use crate::parser::pest_parser::{PestRSDLParser, Rule};
use crate::parser::treeconv::treeconv;
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

    let mut preprocessed_files = HashSet::new();
    let mut preprocess_queue: VecDeque<PathBuf> = VecDeque::new();
    let mut parse_stack: Vec<(PathBuf, String)> = Vec::new();
    let mut tydes = Vec::new();

    let path = Path::new(&args[1]).canonicalize().unwrap();
    let workdir = path.parent().unwrap().canonicalize().unwrap();

    preprocess_queue.push_back(path);

    while !preprocess_queue.is_empty() {
        let path = preprocess_queue.pop_front().unwrap();
        let display_name = format!("{}", path.display());
        #[cfg(windows)] let display_name = display_name.replace("\\\\?\\", "");

        info!("预处理 {display_name}");

        if preprocessed_files.contains(&path) {
            warn!("文件 {} 已经被处理过，跳过", path.display());
            continue;
        }

        let Ok(file_content) = std::fs::read_to_string(&path) else {
            error!("无法读取文件 {display_name}");
            return;
        };

        let preprocessed = preprocess(&display_name, &file_content);
        parse_stack.push((path.clone(), preprocessed.output_src));
        preprocessed_files.insert(path.clone());

        for include in preprocessed.includes.into_iter().rev() {
            let Ok(include_path) = workdir.join(&include).canonicalize() else {
                error!("无法解析引用的文件 {include}");
                return;
            };
            preprocess_queue.push_back(include_path);
        }
    }

    for (path, src) in parse_stack.into_iter().rev() {
        let display_name = format!("{}", path.display());
        #[cfg(windows)] let display_name = display_name.replace("\\\\?\\", "");

        info!("解析 {display_name}");

        let rsdl = match PestRSDLParser::parse(Rule::rsdl_program, &src) {
            Ok(rsdl) => rsdl,
            Err(e) => {
                error!("解析 RSDL 文件 {display_name} 失败:\n{e}");
                return;
            }
        };

        treeconv(&display_name, rsdl, &mut tydes);
    }

    let mut resolve_ctx = ResolveContext::new();

    for tyde in tydes.iter() {
        if resolve_ctx.min_resolv(tyde).is_err() {
            return;
        }
    }

    for tyde in tydes.iter() {
        if resolve_ctx.min_resolv_chk(tyde).is_err() {
            return;
        }
    }
}
