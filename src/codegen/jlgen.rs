#![allow(unused)]

use std::error::Error;

use crate::{
    codegen::CodeGenerator,
    parser::hir::{
        SumType,
        RSDLType,
        AttrItem,
        TypeConstructor,
        check_ident_attr,
        extract_doc_strings
    },
    min_resolv::ResolveContext
};

pub struct JuliaGenerator();

impl JuliaGenerator {
    fn type_to_string(&self, ty: &RSDLType) -> Option<String> {
        match ty {
            RSDLType::Identifier(ident) => Some(ident.to_string()),
            RSDLType::Native(native) => {
                if let Some(jl_name) = native.get("jl") {
                    Some(jl_name.to_string())
                } else if let Some(jl_name) = native.get("julia") {
                    Some(jl_name.to_string())
                } else {
                    None
                }
            },
            RSDLType::List(inner) => {
                let inner = self.type_to_string(&inner)?;
                Some(format!("Vector{{{}}}", inner))
            },
            RSDLType::Record(inner) => {
                let inner = self.type_to_string(&inner)?;
                Some(format!("Dict{{String, {}}}", inner))
            }
        }
    }

    fn check_julia_skip(&self, attr_list: &[AttrItem]) -> bool {
        check_ident_attr(attr_list, "jl_skip") ||
        check_ident_attr(attr_list, "julia_skip")
    }

    fn gen_doc_simple(
        &self,
        attr_list: &[AttrItem],
        doc_attr_name: &str,
        indent: &str,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        let doc_string_lines = extract_doc_strings(attr_list, doc_attr_name)?;
        if doc_string_lines.is_empty() {
            return Ok(());
        }

        output.push(format!("{}\"\"\"", indent));
        for line in doc_string_lines {
            output.push(format!("{}{}", indent, line));
        }
        output.push(format!("{}\"\"\"", indent));
        Ok(())
    }
}

impl CodeGenerator for JuliaGenerator {
    fn generator_name(&self) -> &'static str {
        "Julia 代码生成器"
    }

    fn lang_ident(&self) -> &'static str {
        "julia"
    }

    fn reserved_idents(&self) -> &[&'static str] {
        &[
            // https://docs.julialang.org/en/v1/base/base/
            "baremodule",
            "begin",
            "break",
            "catch",
            "const",
            "continue",
            "do",
            "else",
            "elseif",
            "end",
            "export",
            "false",
            "finally",
            "for",
            "function",
            "global",
            "if",
            "import",
            "let",
            "local",
            "macro",
            "module",
            "quote",
            "return",
            "struct",
            "true",
            "try",
            "using",
            "while"
        ]
    }

    fn visit_namespace_begin(
        &mut self,
        namespace: &str,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        output.push(format!("module {}", namespace));
        output.push("".to_string());
        Ok(())
    }

    fn visit_namespace_end(
        &mut self,
        namespace: &str,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        output.push("".to_string());
        output.push(format!("end # module {}", namespace));
        output.push("".to_string());
        Ok(())
    }

    fn visit_type_alias(
        &mut self,
        _ctx: &ResolveContext,
        attr: &[AttrItem],
        alias_name: &str,
        target_type: &RSDLType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        if self.check_julia_skip(attr) {
            return Ok(());
        }

        self.gen_doc_simple(attr, "doc", "", output)?;

        let target_type_name = self.type_to_string(target_type)
            .ok_or("RSDL native 类型缺少对应的 Julia 类型")?;

        output.push(format!(
            "const {} = {};",
            alias_name,
            target_type_name
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
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type_ctor(
        &mut self,
        _ctx: &ResolveContext,
        attr: &[AttrItem],
        ctor: &TypeConstructor,
        sum_type: &SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type_scalar_variant(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        variant_name: &str,
        sum_type: &SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        sum_type: &SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
