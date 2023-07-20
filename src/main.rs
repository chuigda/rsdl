mod min_resolv;
mod parser;
mod preprocess;

use std::collections::{HashSet, VecDeque};
use std::env;
use std::path::{Path, PathBuf};

use pest::Parser;
use tracing::{error, info, warn};

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

    let mut resolved_files = HashSet::new();
    let mut queue: VecDeque<PathBuf> = VecDeque::new();
    let mut typedef = Vec::new();

    let path = Path::new(&args[1]).canonicalize().unwrap();
    let workdir = path.parent().unwrap().canonicalize().unwrap();

    queue.push_back(path);

    while !queue.is_empty() {
        let path = queue.pop_front().unwrap();
        let display_name = format!("{}", path.display());

        #[cfg(windows)] let display_name = display_name.replace("\\\\?\\", "");

        info!("正在处理 {display_name}");

        if resolved_files.contains(&path) {
            warn!("文件 {} 已经被处理过，跳过", path.display());
            continue;
        }

        let Ok(file_content) = std::fs::read_to_string(&path) else {
            error!("无法读取文件 {display_name}");
            return;
        };

        let preprocessed = preprocess(&display_name, &file_content);
        let rsdl = match PestRSDLParser::parse(Rule::rsdl_program, &preprocessed.output_src) {
            Ok(rsdl) => rsdl,
            Err(e) => {
                error!("解析 RSDL 文件 {display_name} 失败:\n{e}");
                return;
            }
        };

        treeconv(&display_name, rsdl, &mut typedef);
        resolved_files.insert(path);

        for include in preprocessed.includes {
            let Ok(include_path) = workdir.join(&include).canonicalize() else {
                error!("无法解析引用的文件 {include}");
                return;
            };
            queue.push_back(include_path);
        }
    }

    dbg!(typedef);
}
