//! `rsdl` 源代码的高阶抽象语法树

use std::collections::HashMap;
use std::error::Error;
use smallvec::SmallVec;

/// 一个注解项
#[derive(Debug, Clone)]
pub enum AttrItem {
    /// 标识符
    /// 
    /// # 示例
    /// ```rsdl
    /// [inline]
    /// ```
    Identifier(String),

    /// 字符串字面量
    /// 
    /// # 示例
    /// ```rsdl
    /// ["这是一个字符串字面量"]
    /// ```
    String(String),

    /// “赋值”形式的注解
    /// 
    /// # 示例
    /// ```rsdl
    /// [doc = "左边是赋值目标，右边是赋值的值，可以是任意的 AttrItem"]
    /// ```
    Assignment(String, Box<AttrItem>),

    /// “调用”形式的注解
    /// 
    /// # 示例
    /// ```rsdl
    /// [doc("这是一个调用形式的注解")]
    /// ```
    /// 
    /// ```rsdl
    /// [doc(
    ///     "这是一个调用形式的注解",
    ///     "这是第二个参数",
    ///     third = "这是第三个参数，是一个赋值形式的注解",
    ///     visibility(hidden, "这是第四个参数，是一个调用形式的注解")
    /// )]
    /// ```
    CallAlike(String, Vec<AttrItem>)
}

/// 一个 RSDL 类型
#[derive(Debug, Clone)]
pub enum RSDLType {
    /// 标识符
    Identifier(String),
    /// `native` 类型
    /// 
    /// # 示例
    /// ```rsdl
    /// native(
    ///     -- Rust 代码生成器应该生成 i32 类型
    ///     rust => "i32",
    ///     -- 而 TypeScript 代码生成器应该生成 number 类型
    ///     typescript => "number"
    /// )
    Native(HashMap<String, String>),
    /// 列表（数组）类型
    /// 
    /// # 示例
    /// ```rsdl
    /// list<int>
    /// ```
    /// 
    /// ```rsdl
    /// [int] -- 与上面的写法等价
    /// ```
    List(Box<RSDLType>),
    /// 记录类型
    /// 
    /// # 示例
    /// ```rsdl
    /// record<str, SomeType>
    /// ```
    Record(Box<RSDLType>)
}

/// 一个 RSDL 类型构造器
/// 
/// 简单类型只有一个构造器，而和类型可以有零个或多个构造器
/// 
/// # 示例
/// ```rsdl
/// SimpleType(
///     a: int,
///     b?: str,
///     [boxed] c: SomeType
///     d: list<SomeType>
/// )
#[derive(Debug, Clone)]
pub struct TypeConstructor {
    /// 构造器的名称
    pub name: String,

    /// 构造器的字段列表
    /// 
    /// 元组中的四个元素分别是：
    /// - 字段的注解
    /// - 字段是否可空
    /// - 字段的类型
    /// - 字段的名称
    pub fields: Vec<(SmallVec<[AttrItem; 2]>, bool, RSDLType, String)>
}

/// 一个 RSDL 和类型
/// 
/// 一个和类型中有多个构造器和多个标量变体
/// 
/// # 示例
/// 
/// ```rsdl
/// DateTime : POSIXTimestamp(timestamp: int)
///          | ISO8601String(iso8601: str)
///          | RFC3339Elaborated(year: int, month: int, day: int, hour: int, minute: int, second: float, timezone: int)
///          | UnknownDateTime
#[derive(Debug, Clone)]
pub struct SumType {
    /// 和类型的名称
    pub name: String,

    /// 标量变体
    /// 
    /// 元组中的两个元素分别是：
    /// - 变体的注解
    /// - 变体的名称
    pub scalar_variants: Vec<(SmallVec<[AttrItem; 2]>, String)>,

    /// 构造器
    pub ctors: Vec<(SmallVec<[AttrItem; 2]>, TypeConstructor)>
}

/// 一个 RSDL 类型定义的“内容”
#[derive(Debug, Clone)]
pub enum TypeDefInner {
    /// 类型别名
    AliasType(String, RSDLType),
    /// 简单类型
    SimpleType(TypeConstructor),
    /// 和类型
    SumType(SumType)
}

/// 一个 RSDL 类型定义
#[derive(Debug, Clone)]
pub struct TypeDef {
    /// 类型定义所在的文件
    pub file: String,
    /// 类型定义的注解
    pub attr: SmallVec<[AttrItem; 2]>,
    /// 实际的类型定义内容
    pub inner: TypeDefInner
}

/// 检查一个注解列表中是否包含某个标识符注解
pub fn check_ident_attr(attr_list: &[AttrItem], checked_ident: &str) -> bool {
    for attr in attr_list {
        if let AttrItem::Identifier(ident) = attr {
            if ident == checked_ident {
                return true;
            }
        }
    }
    false
}

/// 检查一个注解列表中是否包含 `private` 注解
pub fn check_private(attr_list: &[AttrItem]) -> bool {
    check_ident_attr(attr_list, "private")
}

/// 检查一个注解列表中是否包含 `boxed` 注解
pub fn check_boxed(attr_list: &[AttrItem]) -> bool {
    check_ident_attr(attr_list, "boxed")
}

/// 提取一个注解列表中的所有文档字符串
/// 
/// # 示例
/// ```rsdl
/// [doc("这是一个文档字符串")]
/// [doc = "这是另一个文档字符串
/// 这个文档字符串有两行"]
/// ```
/// 
/// 提取出来的文档字符串是：
/// ```rust,no_run
/// vec![
///     "这是一个文档字符串".to_string(),
///     "这是另一个文档字符串".to_string(),
///     "这个文档字符串有两行".to_string()
/// ]
/// ```
pub fn extract_doc_strings(
    attr_list: &[AttrItem],
    doc_attr_name: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut ret = Vec::new();

    fn add_doc_string(output: &mut Vec<String>, doc_string: &str) {
        if doc_string.contains("\n") {
            for line in doc_string.split("\n") {
                output.push(line.trim().to_string());
            }
        } else {
            output.push(doc_string.trim().to_string());
        }
    }

    for attr in attr_list {
        match attr {
            AttrItem::CallAlike(fn_alike, param_alike) => {
                if fn_alike == doc_attr_name {
                    if param_alike.len() != 1 {
                        return Err(format!(
                            "{} 属性的参数数量必须为 1，但是此处有 {} 个参数",
                            doc_attr_name,
                            param_alike.len()
                        ).into());
                    }

                    if let AttrItem::String(doc) = &param_alike[0] {
                        add_doc_string(&mut ret, doc);
                    } else {
                        return Err(format!(
                            "{} 属性的参数必须是字符串字面量",
                            doc_attr_name
                        ).into());
                    }
                }
            },
            AttrItem::Assignment(assignee, value) => {
                if assignee == doc_attr_name {
                    if let AttrItem::String(doc) = value.as_ref() {
                        add_doc_string(&mut ret, &doc);
                    } else {
                        return Err(format!(
                            "{} 属性的值必须是字符串字面量",
                            doc_attr_name
                        ).into());
                    }
                }
            },
            _ => {}
        }
    }

    Ok(ret)
}

/// 检查一个注解列表中是否包含 `inline` 注解
pub fn check_inline(attr_list: &[AttrItem]) -> bool {
    check_ident_attr(attr_list, "inline")
}
