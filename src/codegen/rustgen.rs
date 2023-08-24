use std::error::Error;

use smallvec::SmallVec;

use crate::{
    codegen::{
        CodeGenerator,
        CodeGeneratorFactory,
        Doc
    },
    parser::hir::{
        SumType,
        RSDLType,
        AttrItem,
        TypeConstructor,
        check_boxed,
        check_ident_attr,
        check_inline,
        check_private,
        extract_doc_strings
    },
    min_resolv::ResolveContext
};

pub struct RustGenerator();

impl RustGenerator {
    fn type_to_string(&self, ctx: &ResolveContext, ty: &RSDLType) -> Option<String> {
        match ty {
            RSDLType::Identifier(ident) => {
                if let Some((_, rsdl_type, is_inline)) = ctx.known_types.get(ident) {
                    if *is_inline {
                        let Some(rsdl_type) = rsdl_type else { return None; };
                        self.type_to_string(ctx, rsdl_type)
                    } else {
                        Some(ident.to_string())
                    }
                } else {
                    None
                }
            },
            RSDLType::Native(native) => {
                if let Some(rust_name) = native.get(self.lang_ident()) {
                    Some(rust_name.to_string())
                } else {
                    None
                }
            },
            RSDLType::List(inner) => {
                let inner = self.type_to_string(ctx, &inner)?;
                Some(format!("Vec<{}>", inner))
            },
            RSDLType::Record(inner) => {
                let inner = self.type_to_string(ctx, &inner)?;
                Some(format!("HashMap<String, {}>", inner))
            }
        }
    }

    fn gen_rust_derive(
        &self,
        attr_list: &[AttrItem],
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        for attr in attr_list {
            let mut derived_names: SmallVec<[&str; 2]> = SmallVec::new();
            if let AttrItem::CallAlike(fn_alike, param_alike) = attr {
                if fn_alike == "rust_derive" {
                    for param in param_alike {
                        if let AttrItem::Identifier(ident) = param {
                            derived_names.push(ident);
                        } else {
                            return Err("derive 属性的参数必须是标识符".into());
                        }
                    }

                    if !derived_names.is_empty() {
                        output.push_string(format!("#[derive({})]", derived_names.join(", ")));
                    }
                }
            }
        }

        Ok(())
    }

    fn check_rust_skip(&self, attr_list: &[AttrItem]) -> bool {
        check_ident_attr(attr_list, "rust_skip")
    }

    fn gen_rust_attr(
        &self,
        attr_list: &[AttrItem],
        rust_attr_name: &str,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        for attr in attr_list {
            match attr {
                AttrItem::CallAlike(fn_alike, param_alike) => {
                    if fn_alike == rust_attr_name {
                        for rust_attr in param_alike {
                            output.push_string(format!(
                                "#[{}]",
                                self.gen_single_rust_attr(rust_attr)?
                            ));
                        }
                    }
                },
                _ => {}
            }
        }

        Ok(())
    }

    fn gen_single_rust_attr(
        &self,
        attr: &AttrItem
    ) -> Result<String, Box<dyn Error>> {
        match attr {
            AttrItem::Identifier(ident) => Ok(ident.to_string()),
            AttrItem::CallAlike(fn_alike, param_alike) => {
                let param_str = param_alike
                    .iter()
                    .map(|param| self.gen_single_rust_attr(param))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");

                Ok(format!("{}({})", fn_alike, param_str))
            },
            AttrItem::Assignment(assignee, value) => {
                match value.as_ref() {
                    AttrItem::String(s) => Ok(format!("{} = \"{}\"", assignee, s)),
                    AttrItem::Identifier(ident) => Ok(format!("{} = {}", assignee, ident)),
                    _ => Err("属性赋值的值必须是字符串字面量或标识符".into())
                }
            },
            _ => Err("属性必须是标识符、函数调用或赋值".into())
        }
    }

    fn gen_doc(
        &self,
        attr_list: &[AttrItem],
        doc_attr_name: &str,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        let doc_string_lines = extract_doc_strings(attr_list, doc_attr_name)?;
        if doc_string_lines.is_empty() {
            return Ok(());
        }

        for line in doc_string_lines {
            output.push_string(format!("/// {}", line));
        }

        Ok(())
    }

    fn imp_visit_simple_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        sum_type_attr: Option<&[AttrItem]>,
        type_ctor: &TypeConstructor,
        output: &mut Doc,

        doc_attr_name: &str,
        rust_attr_name: &str
    ) -> Result<(), Box<dyn Error>> {
        if self.check_rust_skip(attr) {
            return Ok(());
        }

        self.gen_doc(attr, doc_attr_name, output)?;
        self.gen_rust_derive(attr, output)?;
        if let Some(sum_type_attr) = sum_type_attr {
            self.gen_rust_derive(sum_type_attr, output)?;
        }
        self.gen_rust_attr(attr, rust_attr_name, output)?;
        let private = check_private(attr);

        output.push_string(format!(
            "{}struct {} {{",
            if private { "" } else { "pub " },
            type_ctor.name
        ));

        let mut struct_fields = Box::new(Doc::new(4));

        for (attr, optional, field_type, field_name) in &type_ctor.fields {
            self.gen_doc(attr, "doc", &mut struct_fields)?;
            self.gen_rust_attr(attr, "rust_attr", &mut struct_fields)?;
            let field_private = check_private(attr);
            let field_boxed = check_boxed(attr);

            let inner_type = if field_boxed {
                format!("Box<{}>", self.type_to_string(ctx, field_type)
                    .ok_or("RSDL native 类型缺少对应的 Rust 类型")?)
            } else {
                self.type_to_string(ctx, field_type)
                    .ok_or("RSDL native 类型缺少对应的 Rust 类型")?
            };

            if *optional {
                struct_fields.push_string(format!(
                    "{}{}: Option<{}>,",
                    if field_private { "" } else { "pub " },
                    field_name,
                    inner_type
                ));
            } else {
                struct_fields.push_string(format!(
                    "{}{}: {},",
                    if field_private { "" } else { "pub " },
                    field_name,
                    inner_type
                ));
            }
        }

        output.push_doc(struct_fields);

        output.push_str("}");
        output.push_empty_line();

        Ok(())
    }
}

impl CodeGenerator for RustGenerator {
    fn generator_name(&self) -> &'static str { "Rust 代码生成器" }

    fn lang_ident(&self) -> &'static str { "rust" }

    fn reserved_idents(&self) -> &[&'static str] {
        // https://doc.rust-lang.org/reference/keywords.html
        &[
            // 严格关键字
            "as",
            "break",
            "const",
            "continue",
            "crate",
            "else",
            "enum",
            "extern",
            "false",
            "fn",
            "for",
            "if",
            "impl",
            "in",
            "let",
            "loop",
            "match",
            "mod",
            "move",
            "mut",
            "pub",
            "ref",
            "return",
            "self",
            "Self",
            "static",
            "struct",
            "super",
            "trait",
            "true",
            "type",
            "unsafe",
            "use",
            "where",
            "while",

            // 保留关键字
            "abstract",
            "become",
            "box",
            "do",
            "final",
            "macro",
            "override",
            "priv",
            "typeof",
            "unsized",
            "virtual",
            "yield",

            // Rust 2018 新增保留关键字
            "try"
        ]
    }

    fn visit_namespace_begin(
        &mut self,
        namespace: &str,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        output.push_string(format!("mod {} {{", namespace));
        Ok(())
    }

    fn visit_namespace_end(
        &mut self,
        _namespace: &str,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        output.push_str("}");
        Ok(())
    }

    fn visit_type_alias(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        alias_name: &str,
        target_type: &RSDLType,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        if check_inline(attr) || self.check_rust_skip(attr) {
            return Ok(());
        }

        self.gen_doc(attr, "doc", output)?;
        self.gen_rust_attr(attr, "rust_attr", output)?;
        let private = check_private(attr);

        output.push_string(format!(
            "{}type {} = {};",
            if private { "" } else {"pub "},
            alias_name,
            self.type_to_string(ctx, target_type)
                .ok_or("RSDL native 类型缺少对应的 Rust 类型")?
        ));
        output.push_empty_line();

        Ok(())
    }

    fn visit_simple_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        type_ctor: &TypeConstructor,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        self.imp_visit_simple_type(
            ctx,
            attr,
            None,
            type_ctor,
            output,
            "doc",
            "rust_attr"
        )
    }

    fn visit_sum_type_ctor(
        &mut self,
        _ctx: &ResolveContext,
        _attr: &[AttrItem],
        _ctor: &TypeConstructor,
        _sum_type: &SumType,
        _output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn visit_sum_type_scalar_variant(
        &mut self,
        _ctx: &ResolveContext,
        _attr: &[AttrItem],
        _variant_name: &str,
        _sum_type: &SumType,
        _output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn visit_sum_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        sum_type: &SumType,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        if self.check_rust_skip(attr) {
            return Ok(());
        }

        self.gen_doc(attr, "doc", output)?;
        self.gen_rust_derive(attr, output)?;
        self.gen_rust_attr(attr, "rust_attr", output)?;
        let private = check_private(attr);

        output.push_string(format!(
            "{}enum {} {{",
            if private { "" } else { "pub " },
            sum_type.name
        ));

        let mut enum_variants = Box::new(Doc::new(4));

        for (variant_attr, variant) in &sum_type.scalar_variants {
            self.gen_doc(variant_attr, "doc", &mut enum_variants)?;
            self.gen_rust_attr(variant_attr, "rust_attr", &mut enum_variants)?;

            enum_variants.push_string(format!(
                "{},",
                variant
            ));
        }

        for (ctor_attr, ctor) in &sum_type.ctors {
            self.gen_doc(ctor_attr, "doc", &mut enum_variants)?;
            self.gen_rust_attr(ctor_attr, "rust_attr", &mut enum_variants)?;

            if check_boxed(&ctor_attr) {
                enum_variants.push_string(format!(
                    "{}(Box<{}>),",
                    ctor.name,
                    ctor.name
                ));
            } else {
                enum_variants.push_string(format!(
                    "{}({}),",
                    ctor.name,
                    ctor.name
                ));
            }
        }

        output.push_doc(enum_variants);

        output.push_str("}");
        output.push_empty_line();

        for (ctor_attr, ctor) in &sum_type.ctors {
            self.imp_visit_simple_type(
                ctx,
                ctor_attr,
                Some(attr),
                ctor,
                output,
                "doc_ctor",
                "rust_attr_ctor"
            )?;
        }

        Ok(())
    }
}

pub struct RustGeneratorFactory();

impl CodeGeneratorFactory for RustGeneratorFactory {
    fn generator_name(&self) -> &'static str {
        RustGenerator().generator_name()
    }

    fn lang_ident(&self) -> &'static str {
        RustGenerator().lang_ident()
    }

    fn create(&self) -> Box<dyn CodeGenerator> {
        Box::new(RustGenerator())
    }
}
