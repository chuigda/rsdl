//! `rsdl` 应用程序的实际入口点

use std::collections::{HashSet, VecDeque};
use std::env;
use std::path::PathBuf;

use pest::Parser;
use structopt::StructOpt;
use tracing::{error, info, warn};

use crate::codegen::{codegen, CodeGeneratorFactory};
use crate::min_resolv::ResolveContext;
use crate::parser::pest_parser::{PestRSDLParser, Rule};
use crate::parser::treeconv::treeconv;
use crate::preprocess::preprocess;

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
    stdlib: Option<PathBuf>,

    // -d, --discriminant DISCRIMINANT
    #[structopt(short = "d", long = "discriminant", default_value = "$kind")]
    discriminant: String,
}

/// 启动 RSDL 编译流程
///
/// 这个函数会根据 `RUST_LOG` 设置日志级别、初始化日志系统、
/// 加载预编译的标准库、解析命令行参数、加载代码生成器，
/// 解析用户输入，调用代码生成器生成代码，最后将生成的代码写入输出文件
///
/// 一般而言，下游程序应该直接在 `main` 函数中调用这个函数，并在
/// `prebuilt_stdlib` 参数中传入 [`crate::driver::REFERENTIAL_STDLIB`]
/// 或者自定义的标准库，`build_info` 参数中传入额外的构建信息，
/// `generators` 参数中传入所有要使用的代码生成器。
///
/// # 参数
/// - `prebuilt_stdlib` - 预编译的标准库，在编译输入文件之前加载
/// - `build_info` - 额外的构建信息，例如版权等信息，会在启动时打印
/// - `generators` - 代码生成器工厂列表
pub fn application_start(
    prebuilt_stdlib: &str,
    build_info: Option<&str>,
    generators: &[&dyn CodeGeneratorFactory]
) {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    };

    tracing_subscriber::fmt::init();

    let lang_idents = generators
        .iter()
        .map(|generator| generator.lang_ident())
        .collect::<HashSet<&'static str>>();

    if let Some(build_info) = build_info {
        info!("RSDL 优化编译器 - 非公开构建");
        info!("额外构建信息:");
        for line in build_info.split("\n") {
            info!("{}", line);
        }

        info!("已加载的代码生成器:");
        for generator_factory in generators.iter() {
            info!(
                "  {} -- {}",
                generator_factory.lang_ident(),
                generator_factory.generator_name()
            );
        }
    }

    let opt = Options::from_args();

    if !lang_idents.contains(opt.mode.as_str()) {
        error!("不支持的输出模式 {}", opt.mode);
        return;
    }

    let mut global_attr = Vec::new();
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

        treeconv(&display_name, rsdl, &mut global_attr, &mut tydes);
    } else {
        let rsdl = PestRSDLParser::parse(Rule::rsdl_program, prebuilt_stdlib).unwrap();
        treeconv("(stdlib)", rsdl, &mut global_attr, &mut tydes);
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

        treeconv(&display_name, rsdl, &mut global_attr, &mut tydes);
    }

    let mut resolve_ctx = ResolveContext::new(global_attr, opt.discriminant);

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

    let generator_factory = generators
        .iter()
        .find(|generator| generator.lang_ident() == opt.mode.as_str())
        .unwrap();

    info!("生成结构");
    let mut generator = generator_factory.create();
    if let Ok(output) = codegen(
        opt.namespace.as_ref().map(|s| s.as_str()),
        &tydes,
        &resolve_ctx,
        generator.as_mut()
    ) {
        let output = output.to_string();
        let display_name = format!("{}", opt.output.display());
        #[cfg(windows)] let display_name = display_name.replace("\\\\?\\", "");

        info!("输出文件 {}", display_name);
        if let Err(e) = std::fs::write(&opt.output, output) {
            error!("无法写入输出文件 {}: {}", display_name, e);
        }
    }
}

/// `rsdl` crate 自带的，供参考的标准库
///
/// 该标准库包含了一些常用的类型定义，以及一些常用的函数。可以支持 `rsdl` crate 自带的代码生成器。
/// 如果下游希望实现新的代码生成器，往往需要编写一个新的标准库。
pub const REFERENTIAL_STDLIB: &'static str = include_str!("stdlib.rsdl");
