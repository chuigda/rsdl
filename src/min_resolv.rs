//! 定义了 `rsdl` 代码生成器工作的上下文

use std::collections::HashMap;

use tracing::error;

use crate::parser::hir::{check_inline, RSDLType, TypeDef, TypeDefInner, AttrItem};

/// `rsdl` 代码生成器工作的上下文
pub struct ResolveContext {
    /// 全局注解项
    pub global_attr: Vec<AttrItem>,
    /// 命令行参数定义的 `discriminant` 名称
    pub discriminant: String,
    /// 已知的类型
    ///
    /// 键是类型名称，值是一个三元组，分别是：
    /// - 类型定义所在的文件
    /// - 类型定义的类型
    /// - 类型定义是否是内联的
    pub known_types: HashMap<String, (String, Option<RSDLType>, bool)>
}

impl ResolveContext {
    pub fn new(global_attr: Vec<AttrItem>, discriminant: impl ToString) -> Self {
        Self {
            global_attr,
            discriminant: discriminant.to_string(),
            known_types: HashMap::new()
        }
    }

    pub fn min_resolv(&mut self, tyde: &TypeDef) -> Result<(), ()> {
        match &tyde.inner {
            TypeDefInner::AliasType(name, ty) => {
                if let Some((exist_in_file, _, _)) = self.known_types.get(name) {
                    error!(
                        "{}: 重复的类型定义 {}，此类型先前已经定义于 {}",
                        tyde.file,
                        name,
                        exist_in_file
                    );
                    return Err(());
                }

                let is_inline = check_inline(&tyde.attr);
                self.known_types.insert(name.clone(), (tyde.file.clone(), Some(ty.clone()), is_inline));
            },
            TypeDefInner::SimpleType(ctor) => {
                if let Some((exist_in_file, _, _)) = self.known_types.get(&ctor.name) {
                    error!(
                        "{}: 重复的类型定义 {}，此类型先前已经定义于 {}",
                        tyde.file,
                        ctor.name,
                        exist_in_file
                    );
                    return Err(());
                }

                self.known_types.insert(ctor.name.clone(), (tyde.file.clone(), None, false));
            },
            TypeDefInner::SumType(sum) => {
                if let Some((exist_in_file, _, _)) = self.known_types.get(&sum.name) {
                    error!(
                        "{}: 重复的类型定义 {}，此类型先前已经定义于 {}",
                        tyde.file,
                        sum.name,
                        exist_in_file
                    );
                    return Err(());
                }

                self.known_types.insert(sum.name.clone(), (tyde.file.clone(), None, false));

                if !sum.ctors.is_empty() {
                    for (_, ctor) in &sum.ctors {
                        if let Some((exist_in_file, _, _)) = self.known_types.get(&ctor.name) {
                            error!(
                                "{}: 重复的类型定义 {} (和类型 {} 的构造器)，此类型先前已经定义于 {}",
                                tyde.file,
                                ctor.name,
                                sum.name,
                                exist_in_file
                            );
                            return Err(());
                        }

                        self.known_types.insert(ctor.name.clone(), (tyde.file.clone(), None, false));
                    }

                    for (_, variant) in &sum.scalar_variants {
                        if let Some((exist_in_file, _, _)) = self.known_types.get(variant) {
                            error!(
                                "{}: 重复的类型定义 {} (和类型 {} 的标量变体)，此类型先前已经定义于 {}",
                                tyde.file,
                                variant,
                                sum.name,
                                exist_in_file
                            );
                            return Err(());
                        }

                        self.known_types.insert(variant.clone(), (tyde.file.clone(), None, false));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn min_resolv_chk(&self, tyde: &TypeDef) -> Result<(), ()> {
        match &tyde.inner {
            TypeDefInner::AliasType(name, ty) => if let Err(ident) = self.chktype(ty) {
                error!(
                    "{}: 类型别名 {} 引用了未知的类型 {}",
                    tyde.file,
                    name,
                    ident
                );
                return Err(());
            },
            TypeDefInner::SimpleType(ctor) => {
                for (_, _, ty, name) in &ctor.fields {
                    if let Err(ident) = self.chktype(ty) {
                        error!(
                            "{}: 类型 {} 的字段 {} 引用了未知的类型 {}",
                            tyde.file,
                            ctor.name,
                            name,
                            ident
                        );
                        return Err(());
                    }
                }
            },
            TypeDefInner::SumType(sum_type) => {
                for (_, ctor) in &sum_type.ctors {
                    for (_, _, ty, name) in &ctor.fields {
                        if let Err(ident) = self.chktype(ty) {
                            error!(
                                "{}: 类型 {} 构造器 {} 的字段 {} 引用了未知的类型 {}",
                                tyde.file,
                                sum_type.name,
                                ctor.name,
                                name,
                                ident
                            );
                            return Err(());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn chktype<'a>(&self, ty: &'a RSDLType) -> Result<(), &'a str> {
        match ty {
            RSDLType::Identifier(ident) => {
                if self.known_types.get(ident.as_str()).is_none() {
                    return Err(ident.as_str());
                }
            },
            RSDLType::List(inner) => return self.chktype(inner),
            RSDLType::Record(inner) => return self.chktype(inner),
            _ => {}
        }

        Ok(())
    }
}
