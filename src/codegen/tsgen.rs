use std::{error::Error, fmt::format};

use crate::{
    codegen::CodeGenerator,
    parser::hir::{
        RSDLType,
        AttrItem,
        TypeConstructor,
        check_ident_attr, check_private
    },
    min_resolv::ResolveContext
};


pub struct TSClassGenerator();

impl TSClassGenerator {
    fn type_to_string(&self, ty: &RSDLType) -> Option<String> {
        match ty {
            RSDLType::Identifier(ident) => Some(ident.to_string()),
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
                let inner = self.type_to_string(&inner)?;
                Some(format!("{}[]", inner))
            },
            RSDLType::Record(inner) => {
                let inner = self.type_to_string(&inner)?;
                Some(format!("Record<string, {}>", inner))
            }
        }
    }

    fn check_typescript_skip(&self, attr_list: &[AttrItem]) -> bool {
        check_ident_attr(attr_list, "ts_skip") ||
        check_ident_attr(attr_list, "typescript_skip")
    }
}

impl CodeGenerator for TSClassGenerator {
    fn generator_name(&self) -> &'static str { "基于 class 的TypeScript 代码生成器" }

    fn lang_ident(&self) -> &'static str { "ts-class" }

    fn reserved_idents(&self) -> &[&'static str] {
        // https://github.com/microsoft/TypeScript/issues/2536
        &[
            // 保留字
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
            "typeof",
            "var",
            "void",
            "while",
            "with",
            
            // 严格模式保留字
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
            "symbol",
            "type",
            "from",
            "of"
        ]
    }

    fn visit_namespace_begin(
        &mut self,
        _namespace: &str,
        _output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn visit_namespace_end(
        &mut self,
        _namespace: &str,
        _output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
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
        if self.check_typescript_skip(attr) {
            return Ok(());
        }
        
        let private = check_private(attr);
        output.push(format!(
            "{}type {} = {};\n",
            if private { "" } else { "export " },
            alias_name,
            self.type_to_string(target_type).unwrap()
        ));
        output.push("".to_string());

        Ok(())
    }

    fn visit_simple_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        type_ctor: &TypeConstructor,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type_ctor(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        ctor: &TypeConstructor,
        sum_type: &crate::parser::hir::SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type_scalar_variant(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        variant_name: &str,
        sum_type: &crate::parser::hir::SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        sum_type: &crate::parser::hir::SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
