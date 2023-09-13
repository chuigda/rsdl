//! `rsdl` 的预处理器模块

use tracing::warn;

/// 对单个文件预处理的结果
#[derive(Debug, Clone)]
pub struct PreprocessResult {
    /// 输出代码，`#include` 预处理指令和注释已被移除
    pub output_src: String,
    /// 该文件通过 `#include` 包含的其他文件
    pub includes: Vec<String>
}

/// 对单个文件进行预处理
///
/// 一般而言，下游不应该直接使用此函数。请参见 [`crate::driver::application_start`]
pub fn preprocess(file_name: &str, src: &str) -> PreprocessResult {
    let mut output_src = String::new();
    let mut includes = Vec::new();

    for (idx, line) in src.split("\n").enumerate() {
        let lineno = idx + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("#!") {
            output_src.push('\n');
            continue;
        }
        else if trimmed.starts_with("#include ") {
            let included_file = trimmed[9..].trim();
            if !included_file.starts_with('"') || !included_file.ends_with('"') {
                warn!("{file_name}:{lineno}: 无效的 #include: {line}");
                continue;
            }

            includes.push(included_file[1..included_file.len() - 1].to_string());
        }
        else if trimmed.starts_with("#") {
            warn!("{file_name}:{lineno}: 无效的预处理指令: {line}");
        }
        else if trimmed.starts_with("include ") {
            // be compatible with previous version
            let included_module = trimmed[8..].trim();
            let module_path = included_module.split(".");

            #[cfg(windows)] let mut module_path = module_path.collect::<Vec<&str>>().join("\\");
            #[cfg(not(windows))] let mut module_path = module_path.collect::<Vec<&str>>().join("/");

            module_path.push_str(".asdl");
            includes.push(module_path);
        }
        else {
            let splitted = line.split("--");
            let mut iter = splitted.into_iter();
            output_src.push_str(iter.next().unwrap());
            output_src.push('\n');
        }
    }

    PreprocessResult {
        output_src,
        includes
    }
}
