use std::error::Error;

use crate::{
    codegen::CodeGenerator,
    parser::hir::{
        RSDLType,
        AttrItem,
        TypeConstructor,
        check_private,
        check_boxed,
        check_ident_attr,
        extract_doc_strings
    },
    min_resolv::ResolveContext
};
use crate::codegen::rustgen::RustGenerator;
use crate::parser::hir::SumType;

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
}

impl CodeGenerator for RustGenerator {
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
        ctx: &ResolveContext,
        attr: &[AttrItem],
        alias_name: &str,
        target_type: &RSDLType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_simple_type(&mut self, ctx: &ResolveContext, attr: &[AttrItem], type_ctor: &TypeConstructor, output: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type_ctor(&mut self, ctx: &ResolveContext, attr: &[AttrItem], ctor: &TypeConstructor, sum_type: &SumType, output: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type_scalar_variant(&mut self, ctx: &ResolveContext, attr: &[AttrItem], variant_name: &str, sum_type: &SumType, output: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type(&mut self, ctx: &ResolveContext, attr: &[AttrItem], sum_type: &SumType, output: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
