//! RSDL - Rusty Zephyr ASDL 编译器
//!
//! 这是 RSDL 编译器，以库的形式提供。一般而言，
//! 下游用户只需要实现自己的代码生成器（[`crate::codegen::CodeGenerator`])）
//! 和代码生成器工厂（[`crate::codegen::CodeGeneratorFactory`]），
//! 并编写与之配套的标准库，然后在自己程序的入口点（`main` 函数）调用
//! [`crate::driver::application_start`] 即可。

pub mod codegen;
pub mod driver;
pub mod min_resolv;
pub mod parser;
pub mod preprocess;
