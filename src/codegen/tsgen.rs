use std::error::Error;

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
        check_ident_attr,
        check_inline,
        check_private,
        extract_doc_strings
    },
    min_resolv::ResolveContext
};

pub struct TSInterfaceGenerator();

impl TSInterfaceGenerator {
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
                if let Some(ts_name) = native.get("ts") {
                    Some(ts_name.to_string())
                } else if let Some(ts_name) = native.get("typescript") {
                    Some(ts_name.to_string())
                } else {
                    None
                }
            },
            RSDLType::List(inner) => {
                let inner = self.type_to_string(ctx, &inner)?;
                Some(format!("{}[]", inner))
            },
            RSDLType::Record(inner) => {
                let inner = self.type_to_string(ctx, &inner)?;
                Some(format!("Record<string, {}>", inner))
            }
        }
    }

    fn check_ts_skip(&self, attr_list: &[AttrItem]) -> bool {
        check_ident_attr(attr_list, "typescript_skip") || check_ident_attr(attr_list, "ts_skip")
    }

    fn gen_doc(
        &self,
        attr_list: &[AttrItem],
        doc_attr_names: &[&str],
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        let mut doc_string_lines = Vec::new();
        for doc_attr_name in doc_attr_names {
            doc_string_lines.append(&mut extract_doc_strings(attr_list, doc_attr_name)?);
        }

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
        type_ctor: &TypeConstructor,
        output: &mut Doc,

        doc_attr_names: &[&str],
        parent: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        if self.check_ts_skip(attr) {
            return Ok(());
        }
        self.gen_doc(attr, doc_attr_names, output)?;
        let private = check_private(attr);

        if let Some(parent) = parent {
            output.push_string(format!(
                "{}interface {} extends {}Base<\"{}\">{{",
                if private { "" } else { "export " },
                type_ctor.name,
                parent,
                type_ctor.name
            ));
        } else {
            output.push_string(format!(
                "{}interface {} {{",
                if private { "" } else { "export " },
                type_ctor.name
            ));
        }

        let mut fields = Box::new(Doc::new(4));
        for (attr, optional, field_type, field_name) in &type_ctor.fields {
            self.gen_doc(attr, &["doc"], &mut fields)?;

            let inner_type = self.type_to_string(ctx, field_type)
                .ok_or("RSDL native 类型缺少对应的 Typescript 类型")?;

            fields.push_string(format!(
                "{}{}: {},",
                field_name,
                if *optional { "?" } else { "" },
                inner_type
            ));
        }
        output.push_doc(fields);

        output.push_str("}");
        output.push_empty_line();

        Ok(())
    }
}

impl CodeGenerator for TSInterfaceGenerator {
    fn generator_name(&self) -> &'static str {
        "TypeScript (Interface) 代码生成器"
    }

    fn lang_ident(&self) -> &'static str {
        "typescript"
    }

    fn reserved_idents(&self) -> &[&'static str] {
        &[
            // Reserved words
            "break",
            "case",
            "catch",
            "class",
            "const",
            "continue",
            "debugger",
            "default",
            "delete",
            "do",
            "else",
            "enum",
            "export",
            "extends",
            "false",
            "finally",
            "for",
            "function",
            "if",
            "import",
            "in",
            "instanceof",
            "new",
            "null",
            "return",
            "super",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "typeOf",
            "var",
            "void",
            "while",
            "with",

            // Strict Mode Reserved Words
            "as",
            "implements",
            "interface",
            "let",
            "package",
            "private",
            "protected",
            "public",
            "static",
            "yield",
        ]
    }

    fn visit_namespace_begin(
        &mut self,
        _namespace: &str,
        _output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        Err("TypeScript 代码生成器不支持 namespace 功能".into())
    }

    fn visit_namespace_end(
        &mut self,
        _namespace: &str,
        _output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        unreachable!()
    }

    fn visit_type_alias(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        alias_name: &str,
        target_type: &RSDLType,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        if check_inline(attr) || self.check_ts_skip(attr) {
            return Ok(());
        }

        self.gen_doc(attr, &["doc"], output)?;
        let private = check_private(attr);

        output.push_string(format!(
            "{}type {} = {};",
            if private { "" } else { "export " },
            alias_name,
            self.type_to_string(ctx, target_type)
                .ok_or("RSDL native 类型缺少对应的 Typescript 类型")?
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
            type_ctor,
            output,
            &["doc"],
            None
        )
    }

    fn visit_sum_type_ctor(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        ctor: &TypeConstructor,
        sum_type: &SumType,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        self.imp_visit_simple_type(
            ctx,
            attr,
            ctor,
            output,
            &["doc", "doc_ctor"],
            Some(sum_type.name.as_str())
        )
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
        if self.check_ts_skip(attr) {
            return Ok(());
        }

        self.gen_doc(attr, &["doc"], output)?;
        let private = check_private(attr);

        output.push_string(format!(
            "{}type {} = ",
            if private { "" } else { "export " },
            sum_type.name
        ));
        
        let mut sum_variants = Box::new(Doc::new(4));
        for (_, variant) in &sum_type.scalar_variants {
            sum_variants.push_string(format!(
                "| {}",
                variant
            ));
        }

        for (_, ctor) in &sum_type.ctors {
            sum_variants.push_string(format!(
                "| {}",
                ctor.name,
            ));
        }
        output.push_doc(sum_variants);

        output.push_empty_line();

        output.push_string(format!(
            "{}interface {}Base<K extends string> {{",
            if private { "" } else { "export " },
            sum_type.name
        ));
        let mut fields = Box::new(Doc::new(4));
        fields.push_string(format!("{}: K;", ctx.discriminant));
        output.push_doc(fields);
        output.push_string("}".to_string());

        if !sum_type.scalar_variants.is_empty() {
            output.push_empty_line();
        }

        for (variant_attr, variant) in &sum_type.scalar_variants {
            self.gen_doc(variant_attr, &["doc", "doc_ctor"], output)?;
            output.push_string(format!(
                "{}interface {} extends {}Base<\"{}\"> {{}}",
                if private { "" } else { "export " },
                variant,
                sum_type.name,
                variant
            ));
        }

        output.push_empty_line();
        Ok(())
    }
}

pub struct TSInterfaceGeneratorFactory();

impl CodeGeneratorFactory for TSInterfaceGeneratorFactory {
    fn generator_name(&self) -> &'static str {
        TSInterfaceGenerator().generator_name()
    }

    fn lang_ident(&self) -> &'static str {
        TSInterfaceGenerator().lang_ident()
    }

    fn create(&self) -> Box<dyn CodeGenerator> {
        Box::new(TSInterfaceGenerator())
    }
}
