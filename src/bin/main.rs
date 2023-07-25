use std::collections::{HashSet, VecDeque};
use std::env;
use std::path::PathBuf;

use pest::Parser;
use structopt::StructOpt;
use tracing::{error, info, warn};

use rsdl_core::codegen::codegen;
use rsdl_core::codegen::rustgen::RustGenerator;
use rsdl_core::min_resolv::ResolveContext;
use rsdl_core::parser::pest_parser::{PestRSDLParser, Rule};
use rsdl_core::parser::treeconv::treeconv;
use rsdl_core::preprocess::preprocess;

#[derive(Debug, StructOpt)]
#[structopt(name = "rsdl", about = "RSDL 优化编译器")]
struct Options {
    // -i, --input FILENAME
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,

    // -o, --output FILENAME
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    // -t, --mode MODE
    #[structopt(short = "t", long = "mode")]
    mode: String,

    // --namespace NAMESPACE
    #[structopt(long)]
    namespace: Option<String>,

    // --stdlib STDLIB
    #[structopt(long, parse(from_os_str))]
    stdlib: Option<PathBuf>
}

fn main() {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    };

    tracing_subscriber::fmt::init();

    let opt = Options::from_args();

    if opt.mode != "rust" {
        error!("不支持的输出模式 {}", opt.mode);
        return;
    }

    let mut tydes = Vec::new();
    if let Some(stdlib) = opt.stdlib {
        let display_name = format!("{}", stdlib.display());

        let Ok(content) = std::fs::read_to_string(&stdlib) else {
            error!("无法打开指定的 stdlib 文件 {display_name}");
            return;
        };

        let rsdl = match PestRSDLParser::parse(Rule::rsdl_program, &content) {
            Ok(rsdl) => rsdl,
            Err(e) => {
                error!("解析 stdlib 文件 {display_name} 失败:\n{e}");
                return;
            }
        };

        treeconv(&display_name, rsdl, &mut tydes);
    } else {
        let rsdl = PestRSDLParser::parse(Rule::rsdl_program, include_str!("../stdlib.rsdl")).unwrap();
        treeconv("(stdlib)", rsdl, &mut tydes);
    }

    let mut preprocessed_files = HashSet::new();
    let mut preprocess_queue: VecDeque<PathBuf> = VecDeque::new();
    let mut parse_stack: Vec<(PathBuf, String)> = Vec::new();

    let path = opt.input.canonicalize().unwrap();
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

    let mut generator = RustGenerator();
    if let Ok(output) = codegen(
        opt.namespace.as_ref().map(|s| s.as_str()),
        &tydes,
        &resolve_ctx,
        &mut generator
    ) {
        let output = output.join("\n");
        
        if let Err(e) = std::fs::write(&opt.output, output) {
            error!("无法写入输出文件 {}: {}", opt.output.display(), e);
        }
    }
}