use std::error::Error;

use smallvec::SmallVec;

use crate::{
    codegen::CodeGenerator,
    parser::hir::{
        RSDLType,
        AttrItem,
        TypeConstructor, 
        check_private,
        check_boxed, check_ident_attr
    },
    min_resolv::ResolveContext
};

pub struct RustGenerator();

impl RustGenerator {
    fn type_to_string(&self, ty: &RSDLType) -> Option<String> {
        match ty {
            RSDLType::Identifier(ident) => Some(ident.to_string()),
            RSDLType::Native(native) => {
                if let Some(rust_name) = native.get(self.lang_ident()) {
                    Some(rust_name.to_string())
                } else {
                    None
                }
            },
            RSDLType::List(inner) => {
                let inner = self.type_to_string(&inner)?;
                Some(format!("Vec<{}>", inner))
            },
            RSDLType::Record(inner) => {
                let inner = self.type_to_string(&inner)?;
                Some(format!("HashMap<String, {}>", inner))
            }
        }
    }

    fn gen_rust_derive(
        &self,
        attr_list: &[AttrItem],
        output: &mut Vec<String>
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
                        output.push(format!("#[derive({})]", derived_names.join(", ")));
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
        indent: &str,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        for attr in attr_list {
            match attr {
                AttrItem::CallAlike(fn_alike, param_alike) => {
                    if fn_alike == rust_attr_name {
                        for rust_attr in param_alike {
                            output.push(format!(
                                "{}#[{}]",
                                indent,
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
        indent: &str,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
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
                            output.push(format!("{}/// {}", indent, doc));
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
                            output.push(format!("{}/// {}", indent, doc));
                        } else {
                            return Err(format!(
                                "{} 属性的参数必须是字符串字面量",
                                doc_attr_name
                            ).into());
                        }
                    }
                },
                _ => {}
            }
        }

        Ok(())
    }

    fn imp_visit_simple_type(
        &mut self,
        attr: &[AttrItem],
        type_ctor: &TypeConstructor,
        output: &mut Vec<String>,

        doc_attr_name: &str,
        rust_attr_name: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.check_rust_skip(attr) {
            return Ok(());
        }

        self.gen_doc(attr, doc_attr_name, "", output)?;
        self.gen_rust_attr(attr, rust_attr_name, "", output)?;
        self.gen_rust_derive(attr, output)?;
        let private = check_private(attr);

        output.push(format!(
            "{}struct {} {{",
            if private { "" } else { "pub " },
            type_ctor.name
        ));

        for (attr, optional, field_type, field_name) in &type_ctor.fields {
            self.gen_doc(attr, "doc", "    ", output)?;
            self.gen_rust_attr(attr, "rust_attr", "    ", output)?;
            let field_private = check_private(attr);
            let field_boxed = check_boxed(attr);

            let inner_type = if field_boxed {
                format!("Box<{}>", self.type_to_string(field_type)
                    .ok_or("RSDL native 类型缺少对应的 Rust 类型")?)
            } else {
                self.type_to_string(field_type)
                    .ok_or("RSDL native 类型缺少对应的 Rust 类型")?
            };

            if *optional {
                output.push(format!(
                    "    {}{}: Option<{}>,",
                    if field_private { "" } else { "pub " },
                    field_name,
                    inner_type
                ));
            } else {
                output.push(format!(
                    "    {}{}: {},",
                    if field_private { "" } else { "pub " },
                    field_name,
                    inner_type
                ));
            }
        }

        output.push("}".to_string());
        output.push("".to_string());

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
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn std::error::Error>> {
        output.push(format!("pub mod {} {{", namespace));
        Ok(())
    }

    fn visit_namespace_end(
        &mut self,
        namespace: &str,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn std::error::Error>> {
        output.push(format!("}} // mod {}", namespace));
        Ok(())
    }

    fn visit_type_alias(
        &mut self,
        _ctx: &ResolveContext,
        attr: &[AttrItem],
        alias_name: &str,
        target_type: &crate::parser::hir::RSDLType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.check_rust_skip(attr) {
            return Ok(());
        }

        self.gen_doc(attr, "doc", "", output)?;
        self.gen_rust_attr(attr, "rust_attr", "", output)?;
        let private = check_private(attr);

        output.push(format!(
            "{}type {} = {};",
            if private { "" } else {"pub "},
            alias_name,
            self.type_to_string(target_type)
                .ok_or("RSDL native 类型缺少对应的 Rust 类型")?
        ));
        output.push("".to_string());

        Ok(())
    }

    fn visit_simple_type(
        &mut self,
        _ctx: &ResolveContext,
        attr: &[AttrItem],
        type_ctor: &TypeConstructor,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.imp_visit_simple_type(
            attr,
            type_ctor,
            output,
            "doc",
            "rust_attr"
        )
    }

    fn visit_sum_type_ctor(
        &mut self,
        _ctx: &ResolveContext,
        attr: &[AttrItem],
        ctor: &TypeConstructor,
        _sum_type: &crate::parser::hir::SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.imp_visit_simple_type(
            attr,
            ctor,
            output,
            "doc_ctor",
            "rust_attr_ctor"
        )
    }

    fn visit_sum_type_scalar_variant(
        &mut self,
        _ctx: &ResolveContext,
        _attr: &[AttrItem],
        _variant_name: &str,
        _sum_type: &crate::parser::hir::SumType,
        _output: &mut Vec<String>
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn visit_sum_type(
        &mut self,
        _ctx: &ResolveContext,
        attr: &[AttrItem],
        sum_type: &crate::parser::hir::SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.check_rust_skip(attr) {
            return Ok(());
        }

        self.gen_doc(attr, "doc", "", output)?;
        self.gen_rust_attr(attr, "rust_attr", "", output)?;
        self.gen_rust_derive(attr, output)?;
        let private = check_private(attr);

        output.push(format!(
            "{}enum {} {{",
            if private { "" } else { "pub " },
            sum_type.name
        ));

        for (variant_attr, variant) in &sum_type.scalar_variants {
            self.gen_doc(variant_attr, "doc", "    ", output)?;
            self.gen_rust_attr(variant_attr, "rust_attr", "    ", output)?;

            output.push(format!(
                "    {},",
                variant
            ));
        }

        for (ctor_attr, ctor) in &sum_type.ctors {
            self.gen_doc(ctor_attr, "doc", "    ", output)?;
            self.gen_rust_attr(ctor_attr, "rust_attr", "    ", output)?;

            output.push(format!(
                "    {}({}),",
                ctor.name,
                ctor.name
            ));
        }

        output.push("}".to_string());
        output.push("".to_string());

        Ok(())
    }
}
