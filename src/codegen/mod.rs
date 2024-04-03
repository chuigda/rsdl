//! 代码生成器所需的类型定义和 `trait`

pub mod rustgen;
pub mod tsgen;
// pub mod pl5gen;

use std::collections::HashSet;
use std::error::Error;
use std::ops::Deref;

use tracing::error;

use crate::min_resolv::ResolveContext;
use crate::parser::hir::{
    AttrItem,
    RSDLType,
    SumType,
    TypeConstructor,
    TypeDef,
    TypeDefInner
};

/// 用于输出生成代码的文档
///
/// 一个文档中可以包含多个 `DocItem`，包括：
/// - `DocItem::Text`：文本
/// - `DocItem::TextLiteral`：文本字面量，用于输出固定的文本
/// - `DocItem::EmptyLine`：空行
/// - `DocItem::DocBlock`：子文档块
///
/// 文档可以嵌套，并且缩进会被正确处理
///
/// # 示例
///
/// ```rust,no_run
/// use rsdl::codegen::Doc;
///
/// let mut doc = Doc::new(0);
/// doc.push_str("Hello, world!");
///
/// let mut sub_doc = Doc::new(4);
/// sub_doc.push_str("Zdravstvuyte, mir");
/// sub_doc.push_str("Hola Mundo");
///
/// let mut sub_doc2 = Doc::new(4);
/// sub_doc2.push_str("Bonjour le monde");
/// sub_doc2.push_empty_line();
/// let user_name = some_user_input();
/// sub_doc2.push_string(format!("Hello, {}", user_name));
///
/// sub_doc.push_doc(Box::new(sub_doc2));
/// doc.push_doc(Box::new(sub_doc));
/// println!("{}", doc.to_string());
/// ```
///
/// 输出：
///
/// ```text
/// Hello, world!
///     Zdravstvuyte, mir
///     Hola Mundo
///         Bonjour le monde
///
///         Hello, (user input)
/// ```
#[derive(Debug)]
pub struct Doc {
    pub(crate) indent: u32,
    pub(crate) items: Vec<DocItem>
}

impl Doc {
    /// 创建一个具有 `indent` 缩进的文档
    pub fn new(indent: u32) -> Self {
        Self {
            indent,
            items: Vec::new()
        }
    }

    /// 向文档中添加一个 `DocItem`
    ///
    /// 不建议直接使用此方法，而是使用 `push_str`、`push_string`、`push_empty_line`、`push_doc`
    pub fn push(&mut self, item: DocItem) -> &mut Self {
        self.items.push(item);
        self
    }

    /// 向文档中添加一个子文档块
    pub fn push_doc(&mut self, doc: Box<Doc>) -> &mut Self {
        self.items.push(DocItem::DocBlock(doc));
        self
    }

    /// 向文档中添加一行文本（字面量）
    ///
    /// 注意：不要一次性用 `push_str` 添加多行文本，这种情况下应该多次调用 `push_str`
    pub fn push_str(&mut self, s: &'static str) -> &mut Self {
        self.items.push(DocItem::TextLiteral(s));
        self
    }

    /// 向文档中添加一行文本
    ///
    /// 注意：不要一次性用 `push_string` 添加多行文本，这种情况下应该多次调用 `push_string`
    pub fn push_string(&mut self, s: String) -> &mut Self {
        self.items.push(DocItem::Text(s));
        self
    }

    /// 向文档中添加一个空行
    ///
    /// 相比于 `push_str("")` 而言，`push_empty_line` 不受缩进影响
    pub fn push_empty_line(&mut self) -> &mut Self {
        self.items.push(DocItem::EmptyLine);
        self
    }
}

impl ToString for Doc {
    fn to_string(&self) -> String {
        let mut buf = String::new();
        doc_to_string_impl(&mut buf, self.indent, self);
        buf
    }
}

fn doc_to_string_impl(buf: &mut String, indent: u32, doc: &Doc) {
    let indent_str = " ".repeat(indent as usize);

    for (idx, item) in doc.items.iter().enumerate() {
        match item {
            DocItem::Text(s) => {
                buf.push_str(&indent_str);
                buf.push_str(s);
                buf.push_str("\n");
            },
            DocItem::TextLiteral(s) => {
                buf.push_str(&indent_str);
                buf.push_str(s);
                buf.push_str("\n");
            },
            DocItem::EmptyLine => {
                if idx + 1 == doc.items.len() {
                    continue;
                }
                buf.push('\n');
            },
            DocItem::DocBlock(doc) => {
                doc_to_string_impl(buf, indent + doc.indent, doc);
            }
        }
    }
}

/// 文档中的一个元素
#[derive(Debug)]
pub enum DocItem {
    Text(String),
    TextLiteral(&'static str),
    EmptyLine,
    DocBlock(Box<Doc>),
}

/// 代码生成器 `trait`
///
/// 下游代码可以实现此 `trait` 来编写自己的代码生成器。代码生成器会按照如下顺序，
/// 调用 `CodeGenerator` 中的方法：
///
/// - `pre_visit`
/// - `visit_namespace_begin`
/// - `visit_all_typedefs`
///     - `visit_type_alias`
///     - `visit_simple_type`
///     - `visit_sum_type`
///         - `visit_sum_type_scalar_variant`
///         - `visit_sum_type_ctor`
/// - `visit_namespace_end`
pub trait CodeGenerator {
    /// 报告代码生成器的用户可见名称
    ///
    /// 这个名称会在命令行以及日志中显示
    fn generator_name(&self) -> &'static str;

    /// 报告代码生成器的语言标识符
    fn lang_ident(&self) -> &'static str;

    /// 报告语言中的保留标识符
    ///
    /// 在执行代码生成之前，`rsdl` 编译器会检查即将生成的代码中是否包含保留标识符，
    /// 并且提前给出错误信息
    fn reserved_idents(&self) -> &[&'static str];

    /// 进入名称空间时代码生成器的行为
    ///
    /// 当用户通过命令行指定了 `--namespace` 参数时，`rsdl` 编译器会在开始生成代码时
    /// 调用此方法
    ///
    /// 不支持名称空间的代码生成器应该返回 `Err`（推荐的行为），或者也可以忽略此方法
    fn visit_namespace_begin(
        &mut self,
        namespace: &str,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>>;

    /// 离开名称空间时代码生成器的行为
    ///
    /// 当用户通过命令行指定了 `--namespace` 参数时，`rsdl` 编译器会在结束生成代码时
    /// 调用此方法
    ///
    /// 不支持名称空间的代码生成器应该调用 `unreachable!()`，或者也可以忽略此方法
    fn visit_namespace_end(
        &mut self,
        namespace: &str,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>>;

    /// 生成类型别名时代码生成器的行为
    fn visit_type_alias(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        alias_name: &str,
        target_type: &RSDLType,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>>;

    /// 生成简单类型时代码生成器的行为
    fn visit_simple_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        type_ctor: &TypeConstructor,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>>;

    /// 生成和类型时代码生成器的行为
    ///
    /// 注意：`rsdl` 在调用 `visit_sum_type` 之后，会立即对和类型所有的构造器调用
    /// `visit_sum_type_ctor`，对所有的标量变体调用 `visit_sum_type_scalar_variant`。
    /// 你可以选择在 `visit_sum_type_ctor` 和 `visit_sum_type_scalar_variant`
    /// 中处理和类型的构造器和标量变体，或者也可以将这两个函数的实现留空，
    /// 在 `visit_sum_type` 中处理所有的构造器和标量变体。
    fn visit_sum_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        sum_type: &SumType,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>>;

    /// 生成和类型的构造器时代码生成器的行为
    fn visit_sum_type_ctor(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        ctor: &TypeConstructor,
        sum_type: &SumType,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>>;

    /// 生成和类型的标量变体时代码生成器的行为
    fn visit_sum_type_scalar_variant(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        variant_name: &str,
        sum_type: &SumType,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>>;

    /// 实现此方法并将其他所有 `visit` 方法留空，可以完全地自定义代码生成器的行为
    fn visit_all_typedefs(
        &mut self,
        _ctx: &ResolveContext,
        _typedefs: &[TypeDef],
        _output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// 在开始生成代码时的行为
    ///
    /// 用户可以通过实现此方法来向生成的代码中添加一些头部信息，
    /// 例如版权信息、导入语句等
    fn pre_visit(
        &mut self,
        _ctx: &ResolveContext,
        _output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

/// 生成代码
///
/// 一般而言，下游不应该直接使用此函数。请参见 [`crate::driver::application_start`]
pub fn codegen(
    namespace: Option<&str>,
    tyde: &[TypeDef],
    ctx: &ResolveContext,
    codegen: &mut dyn CodeGenerator
) -> Result<Doc, Box<dyn Error>> {
    if let Err(e) = check_reserved_idents(ctx, codegen, namespace, tyde) {
        error!("{e}");
        return Err(e);
    }

    let mut output = Doc::new(0);
    codegen.pre_visit(ctx, &mut output)?;

    if let Some(namespace) = namespace {
        codegen.visit_namespace_begin(namespace, &mut output)
            .map_err(|err| {
                error!(
                    "{}: 进入命名空间 {} 时遇到错误: {}",
                    codegen.generator_name(),
                    namespace,
                    err
                );
                err
            })?;
    }

    codegen.visit_all_typedefs(ctx, tyde, &mut output)
        .map_err(|err| {
            error!(
                "{}: 生成文件 {} 时遇到错误: {}",
                codegen.generator_name(),
                tyde[0].file,
                err
            );
            err
        })?;

    for d in tyde {
        match &d.inner {
            TypeDefInner::AliasType(name, aliased) => {
                codegen.visit_type_alias(ctx, &d.attr, &name, &aliased, &mut output)
                    .map_err(|err| {
                        error!(
                            "{}: 生成文件 {} 中的类型别名 {} 时遇到错误: {}",
                            codegen.generator_name(),
                            d.file,
                            name,
                            err
                        );
                        err
                    })?
            },
            TypeDefInner::SimpleType(simple_type) => {
                codegen.visit_simple_type(ctx, &d.attr, &simple_type, &mut output)
                    .map_err(|err| {
                        error!(
                            "{}: 生成文件 {} 中的简单类型 {} 时遇到错误: {}",
                            codegen.generator_name(),
                            d.file,
                            simple_type.name,
                            err
                        );
                        err
                    })?
            },
            TypeDefInner::SumType(sum_type) => {
                codegen.visit_sum_type(ctx, &d.attr, &sum_type, &mut output)
                    .map_err(|err| {
                        error!(
                            "{}: 生成文件 {} 中的和类型 {} 时遇到错误: {}",
                            codegen.generator_name(),
                            d.file,
                            sum_type.name,
                            err
                        );
                        err
                    })?;

                for (attr, variant) in &sum_type.scalar_variants {
                    codegen.visit_sum_type_scalar_variant(
                        ctx,
                        attr,
                        variant,
                        &sum_type,
                        &mut output
                    ).map_err(|err| {
                        error!(
                            "{}: 生成文件 {} 中的和类型 {} 的变体 {} 时遇到错误: {}",
                            codegen.generator_name(),
                            d.file,
                            sum_type.name,
                            variant,
                            err
                        );
                        err
                    })?;
                }

                for (attr, ctor) in &sum_type.ctors {
                    codegen.visit_sum_type_ctor(
                        ctx,
                        attr,
                        ctor,
                        &sum_type,
                        &mut output
                    ).map_err(|err| {
                        error!(
                            "{}: 生成文件 {} 中的和类型 {} 的构造函数 {} 时遇到错误: {}",
                            codegen.generator_name(),
                            d.file,
                            sum_type.name,
                            ctor.name,
                            err
                        );
                        err
                    })?;
                }
            }
        }
    }

    if let Some(namespace) = namespace {
        codegen.visit_namespace_end(namespace, &mut output)
            .map_err(|err| {
                error!(
                    "{}: 离开命名空间 {} 时遇到错误: {}",
                    codegen.generator_name(),
                    namespace,
                    err
                );
                err
            })?;
    }

    Ok(output)
}

fn check_reserved_idents(
    ctx: &ResolveContext,
    codegen: &dyn CodeGenerator,
    namespace: Option<&str>,
    tyde: &[TypeDef]
) -> Result<(), Box<dyn Error>> {
    let reserved_idents = codegen.reserved_idents()
        .into_iter()
        .map(Deref::deref)
        .collect::<HashSet<_>>();

    if let Some(namespace) = namespace {
        if reserved_idents.contains(namespace) {
            return Err(format!(
                "{}: 生成器报告命名空间名称 {} 与保留标识符冲突",
                codegen.generator_name(),
                namespace
            ).into());
        }
    }

    for (ty_name, (exist_in_file, _, is_inline)) in ctx.known_types.iter() {
        if reserved_idents.contains(ty_name.as_str()) && !is_inline {
            return Err(format!(
                "{}: 生成器报告文件 {} 中的非内联类型 {} 与保留标识符冲突",
                codegen.generator_name(),
                exist_in_file,
                ty_name
            ).into());
        }
    }

    for tyde in tyde {
        match &tyde.inner {
            TypeDefInner::SimpleType(simple_type) => {
                for (_, _, _, field_name) in &simple_type.fields {
                    if reserved_idents.contains(field_name.as_str()) {
                        return Err(format!(
                            "{}: 生成器报告文件 {} 中的简单类型 {} 的字段 {} 与保留标识符冲突",
                            codegen.generator_name(),
                            tyde.file,
                            simple_type.name,
                            field_name
                        ).into());
                    }
                }
            },
            TypeDefInner::SumType(sum_type) => {
                for (_, ctor) in &sum_type.ctors {
                    for (_, _, _, field_name) in &ctor.fields {
                        if reserved_idents.contains(field_name.as_str()) {
                            return Err(format!(
                                "{}: 生成器报告文件 {} 中的和类型 {} 的构造函数 {} 的字段 {} 与保留标识符冲突",
                                codegen.generator_name(),
                                tyde.file,
                                sum_type.name,
                                ctor.name,
                                field_name
                            ).into());
                        }
                    }
                }
            },
            _ => {}
        }
    }

    Ok(())
}

/// 代码生成器工厂，用于创建代码生成器
pub trait CodeGeneratorFactory {
    /// 报告代码生成器的用户可见名称
    ///
    /// 这个名称会在命令行以及日志中显示
    fn generator_name(&self) -> &'static str;

    /// 报告代码生成器的语言标识符
    ///
    /// 语言标识符用于在命令行参数中指定代码生成器
    fn lang_ident(&self) -> &'static str;

    /// 创建代码生成器
    fn create(&self) -> Box<dyn CodeGenerator>;
}
