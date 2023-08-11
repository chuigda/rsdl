pub mod rustgen;

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

pub trait CodeGenerator {
    fn generator_name(&self) -> &'static str;
    fn lang_ident(&self) -> &'static str;
    fn reserved_idents(&self) -> &[&'static str];

    fn visit_namespace_begin(
        &mut self,
        namespace: &str,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>>;

    fn visit_namespace_end(
        &mut self,
        namespace: &str,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>>;

    fn visit_type_alias(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        alias_name: &str,
        target_type: &RSDLType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>>;

    fn visit_simple_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        type_ctor: &TypeConstructor,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>>;

    fn visit_sum_type_ctor(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        ctor: &TypeConstructor,
        sum_type: &SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>>;

    fn visit_sum_type_scalar_variant(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        variant_name: &str,
        sum_type: &SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>>;

    fn visit_sum_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        sum_type: &SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>>;

    // downstream codegen may overwrite this, or just ignore it
    fn visit_all_typedefs(
        &mut self,
        _ctx: &ResolveContext,
        _typedefs: &[TypeDef],
        _output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

pub fn codegen(
    namespace: Option<&str>,
    tyde: &[TypeDef],
    ctx: &ResolveContext,
    codegen: &mut dyn CodeGenerator
) -> Result<Vec<String>, Box<dyn Error>> {
    if let Err(e) = check_reserved_idents(ctx, codegen, namespace, tyde) {
        error!("{e}");
        return Err(e);
    }

    let mut output = Vec::new();
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

pub trait CodeGeneratorFactory {
    fn generator_name(&self) -> &'static str;
    fn lang_ident(&self) -> &'static str;

    fn create(&self) -> Box<dyn CodeGenerator>;
}
